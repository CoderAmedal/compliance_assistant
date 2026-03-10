<template>
  <div class="knowledge-container">
    <a-layout>
      <a-layout-content>
        <div class="toolbar">
          <a-space>
            <a-button type="primary" @click="handleAddDocument">
              <plus-outlined /> 添加文档
            </a-button>
            <a-button @click="knowledgeStore.loadDocuments">
              <reload-outlined /> 刷新
            </a-button>
          </a-space>
        </div>

        <a-spin :spinning="knowledgeStore.loading">
          <a-list
            :grid="{ gutter: 16, column: 4 }"
            :data-source="knowledgeStore.documents"
          >
            <template #renderItem="{ item }">
              <a-list-item>
                <a-card hoverable>
                  <template #cover>
                    <div class="document-icon">
                      <file-text-outlined style="font-size: 48px" />
                    </div>
                  </template>
                  <a-card-meta :title="item.title">
                    <template #description>
                      <div>{{ formatFileSize(item.fileSize) }}</div>
                      <div>{{ item.fileType.toUpperCase() }}</div>
                    </template>
                  </a-card-meta>
                  <template #actions>
                    <delete-outlined @click="handleDeleteDocument(item.id)" />
                  </template>
                </a-card>
              </a-list-item>
            </template>
          </a-list>
        </a-spin>
      </a-layout-content>
    </a-layout>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useKnowledgeStore } from '@/stores/useKnowledgeStore'
import {
  PlusOutlined,
  ReloadOutlined,
  FileTextOutlined,
  DeleteOutlined,
} from '@ant-design/icons-vue'
import { message } from 'ant-design-vue'

const knowledgeStore = useKnowledgeStore()

onMounted(() => {
  knowledgeStore.loadDocuments()
})

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(2) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(2) + ' MB'
}

async function handleAddDocument() {
  message.info('请使用文件选择器添加文档（功能开发中）')
}

async function handleDeleteDocument(id: string) {
  try {
    await knowledgeStore.deleteDocument(id)
    message.success('删除成功')
  } catch (error) {
    message.error('删除失败')
  }
}
</script>

<style scoped>
.knowledge-container {
  height: 100%;
}

.toolbar {
  margin-bottom: 16px;
}

.document-icon {
  height: 120px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #f5f5f5;
}
</style>