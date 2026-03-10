import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ToolPermission, ToolResult } from '@/types/tool'

export const useToolStore = defineStore('tool', () => {
  const tools = ref<Record<string, unknown>[]>([])
  const permissions = ref<ToolPermission[]>([])
  const loading = ref(false)

  async function loadTools() {
    try {
      loading.value = true
      tools.value = await invoke('list_tools')
    } catch (error) {
      console.error('Failed to load tools:', error)
    } finally {
      loading.value = false
    }
  }

  async function loadPermissions() {
    try {
      loading.value = true
      permissions.value = await invoke<ToolPermission[]>('get_tool_permissions')
    } catch (error) {
      console.error('Failed to load permissions:', error)
    } finally {
      loading.value = false
    }
  }

  async function executeTool(toolName: string, params: Record<string, unknown>) {
    try {
      loading.value = true
      const result = await invoke<ToolResult>('execute_tool', {
        toolName,
        params,
      })
      return result
    } catch (error) {
      console.error('Failed to execute tool:', error)
      throw error
    } finally {
      loading.value = false
    }
  }

  async function setPermission(
    toolName: string,
    enabled: boolean,
    requireConfirmation: boolean
  ) {
    try {
      await invoke('set_tool_permission', {
        toolName,
        enabled,
        requireConfirmation,
      })
      await loadPermissions()
    } catch (error) {
      console.error('Failed to set permission:', error)
      throw error
    }
  }

  return {
    tools,
    permissions,
    loading,
    loadTools,
    loadPermissions,
    executeTool,
    setPermission,
  }
})