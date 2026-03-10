<template>
  <a-config-provider :locale="zhCN">
    <a-layout style="min-height: 100vh">
      <a-layout-sider v-model:collapsed="collapsed" collapsible>
        <div class="logo">
          <h2 v-if="!collapsed">AI 助手</h2>
          <span v-else>AI</span>
        </div>
        <a-menu v-model:selectedKeys="selectedKeys" theme="dark" mode="inline">
          <a-menu-item key="chat">
            <message-outlined />
            <span>聊天</span>
          </a-menu-item>
          <a-menu-item key="knowledge">
            <book-outlined />
            <span>知识库</span>
          </a-menu-item>
          <a-menu-item key="tools">
            <tool-outlined />
            <span>工具</span>
          </a-menu-item>
          <a-menu-item key="settings">
            <setting-outlined />
            <span>设置</span>
          </a-menu-item>
        </a-menu>
      </a-layout-sider>
      <a-layout>
        <a-layout-header style="background: #fff; padding: 0">
          <div class="header">
            <h3>{{ pageTitle }}</h3>
          </div>
        </a-layout-header>
        <a-layout-content style="margin: 16px">
          <div style="padding: 24px; background: #fff; min-height: 360px">
            <router-view />
          </div>
        </a-layout-content>
      </a-layout>
    </a-layout>
  </a-config-provider>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useRouter } from 'vue-router'
import zhCN from 'ant-design-vue/es/locale/zh_CN'
import {
  MessageOutlined,
  BookOutlined,
  ToolOutlined,
  SettingOutlined,
} from '@ant-design/icons-vue'

const router = useRouter()
const collapsed = ref(false)
const selectedKeys = ref(['chat'])

const pageTitle = computed(() => {
  const titles: Record<string, string> = {
    chat: '聊天对话',
    knowledge: '知识库管理',
    tools: '工具管理',
    settings: '系统设置',
  }
  return titles[selectedKeys.value[0]] || 'AI 助手'
})

watch(selectedKeys, (keys) => {
  if (keys[0]) {
    router.push({ name: keys[0] })
  }
}, { immediate: true })
</script>

<style scoped>
.logo {
  height: 32px;
  margin: 16px;
  background: rgba(255, 255, 255, 0.3);
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  font-weight: bold;
}

.header {
  padding: 0 24px;
}

.header h3 {
  margin: 0;
  line-height: 64px;
}
</style>