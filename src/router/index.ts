import { createRouter, createWebHistory } from 'vue-router'
import ChatView from '@/views/Chat.vue'
import KnowledgeView from '@/views/Knowledge.vue'
import ToolsView from '@/views/Tools.vue'
import SettingsView from '@/views/Settings.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'chat',
      component: ChatView,
    },
    {
      path: '/knowledge',
      name: 'knowledge',
      component: KnowledgeView,
    },
    {
      path: '/tools',
      name: 'tools',
      component: ToolsView,
    },
    {
      path: '/settings',
      name: 'settings',
      component: SettingsView,
    },
  ],
})

export default router