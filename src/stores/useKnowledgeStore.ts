import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Document, DocumentStatus, DocumentChunk } from '@/types/document'

export const useKnowledgeStore = defineStore('knowledge', () => {
  const documents = ref<Document[]>([])
  const loading = ref(false)
  const processing = ref<string | null>(null)
  const currentChunks = ref<DocumentChunk[]>([])
  const loadingChunks = ref(false)

  async function loadDocuments() {
    try {
      loading.value = true
      documents.value = await invoke<Document[]>('list_documents')
    } catch (error) {
      console.error('Failed to load documents:', error)
    } finally {
      loading.value = false
    }
  }

  async function addDocument(filePath: string) {
    try {
      loading.value = true
      const document = await invoke<Document>('add_document', { filePath })
      documents.value.unshift(document)
      return document
    } catch (error) {
      console.error('Failed to add document:', error)
      throw error
    } finally {
      loading.value = false
    }
  }

  async function processDocument(id: string) {
    try {
      processing.value = id
      const updatedDoc = await invoke<Document>('process_document', { id })
      
      const index = documents.value.findIndex(d => d.id === id)
      if (index !== -1) {
        documents.value[index] = updatedDoc
      }
      
      return updatedDoc
    } catch (error) {
      console.error('Failed to process document:', error)
      throw error
    } finally {
      processing.value = null
    }
  }

  async function getDocument(id: string) {
    try {
      const document = await invoke<Document>('get_document', { id })
      return document
    } catch (error) {
      console.error('Failed to get document:', error)
      throw error
    }
  }

  async function getDocumentChunks(id: string) {
    try {
      loadingChunks.value = true
      const chunks = await invoke<DocumentChunk[]>('get_document_chunks', { documentId: id })
      currentChunks.value = chunks
      return chunks
    } catch (error) {
      console.error('Failed to get document chunks:', error)
      throw error
    } finally {
      loadingChunks.value = false
    }
  }

  async function deleteDocument(id: string) {
    try {
      await invoke('delete_document', { id })
      documents.value = documents.value.filter((d) => d.id !== id)
    } catch (error) {
      console.error('Failed to delete document:', error)
      throw error
    }
  }

  async function searchKnowledge(query: string, limit?: number) {
    try {
      loading.value = true
      const results = await invoke('search_knowledge', { query, limit })
      return results
    } catch (error) {
      console.error('Failed to search knowledge:', error)
      throw error
    } finally {
      loading.value = false
    }
  }

  function getDocumentStatus(id: string): DocumentStatus | undefined {
    return documents.value.find(d => d.id === id)?.status
  }

  function isProcessing(id: string): boolean {
    return processing.value === id
  }

  return {
    documents,
    loading,
    processing,
    currentChunks,
    loadingChunks,
    loadDocuments,
    addDocument,
    processDocument,
    getDocument,
    getDocumentChunks,
    deleteDocument,
    searchKnowledge,
    getDocumentStatus,
    isProcessing,
  }
})