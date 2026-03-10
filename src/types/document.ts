export interface Document {
  id: string
  title: string
  filePath: string
  fileType: string
  fileSize: number
  createdAt: number
  updatedAt: number
}

export interface DocumentChunk {
  id: string
  documentId: string
  chunkIndex: number
  content: string
  createdAt: number
}