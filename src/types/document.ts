export interface Document {
  id: string
  title: string
  filePath: string
  fileType: string
  fileSize: number
  status: DocumentStatus
  chunkCount: number
  createdAt: number
  updatedAt: number
}

export type DocumentStatus = 'pending' | 'processing' | 'ready' | 'failed'

export interface DocumentChunk {
  id: string
  documentId: string
  chunkIndex: number
  content: string
  createdAt: number
}