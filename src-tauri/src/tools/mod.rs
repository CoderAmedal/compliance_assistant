pub mod registry;
pub mod executor;
pub mod permission;

pub use registry::ToolRegistry;
pub use executor::ToolExecutor;
pub use permission::PermissionManager;

use serde::{Deserialize, Serialize};
use async_trait::async_trait;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
    #[serde(default)]
    pub require_confirmation: bool,
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn definition(&self) -> ToolDefinition;
    async fn execute(&self, params: serde_json::Value) -> crate::AppResult<serde_json::Value>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ToolResult {
    pub fn success(output: serde_json::Value) -> Self {
        Self {
            success: true,
            output,
            error: None,
        }
    }
    
    pub fn error(message: String) -> Self {
        Self {
            success: false,
            output: serde_json::Value::Null,
            error: Some(message),
        }
    }
}