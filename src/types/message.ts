export interface Message {
  id: string
  sessionId: string
  role: 'user' | 'assistant' | 'system'
  content: string
  timestamp: number
  metadata?: Record<string, unknown>
}

export interface ChatSession {
  id: string
  title: string
  createdAt: number
  updatedAt: number
}