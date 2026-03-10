<template>
  <div class="chat-container">
    <a-layout>
      <a-layout-sider width="260" theme="light" class="session-sider">
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
            <span>{{ session.title }}</span>
            <delete-outlined
              class="delete-icon"
              @click.stop="handleDeleteSession(session.id)"
            />
          </a-menu-item>
        </a-menu>
      </a-layout-sider>
      <a-layout-content class="chat-content">
        <div v-if="!chatStore.currentSessionId" class="empty-state">
          <a-empty description="请选择或创建一个对话" />
        </div>
        <template v-else>
          <div class="message-list">
            <a-spin :spinning="chatStore.loading">
              <div
                v-for="message in chatStore.messages"
                :key="message.id"
                :class="['message-item', message.role]"
              >
                <div class="message-avatar">
                  <a-avatar v-if="message.role === 'user'" :size="32">U</a-avatar>
                  <a-avatar v-else :size="32" style="background-color: #1890ff">AI</a-avatar>
                </div>
                <div class="message-content">
                  <div class="message-role">{{ message.role === 'user' ? '用户' : '智能隐私合规专家' }}</div>
                  <div class="message-text">{{ message.content }}</div>
                </div>
              </div>
            </a-spin>
          </div>
          <div class="message-input">
            <a-textarea
              v-model:value="inputMessage"
              placeholder="输入消息..."
              :auto-size="{ minRows: 3, maxRows: 6 }"
              @press-enter="handleSendMessage"
            />
            <a-button type="primary" :loading="chatStore.loading" @click="handleSendMessage">
              发送
            </a-button>
          </div>
        </template>
      </a-layout-content>
    </a-layout>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useChatStore } from '@/stores/useChatStore'
import { PlusOutlined, MessageOutlined, DeleteOutlined } from '@ant-design/icons-vue'
import { message } from 'ant-design-vue'

const chatStore = useChatStore()
const inputMessage = ref('')

const selectedSessionKeys = computed(() =>
  chatStore.currentSessionId ? [chatStore.currentSessionId] : []
)

onMounted(() => {
  chatStore.loadSessions()
})

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
  height: calc(100vh - 64px - 48px);
}

.session-sider {
  border-right: 1px solid #f0f0f0;
}

.session-header {
  padding: 16px;
}

.chat-content {
  display: flex;
  flex-direction: column;
  height: 100%;
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
  padding: 16px;
}

.message-item {
  display: flex;
  gap: 12px;
  margin-bottom: 24px;
}

.message-item.user {
  flex-direction: row-reverse;
}

.message-item.user .message-content {
  text-align: right;
}

.message-item.user .message-text {
  background: #1890ff;
  color: white;
  display: inline-block;
  text-align: left;
}

.message-content {
  flex: 1;
  max-width: 70%;
}

.message-role {
  font-size: 12px;
  color: #8c8c8c;
  margin-bottom: 4px;
}

.message-text {
  padding: 12px 16px;
  background: #f5f5f5;
  border-radius: 8px;
  white-space: pre-wrap;
  word-break: break-word;
}

.message-item.user .message-text {
  background: #1890ff;
  color: white;
}

.message-input {
  padding: 16px;
  border-top: 1px solid #f0f0f0;
  display: flex;
  gap: 12px;
}

.delete-icon {
  margin-left: auto;
  color: #ff4d4f;
}

.delete-icon:hover {
  opacity: 0.7;
}
</style>