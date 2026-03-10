import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Document } from '@/types/document'

export const useKnowledgeStore = defineStore('knowledge', () => {
  const documents = ref<Document[]>([])
  const loading = ref(false)

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

  return {
    documents,
    loading,
    loadDocuments,
    addDocument,
    deleteDocument,
    searchKnowledge,
  }
})