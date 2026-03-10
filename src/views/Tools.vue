<template>
  <div class="tools-container">
    <a-table :columns="columns" :data-source="toolStore.tools" :loading="toolStore.loading">
      <template #bodyCell="{ column, record }">
        <template v-if="column.key === 'name'">
          <a-tag color="blue">{{ record.name }}</a-tag>
        </template>
        <template v-if="column.key === 'action'">
          <a-button size="small" @click="handleViewTool(record)">查看详情</a-button>
        </template>
      </template>
    </a-table>

    <a-divider />

    <h3>权限管理</h3>
    <a-table
      :columns="permissionColumns"
      :data-source="toolStore.permissions"
      :loading="toolStore.loading"
    >
      <template #bodyCell="{ column, record }">
        <template v-if="column.key === 'enabled'">
          <a-switch
            :checked="record.enabled"
            @change="(checked: boolean) => handlePermissionChange(record.toolName, checked, record.requireConfirmation)"
          />
        </template>
        <template v-if="column.key === 'requireConfirmation'">
          <a-switch
            :checked="record.requireConfirmation"
            @change="(checked: boolean) => handlePermissionChange(record.toolName, record.enabled, checked)"
          />
        </template>
      </template>
    </a-table>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useToolStore } from '@/stores/useToolStore'
import { message } from 'ant-design-vue'

const toolStore = useToolStore()

const columns = [
  {
    title: '工具名称',
    key: 'name',
    dataIndex: 'name',
  },
  {
    title: '描述',
    dataIndex: 'description',
  },
  {
    title: '操作',
    key: 'action',
  },
]

const permissionColumns = [
  {
    title: '工具名称',
    dataIndex: 'toolName',
  },
  {
    title: '是否启用',
    key: 'enabled',
  },
  {
    title: '需要确认',
    key: 'requireConfirmation',
  },
]

onMounted(() => {
  toolStore.loadTools()
  toolStore.loadPermissions()
})

function handleViewTool(tool: Record<string, unknown>) {
  message.info(`查看工具: ${tool.name}`)
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
</script>

<style scoped>
.tools-container {
  padding: 16px;
}
</style>