export interface Tool {
  name: string
  description: string
  parameters: Record<string, unknown>
}

export interface ToolPermission {
  toolName: string
  enabled: boolean
  requireConfirmation: boolean
}

export interface ToolResult {
  success: boolean
  output: Record<string, unknown>
  error?: string
}