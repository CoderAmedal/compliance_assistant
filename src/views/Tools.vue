<template>
  <div class="tools-container">
    <h2>工具管理</h2>
    <p class="description">管理AI可调用的工具，控制启用状态和权限设置</p>
    
    <a-table 
      :columns="columns" 
      :data-source="toolStore.tools" 
      :loading="toolStore.loading"
      :pagination="false"
    >
      <template #bodyCell="{ column, record }">
        <template v-if="column.key === 'name'">
          <a-tag color="blue">{{ record.name }}</a-tag>
        </template>
        <template v-if="column.key === 'parameters'">
          <a-button size="small" @click="showParams(record)">查看参数</a-button>
        </template>
        <template v-if="column.key === 'enabled'">
          <a-switch
            :checked="getToolPermission(record.name)?.enabled ?? true"
            @change="(checked: boolean) => handlePermissionChange(record.name, checked, getToolPermission(record.name)?.requireConfirmation ?? false)"
          />
        </template>
        <template v-if="column.key === 'requireConfirmation'">
          <a-switch
            :checked="getToolPermission(record.name)?.requireConfirmation ?? false"
            :disabled="!(getToolPermission(record.name)?.enabled ?? true)"
            @change="(checked: boolean) => handlePermissionChange(record.name, getToolPermission(record.name)?.enabled ?? true, checked)"
          />
        </template>
      </template>
    </a-table>

    <a-modal
      v-model:open="paramsVisible"
      :title="currentTool?.name"
      :footer="null"
      width="600px"
    >
      <div class="params-content">
        <p><strong>描述：</strong>{{ currentTool?.description }}</p>
        <p><strong>参数定义：</strong></p>
        <pre>{{ JSON.stringify(currentTool?.parameters, null, 2) }}</pre>
      </div>
    </a-modal>

    <a-divider />

    <h3>测试工具</h3>
    <a-form layout="vertical" style="max-width: 600px">
      <a-form-item label="选择工具">
        <a-select v-model:value="testToolName" placeholder="选择要测试的工具">
          <a-select-option v-for="tool in toolStore.tools" :key="tool.name" :value="tool.name">
            {{ tool.name }}
          </a-select-option>
        </a-select>
      </a-form-item>
      <a-form-item label="参数 (JSON格式)">
        <a-textarea
          v-model:value="testParams"
          placeholder='{"path": "/path/to/file"}'
          :auto-size="{ minRows: 3, maxRows: 6 }"
        />
      </a-form-item>
      <a-form-item>
        <a-button type="primary" :loading="testing" @click="handleTestTool">
          执行测试
        </a-button>
      </a-form-item>
      <a-form-item v-if="testResult" label="执行结果">
        <pre class="result-pre">{{ JSON.stringify(testResult, null, 2) }}</pre>
      </a-form-item>
    </a-form>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useToolStore } from '@/stores/useToolStore'
import { message } from 'ant-design-vue'
import type { ToolPermission, ToolResult } from '@/types/tool'

const toolStore = useToolStore()

const paramsVisible = ref(false)
const currentTool = ref<Record<string, unknown> | null>(null)
const testToolName = ref('')
const testParams = ref('{}')
const testResult = ref<ToolResult | null>(null)
const testing = ref(false)

const columns = [
  {
    title: '工具名称',
    key: 'name',
    dataIndex: 'name',
    width: 150,
  },
  {
    title: '描述',
    dataIndex: 'description',
  },
  {
    title: '参数',
    key: 'parameters',
    width: 100,
  },
  {
    title: '启用',
    key: 'enabled',
    width: 80,
  },
  {
    title: '需要确认',
    key: 'requireConfirmation',
    width: 100,
  },
]

onMounted(() => {
  toolStore.loadTools()
  toolStore.loadPermissions()
})

function getToolPermission(toolName: string): ToolPermission | undefined {
  return toolStore.permissions.find((p) => p.toolName === toolName)
}

function showParams(tool: Record<string, unknown>) {
  currentTool.value = tool
  paramsVisible.value = true
}

async function handlePermissionChange(
  toolName: string,
  enabled: boolean,
  requireConfirmation: boolean
) {
  try {
    await toolStore.setPermission(toolName, enabled, requireConfirmation)
    message.success('权限更新成功')
  } catch (error) {
    message.error('权限更新失败')
  }
}

async function handleTestTool() {
  if (!testToolName.value) {
    message.warning('请选择工具')
    return
  }

  let params: Record<string, unknown>
  try {
    params = JSON.parse(testParams.value)
  } catch {
    message.error('参数JSON格式错误')
    return
  }

  try {
    testing.value = true
    testResult.value = await toolStore.executeTool(testToolName.value, params)
  } catch (error) {
    message.error('执行失败')
    console.error(error)
  } finally {
    testing.value = false
  }
}
</script>

<style scoped>
.tools-container {
  padding: 16px;
}

.description {
  color: #666;
  margin-bottom: 16px;
}

.params-content {
  padding: 8px 0;
}

.params-content pre {
  background: #f5f5f5;
  padding: 12px;
  border-radius: 4px;
  overflow: auto;
  max-height: 300px;
}

.result-pre {
  background: #f5f5f5;
  padding: 12px;
  border-radius: 4px;
  overflow: auto;
  max-height: 400px;
  font-size: 12px;
}
</style>