<template>
  <div class="knowledge-container">
    <a-layout>
      <a-layout-content>
        <div class="toolbar">
          <a-space>
            <a-button type="primary" @click="handleUploadClick">
              <upload-outlined /> 上传文档
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
                <a-card hoverable @click="handleViewChunks(item)">
                  <template #cover>
                    <div class="document-icon">
                      <file-text-outlined style="font-size: 48px" />
                    </div>
                  </template>
                  <a-card-meta :title="item.title">
                    <template #description>
                      <div>{{ formatFileSize(item.fileSize) }}</div>
                      <div>{{ item.fileType.toUpperCase() }}</div>
                      <div>
                        <a-tag :color="getStatusColor(item.status)">
                          {{ getStatusText(item.status) }}
                        </a-tag>
                        <span v-if="item.status === 'ready' && item.chunkCount > 0">
                          ({{ item.chunkCount }} 个分块)
                        </span>
                      </div>
                    </template>
                  </a-card-meta>
                  <template #actions>
                    <a-space @click.stop>
                      <a-tooltip v-if="item.status === 'pending'" title="处理文档">
                        <play-circle-outlined @click="handleProcessDocument(item.id)" />
                      </a-tooltip>
                      <a-tooltip v-if="item.status === 'failed'" title="重新处理">
                        <reload-outlined @click="handleProcessDocument(item.id)" />
                      </a-tooltip>
                      <a-tooltip title="删除">
                        <delete-outlined @click="handleDeleteDocument(item.id)" />
                      </a-tooltip>
                    </a-space>
                  </template>
                </a-card>
              </a-list-item>
            </template>
          </a-list>
        </a-spin>
      </a-layout-content>
    </a-layout>

    <!-- 分块详情模态框 -->
    <a-modal
      v-model:open="chunksModalVisible"
      :title="currentDocument?.title || '文档分块'"
      width="80%"
      :footer="null"
    >
      <a-spin :spinning="knowledgeStore.loadingChunks">
        <div v-if="knowledgeStore.currentChunks.length > 0">
          <a-alert
            :message="`共 ${knowledgeStore.currentChunks.length} 个分块`"
            type="info"
            style="margin-bottom: 16px"
          />
          <a-list
            :data-source="knowledgeStore.currentChunks"
            :pagination="{ pageSize: 10 }"
          >
            <template #renderItem="{ item, index }">
              <a-list-item>
                <a-card size="small" style="width: 100%">
                  <template #title>
                    <a-space>
                      <a-tag color="blue">分块 {{ index + 1 }}</a-tag>
                      <span style="font-size: 12px; color: #999">
                        {{ item.content.length }} 字符
                      </span>
                    </a-space>
                  </template>
                  <div class="chunk-content">{{ item.content }}</div>
                </a-card>
              </a-list-item>
            </template>
          </a-list>
        </div>
        <a-empty v-else description="暂无分块数据" />
      </a-spin>
    </a-modal>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { message } from 'ant-design-vue'
import { open } from '@tauri-apps/plugin-dialog'
import { useKnowledgeStore } from '@/stores/useKnowledgeStore'
import type { DocumentStatus, Document } from '@/types/document'
import {
  UploadOutlined,
  ReloadOutlined,
  FileTextOutlined,
  DeleteOutlined,
  PlayCircleOutlined,
} from '@ant-design/icons-vue'

const knowledgeStore = useKnowledgeStore()
const chunksModalVisible = ref(false)
const currentDocument = ref<Document | null>(null)

onMounted(() => {
  knowledgeStore.loadDocuments()
})

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(2) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(2) + ' MB'
}

function getStatusColor(status: DocumentStatus): string {
  switch (status) {
    case 'pending':
      return 'default'
    case 'processing':
      return 'processing'
    case 'ready':
      return 'success'
    case 'failed':
      return 'error'
    default:
      return 'default'
  }
}

function getStatusText(status: DocumentStatus): string {
  switch (status) {
    case 'pending':
      return '待处理'
    case 'processing':
      return '处理中'
    case 'ready':
      return '已就绪'
    case 'failed':
      return '处理失败'
    default:
      return '未知'
  }
}

async function handleUploadClick() {
  try {
    console.log('[DEBUG] Opening file dialog...')
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: 'Documents',
          extensions: ['pdf', 'doc', 'docx', 'txt', 'md', 'html', 'htm', 'xls', 'xlsx'],
        },
      ],
    })
    
    console.log('[DEBUG] Selected file:', selected)
    
    if (selected) {
      try {
        await knowledgeStore.addDocument(selected as string)
        message.success('文档上传成功')
      } catch (error) {
        console.error('[ERROR] Failed to add document:', error)
        message.error('文档上传失败')
      }
    }
  } catch (error) {
    console.error('[ERROR] Failed to open file dialog:', error)
    message.error('打开文件对话框失败: ' + (error as Error).message)
  }
}

async function handleProcessDocument(id: string) {
  try {
    message.loading({ content: '正在处理文档...', key: 'process', duration: 0 })
    await knowledgeStore.processDocument(id)
    message.success({ content: '文档处理成功', key: 'process' })
  } catch (error) {
    message.error({ content: '文档处理失败', key: 'process' })
  }
}

async function handleViewChunks(document: Document) {
  if (document.status !== 'ready' || document.chunkCount === 0) {
    message.warning('文档尚未处理完成或无分块数据')
    return
  }
  
  currentDocument.value = document
  chunksModalVisible.value = true
  
  try {
    await knowledgeStore.getDocumentChunks(document.id)
  } catch (error) {
    message.error('加载分块失败')
    chunksModalVisible.value = false
  }
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

.chunk-content {
  max-height: 300px;
  overflow-y: auto;
  padding: 8px;
  background: #fafafa;
  border-radius: 4px;
  font-size: 13px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-all;
}
</style>