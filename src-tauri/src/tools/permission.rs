use crate::storage::DatabasePool;
use crate::AppResult;
use sqlx::Row;

pub struct PermissionManager {
    pool: DatabasePool,
}

impl PermissionManager {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
    
    pub async fn is_enabled(&self, tool_name: &str) -> bool {
        let result = sqlx::query(
            "SELECT enabled FROM tool_permissions WHERE tool_name = ?",
        )
        .bind(tool_name)
        .fetch_optional(&self.pool)
        .await;
        
        match result {
            Ok(Some(row)) => row.get::<i64, _>("enabled") == 1,
            Ok(None) => true,
            Err(_) => true,
        }
    }
    
    pub async fn requires_confirmation(&self, tool_name: &str) -> bool {
        let result = sqlx::query(
            "SELECT require_confirmation FROM tool_permissions WHERE tool_name = ?",
        )
        .bind(tool_name)
        .fetch_optional(&self.pool)
        .await;
        
        match result {
            Ok(Some(row)) => row.get::<i64, _>("require_confirmation") == 1,
            Ok(None) => true,
            Err(_) => true,
        }
    }
    
    pub async fn set_permission(&self, tool_name: &str, enabled: bool, require_confirmation: bool) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO tool_permissions (tool_name, enabled, require_confirmation)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(tool_name)
        .bind(if enabled { 1 } else { 0 })
        .bind(if require_confirmation { 1 } else { 0 })
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn list_permissions(&self) -> AppResult<Vec<ToolPermission>> {
        let rows = sqlx::query("SELECT tool_name, enabled, require_confirmation FROM tool_permissions")
            .fetch_all(&self.pool)
            .await?;
        
        Ok(rows
            .into_iter()
            .map(|r| ToolPermission {
                tool_name: r.get("tool_name"),
                enabled: r.get::<i64, _>("enabled") == 1,
                require_confirmation: r.get::<i64, _>("require_confirmation") == 1,
            })
            .collect())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolPermission {
    pub tool_name: String,
    pub enabled: bool,
    pub require_confirmation: bool,
}