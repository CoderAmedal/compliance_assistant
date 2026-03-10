use super::{ToolRegistry, PermissionManager, ToolResult};
use crate::AppResult;

pub struct ToolExecutor {
    registry: ToolRegistry,
    permission_manager: PermissionManager,
}

impl ToolExecutor {
    pub fn new(registry: ToolRegistry, permission_manager: PermissionManager) -> Self {
        Self {
            registry,
            permission_manager,
        }
    }
    
    pub async fn execute(&self, tool_name: &str, params: serde_json::Value) -> AppResult<ToolResult> {
        let tool = self.registry.get(tool_name)
            .ok_or_else(|| crate::AppError::ToolPermission(format!("Tool '{}' not found", tool_name)))?;
        
        if !self.permission_manager.is_enabled(tool_name).await {
            return Ok(ToolResult::error(format!("Tool '{}' is disabled", tool_name)));
        }
        
        let definition = tool.definition();
        if definition.require_confirmation && self.permission_manager.requires_confirmation(tool_name).await {
            return Ok(ToolResult::error(format!("Tool '{}' requires user confirmation", tool_name)));
        }
        
        match tool.execute(params).await {
            Ok(output) => Ok(ToolResult::success(output)),
            Err(e) => Ok(ToolResult::error(e.to_string())),
        }
    }
    
    pub fn registry(&self) -> &ToolRegistry {
        &self.registry
    }
}