import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Message, ChatSession } from '@/types/message'

export const useChatStore = defineStore('chat', () => {
  const sessions = ref<ChatSession[]>([])
  const currentSessionId = ref<string | null>(null)
  const messages = ref<Message[]>([])
  const loading = ref(false)

  async function loadSessions() {
    try {
      loading.value = true
      sessions.value = await invoke<ChatSession[]>('get_sessions')
    } catch (error) {
      console.error('Failed to load sessions:', error)
    } finally {
      loading.value = false
    }
  }

  async function createSession(title: string) {
    try {
      const session = await invoke<ChatSession>('create_session', { title })
      sessions.value.unshift(session)
      currentSessionId.value = session.id
      messages.value = []
      return session
    } catch (error) {
      console.error('Failed to create session:', error)
      throw error
    }
  }

  async function selectSession(sessionId: string) {
    currentSessionId.value = sessionId
    await loadMessages(sessionId)
  }

  async function loadMessages(sessionId: string) {
    try {
      loading.value = true
      messages.value = await invoke<Message[]>('get_messages', { sessionId })
    } catch (error) {
      console.error('Failed to load messages:', error)
    } finally {
      loading.value = false
    }
  }

  async function sendMessage(content: string) {
    if (!currentSessionId.value) {
      const session = await createSession('新对话')
      currentSessionId.value = session.id
    }

    try {
      loading.value = true
      const message = await invoke<Message>('send_message', {
        sessionId: currentSessionId.value,
        content,
      })
      messages.value.push(message)
      
      // 更新会话标题（使用第一条消息）
      if (messages.value.length === 1) {
        const title = content.length > 20 ? content.substring(0, 20) + '...' : content
        await invoke('update_session_title', { id: currentSessionId.value, title })
        const session = sessions.value.find(s => s.id === currentSessionId.value)
        if (session) {
          session.title = title
        }
      }
    } catch (error) {
      console.error('Failed to send message:', error)
      throw error
    } finally {
      loading.value = false
    }
  }

  async function deleteSession(sessionId: string) {
    try {
      await invoke('delete_session', { id: sessionId })
      sessions.value = sessions.value.filter((s) => s.id !== sessionId)
      if (currentSessionId.value === sessionId) {
        currentSessionId.value = null
        messages.value = []
      }
    } catch (error) {
      console.error('Failed to delete session:', error)
      throw error
    }
  }

  return {
    sessions,
    currentSessionId,
    messages,
    loading,
    loadSessions,
    createSession,
    selectSession,
    loadMessages,
    sendMessage,
    deleteSession,
  }
})