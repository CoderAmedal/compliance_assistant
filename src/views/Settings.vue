<template>
  <div class="settings-container">
    <a-card title="聊天模型配置">
      <a-form :label-col="{ span: 6 }" :wrapper-col="{ span: 14 }">
        <a-form-item label="模型提供商">
          <a-select v-model:value="chatModel.provider" style="width: 200px" @change="handleChatProviderChange">
            <a-select-option value="ollama">Ollama</a-select-option>
            <a-select-option value="openai">OpenAI</a-select-option>
          </a-select>
        </a-form-item>

        <a-form-item label="服务地址">
          <a-input v-model:value="chatModel.baseUrl" style="width: 400px" placeholder="http://localhost:11434" />
        </a-form-item>

        <a-form-item label="模型名称">
          <a-input v-model:value="chatModel.model" style="width: 300px" placeholder="llama3.2" />
        </a-form-item>

        <a-form-item v-if="chatModel.provider === 'openai'" label="API Key">
          <a-input-password v-model:value="chatModel.apiKey" style="width: 400px" placeholder="sk-..." />
        </a-form-item>

        <a-form-item label="Temperature">
          <a-row :gutter="16">
            <a-col :span="18">
              <a-slider v-model:value="chatModel.temperature" :min="0" :max="1" :step="0.1" />
            </a-col>
            <a-col :span="6">
              <a-input-number v-model:value="chatModel.temperature" :min="0" :max="1" :step="0.1" style="width: 100%" />
            </a-col>
          </a-row>
        </a-form-item>

        <a-form-item label="Top P">
          <a-row :gutter="16">
            <a-col :span="18">
              <a-slider v-model:value="chatModel.topP" :min="0" :max="1" :step="0.1" />
            </a-col>
            <a-col :span="6">
              <a-input-number v-model:value="chatModel.topP" :min="0" :max="1" :step="0.1" style="width: 100%" />
            </a-col>
          </a-row>
        </a-form-item>

        <a-form-item :wrapper-col="{ offset: 6 }">
          <a-space>
            <a-button type="primary" :loading="savingChat" @click="saveChatModel">保存配置</a-button>
            <a-button @click="resetChatModel">重置</a-button>
          </a-space>
        </a-form-item>
      </a-form>
    </a-card>

    <a-card title="向量化模型配置" style="margin-top: 16px">
      <a-form :label-col="{ span: 6 }" :wrapper-col="{ span: 14 }">
        <a-form-item label="模型提供商">
          <a-select v-model:value="embeddingModel.provider" style="width: 200px" @change="handleEmbeddingProviderChange">
            <a-select-option value="ollama">Ollama</a-select-option>
            <a-select-option value="openai">OpenAI</a-select-option>
          </a-select>
        </a-form-item>

        <a-form-item label="服务地址">
          <a-input v-model:value="embeddingModel.baseUrl" style="width: 400px" placeholder="http://localhost:11434" />
        </a-form-item>

        <a-form-item label="模型名称">
          <a-input v-model:value="embeddingModel.model" style="width: 300px" placeholder="nomic-embed-text" />
        </a-form-item>

        <a-form-item v-if="embeddingModel.provider === 'openai'" label="API Key">
          <a-input-password v-model:value="embeddingModel.apiKey" style="width: 400px" placeholder="sk-..." />
        </a-form-item>

        <a-form-item :wrapper-col="{ offset: 6 }">
          <a-space>
            <a-button type="primary" :loading="savingEmbedding" @click="saveEmbeddingModel">保存配置</a-button>
            <a-button @click="resetEmbeddingModel">重置</a-button>
          </a-space>
        </a-form-item>
      </a-form>
    </a-card>

    <a-card title="数据管理" style="margin-top: 16px">
      <a-space direction="vertical" style="width: 100%">
        <a-button @click="handleExportData">导出数据</a-button>
        <a-button @click="handleImportData">导入数据</a-button>
        <a-button danger @click="handleClearData">清空所有数据</a-button>
      </a-space>
    </a-card>

    <a-card title="关于" style="margin-top: 16px">
      <p><strong>Compliance Assistant</strong></p>
      <p>版本: 0.1.0</p>
      <p>离线AI聊天助手，支持知识库管理和工具调用</p>
    </a-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { message } from 'ant-design-vue'
import { invoke } from '@tauri-apps/api/core'

interface ChatModelConfig {
  provider: string
  baseUrl: string
  model: string
  temperature: number
  topP: number
  apiKey: string
}

interface EmbeddingModelConfig {
  provider: string
  baseUrl: string
  model: string
  apiKey: string
}

const chatModel = ref<ChatModelConfig>({
  provider: 'ollama',
  baseUrl: 'http://localhost:11434',
  model: 'llama3.2',
  temperature: 0.7,
  topP: 0.9,
  apiKey: '',
})

const embeddingModel = ref<EmbeddingModelConfig>({
  provider: 'ollama',
  baseUrl: 'http://localhost:11434',
  model: 'nomic-embed-text',
  apiKey: '',
})

const savingChat = ref(false)
const savingEmbedding = ref(false)

const defaultOllamaUrl = 'http://localhost:11434'
const defaultOpenaiUrl = 'https://api.openai.com/v1'

async function loadChatModelConfig() {
  try {
    const config = await invoke<ChatModelConfig>('get_chat_model_config')
    chatModel.value = {
      provider: config.provider,
      baseUrl: config.baseUrl,
      model: config.model,
      temperature: config.temperature,
      topP: config.topP,
      apiKey: config.apiKey || '',
    }
  } catch (e) {
    console.error('加载聊天模型配置失败:', e)
  }
}

async function loadEmbeddingModelConfig() {
  try {
    const config = await invoke<EmbeddingModelConfig>('get_embedding_model_config')
    embeddingModel.value = {
      provider: config.provider,
      baseUrl: config.baseUrl,
      model: config.model,
      apiKey: config.apiKey || '',
    }
  } catch (e) {
    console.error('加载向量化模型配置失败:', e)
  }
}

function handleChatProviderChange() {
  if (chatModel.value.provider === 'ollama') {
    chatModel.value.baseUrl = defaultOllamaUrl
    chatModel.value.model = 'llama3.2'
  } else {
    chatModel.value.baseUrl = defaultOpenaiUrl
    chatModel.value.model = 'gpt-3.5-turbo'
  }
}

function handleEmbeddingProviderChange() {
  if (embeddingModel.value.provider === 'ollama') {
    embeddingModel.value.baseUrl = defaultOllamaUrl
    embeddingModel.value.model = 'nomic-embed-text'
  } else {
    embeddingModel.value.baseUrl = defaultOpenaiUrl
    embeddingModel.value.model = 'text-embedding-3-small'
  }
}

async function saveChatModel() {
  savingChat.value = true
  try {
    await invoke('update_chat_model_config', {
      request: {
        provider: chatModel.value.provider,
        baseUrl: chatModel.value.baseUrl,
        model: chatModel.value.model,
        temperature: chatModel.value.temperature,
        topP: chatModel.value.topP,
        apiKey: chatModel.value.apiKey || null,
      }
    })
    message.success('聊天模型配置已保存，重启应用后生效')
  } catch (e) {
    message.error('保存失败: ' + String(e))
  } finally {
    savingChat.value = false
  }
}

async function saveEmbeddingModel() {
  savingEmbedding.value = true
  try {
    await invoke('update_embedding_model_config', {
      request: {
        provider: embeddingModel.value.provider,
        baseUrl: embeddingModel.value.baseUrl,
        model: embeddingModel.value.model,
        apiKey: embeddingModel.value.apiKey || null,
      }
    })
    message.success('向量化模型配置已保存，重启应用后生效')
  } catch (e) {
    message.error('保存失败: ' + String(e))
  } finally {
    savingEmbedding.value = false
  }
}

async function resetChatModel() {
  await loadChatModelConfig()
  message.info('已重置为当前配置')
}

async function resetEmbeddingModel() {
  await loadEmbeddingModelConfig()
  message.info('已重置为当前配置')
}

function handleExportData() {
  message.info('导出功能开发中')
}

function handleImportData() {
  message.info('导入功能开发中')
}

function handleClearData() {
  message.warning('清空数据功能开发中')
}

onMounted(() => {
  loadChatModelConfig()
  loadEmbeddingModelConfig()
})
</script>

<style scoped>
.settings-container {
  max-width: 800px;
  margin: 0 auto;
}
</style>