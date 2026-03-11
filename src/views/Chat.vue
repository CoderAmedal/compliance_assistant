<template>
  <div class="chat-container" ref="containerRef">
    <div class="session-panel" :style="{ width: siderWidth + 'px' }">
      <div class="session-header">
        <a-button type="primary" block @click="handleNewSession">
          <plus-outlined /> 新建对话
        </a-button>
      </div>
      <a-menu
        v-model:selectedKeys="selectedSessionKeys"
        mode="vertical"
        style="border-right: none"
      >
        <a-menu-item
          v-for="session in chatStore.sessions"
          :key="session.id"
          @click="handleSelectSession(session.id)"
        >
          <message-outlined />
          <span class="session-title">{{ session.title }}</span>
          <delete-outlined
            class="delete-icon"
            @click.stop="handleDeleteSession(session.id)"
          />
        </a-menu-item>
      </a-menu>
    </div>
    
    <div 
      class="resize-handle"
      @mousedown="startResize"
      :class="{ active: isResizing }"
    ></div>
    
    <div class="chat-main">
      <div v-if="!chatStore.currentSessionId" class="empty-state">
        <a-empty description="请选择或创建一个对话" />
      </div>
      <template v-else>
        <div class="message-list" ref="messageListRef">
          <a-spin :spinning="chatStore.loading">
            <div
              v-for="message in chatStore.messages"
              :key="message.id"
              :class="['message-item', message.role]"
            >
              <div class="message-avatar">
                <a-avatar v-if="message.role === 'user'" :size="36">U</a-avatar>
                <a-avatar v-else :size="36" style="background-color: #1890ff">AI</a-avatar>
              </div>
              <div class="message-content">
                <div class="message-role">
                  {{ message.role === 'user' ? '用户' : '智能隐私合规专家' }}
                  <span class="message-time">{{ formatTime(message.timestamp) }}</span>
                </div>
                <div class="message-text" v-html="formatContent(message.content)"></div>
                <div v-if="message.metadata?.tool_calls" class="tool-calls">
                  <div class="tool-calls-header">
                    <tool-outlined /> 工具调用
                  </div>
                  <div
                    v-for="(tc, idx) in message.metadata.tool_calls"
                    :key="idx"
                    class="tool-call-item"
                  >
                    <div class="tool-call-name">
                      <a-tag color="blue">{{ tc.tool }}</a-tag>
                    </div>
                    <div class="tool-call-args">
                      <span class="label">参数:</span>
                      <code>{{ tc.arguments }}</code>
                    </div>
                    <div v-if="tc.result" class="tool-call-result">
                      <span class="label">结果:</span>
                      <pre>{{ JSON.stringify(tc.result, null, 2) }}</pre>
                    </div>
                    <div v-if="tc.error" class="tool-call-error">
                      <span class="label">错误:</span>
                      <span>{{ tc.error }}</span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </a-spin>
        </div>
        <div class="message-input">
          <a-textarea
            v-model:value="inputMessage"
            placeholder="输入消息... (Shift+Enter换行，Enter发送)"
            :auto-size="{ minRows: 3, maxRows: 6 }"
            @press-enter="handleSendMessage"
          />
          <div class="input-actions">
            <a-button type="primary" :loading="chatStore.loading" @click="handleSendMessage">
              发送
            </a-button>
          </div>
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, nextTick, watch } from 'vue'
import { useChatStore } from '@/stores/useChatStore'
import { PlusOutlined, MessageOutlined, DeleteOutlined, ToolOutlined } from '@ant-design/icons-vue'
import { message } from 'ant-design-vue'

const chatStore = useChatStore()
const inputMessage = ref('')
const siderWidth = ref(280)
const isResizing = ref(false)
const containerRef = ref<HTMLElement | null>(null)
const messageListRef = ref<HTMLElement | null>(null)

const selectedSessionKeys = computed(() =>
  chatStore.currentSessionId ? [chatStore.currentSessionId] : []
)

onMounted(() => {
  chatStore.loadSessions()
})

watch(() => chatStore.messages.length, () => {
  nextTick(() => {
    if (messageListRef.value) {
      messageListRef.value.scrollTop = messageListRef.value.scrollHeight
    }
  })
})

function startResize(e: MouseEvent) {
  isResizing.value = true
  const startX = e.clientX
  const startWidth = siderWidth.value
  
  const onMouseMove = (e: MouseEvent) => {
    const newWidth = startWidth + (e.clientX - startX)
    if (newWidth >= 200 && newWidth <= 500) {
      siderWidth.value = newWidth
    }
  }
  
  const onMouseUp = () => {
    isResizing.value = false
    document.removeEventListener('mousemove', onMouseMove)
    document.removeEventListener('mouseup', onMouseUp)
  }
  
  document.addEventListener('mousemove', onMouseMove)
  document.addEventListener('mouseup', onMouseUp)
}

function formatTime(timestamp: number): string {
  const date = new Date(timestamp * 1000)
  return date.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })
}

function formatContent(content: string): string {
  return content
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/\n/g, '<br>')
}

async function handleNewSession() {
  try {
    await chatStore.createSession('新对话')
  } catch (error) {
    message.error('创建对话失败')
  }
}

async function handleSelectSession(sessionId: string) {
  await chatStore.selectSession(sessionId)
}

async function handleDeleteSession(sessionId: string) {
  try {
    await chatStore.deleteSession(sessionId)
    message.success('删除成功')
  } catch (error) {
    message.error('删除失败')
  }
}

async function handleSendMessage(e: KeyboardEvent) {
  if (e.shiftKey) return
  e.preventDefault()

  if (!inputMessage.value.trim()) return

  try {
    await chatStore.sendMessage(inputMessage.value.trim())
    inputMessage.value = ''
  } catch (error) {
    message.error('发送失败')
  }
}
</script>

<style scoped>
.chat-container {
  display: flex;
  height: 100%;
  background: #fff;
}

.session-panel {
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  border-right: 1px solid #e8e8e8;
  background: #fafafa;
  overflow: hidden;
}

.session-header {
  padding: 16px;
  border-bottom: 1px solid #e8e8e8;
}

.session-title {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.resize-handle {
  width: 5px;
  cursor: col-resize;
  background: transparent;
  transition: background 0.2s;
  flex-shrink: 0;
}

.resize-handle:hover,
.resize-handle.active {
  background: #1890ff;
}

.chat-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  overflow: hidden;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
}

.message-list {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
  background: #f5f7fa;
}

.message-item {
  display: flex;
  gap: 12px;
  margin-bottom: 24px;
  animation: fadeIn 0.3s ease;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.message-item.user {
  flex-direction: row-reverse;
}

.message-item.user .message-content {
  align-items: flex-end;
}

.message-avatar {
  flex-shrink: 0;
}

.message-content {
  flex: 1;
  max-width: 75%;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.message-role {
  font-size: 13px;
  color: #666;
  display: flex;
  align-items: center;
  gap: 8px;
}

.message-time {
  font-size: 11px;
  color: #999;
}

.message-text {
  padding: 14px 18px;
  background: #fff;
  border-radius: 12px;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.06);
  line-height: 1.6;
  word-break: break-word;
}

.message-item.user .message-text {
  background: linear-gradient(135deg, #1890ff 0%, #096dd9 100%);
  color: white;
  border-radius: 12px 12px 4px 12px;
}

.message-item.assistant .message-text {
  border-radius: 12px 12px 12px 4px;
}

.tool-calls {
  margin-top: 10px;
  padding: 10px 14px;
  background: #fff;
  border: 1px solid #e8e8e8;
  border-radius: 8px;
}

.tool-calls-header {
  font-weight: 500;
  color: #666;
  margin-bottom: 10px;
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
}

.tool-call-item {
  padding: 10px;
  background: #fafafa;
  border: 1px solid #e8e8e8;
  border-radius: 6px;
  margin-bottom: 8px;
}

.tool-call-item:last-child {
  margin-bottom: 0;
}

.tool-call-name {
  margin-bottom: 6px;
}

.tool-call-args,
.tool-call-result,
.tool-call-error {
  font-size: 12px;
  margin-top: 6px;
}

.tool-call-args .label,
.tool-call-result .label,
.tool-call-error .label {
  color: #8c8c8c;
  margin-right: 4px;
}

.tool-call-args code {
  background: #f0f0f0;
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 11px;
}

.tool-call-result pre {
  margin: 6px 0 0 0;
  padding: 10px;
  background: #f0f0f0;
  border-radius: 6px;
  font-size: 11px;
  max-height: 200px;
  overflow: auto;
}

.tool-call-error {
  color: #ff4d4f;
}

.message-input {
  padding: 16px 20px;
  border-top: 1px solid #e8e8e8;
  background: #fff;
}

.input-actions {
  margin-top: 10px;
  display: flex;
  justify-content: flex-end;
}

.delete-icon {
  margin-left: auto;
  color: #ff4d4f;
  opacity: 0;
  transition: opacity 0.2s;
}

.session-panel .ant-menu-item:hover .delete-icon {
  opacity: 1;
}
</style>