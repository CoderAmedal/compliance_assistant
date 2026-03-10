mod error;
pub mod config;
pub mod chat;
pub mod knowledge;
pub mod tools;
mod skills;
pub mod llm;
mod storage;

pub use error::{AppError, AppResult};
pub use config::{AppConfig, ModelProvider};
pub use chat::{ChatSession, ChatMessage, MessageRole};
pub use knowledge::{Document, DocumentChunk, DocumentStatus};
pub use tools::{ToolRegistry, ToolExecutor, PermissionManager};
pub use skills::{Skill, SkillLoader, SkillRunner};
pub use llm::{LlmClient, OllamaClient, OpenAIClient, EmbeddingService};

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

pub struct AppState {
    pub config: Arc<RwLock<AppConfig>>,
    pub db_pool: storage::DatabasePool,
    pub llm_client: Arc<dyn llm::LlmClient>,
}

pub async fn init_app() -> AppResult<AppState> {
    let exe_path = std::env::current_exe()
        .map_err(|e| AppError::Config(format!("Cannot get executable path: {}", e)))?;
    
    let data_dir = exe_path
        .parent()
        .ok_or_else(|| AppError::Config("Cannot get executable directory".to_string()))?
        .join("data");
    
    std::fs::create_dir_all(&data_dir)?;
    
    let database_path = data_dir.join("data.db");
    let database_url = format!("sqlite:{}", database_path.display());
    let db_pool = storage::init_database(&database_url).await?;
    
    let config = AppConfig::load(&db_pool).await?;
    
    let llm_client: Arc<dyn llm::LlmClient> = match config.chat_model.provider {
        config::ModelProvider::Ollama => Arc::new(OllamaClient::new(
            config.chat_model.ollama.base_url.clone(),
            config.chat_model.ollama.model.clone(),
            config.chat_model.ollama.temperature,
            config.chat_model.ollama.top_p,
        )),
        config::ModelProvider::OpenAI => Arc::new(OpenAIClient::new(
            config.chat_model.openai.base_url.clone(),
            config.chat_model.openai.api_key.clone(),
            config.chat_model.openai.model.clone(),
            config.chat_model.openai.temperature,
            config.chat_model.openai.top_p,
        )),
    };
    
    Ok(AppState {
        config: Arc::new(RwLock::new(config)),
        db_pool,
        llm_client,
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionResponse {
    pub id: String,
    pub title: String,
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
}

impl From<ChatSession> for SessionResponse {
    fn from(session: ChatSession) -> Self {
        Self {
            id: session.id,
            title: session.title,
            created_at: session.created_at,
            updated_at: session.updated_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageResponse {
    pub id: String,
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub timestamp: i64,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentResponse {
    pub id: String,
    pub title: String,
    #[serde(rename = "filePath")]
    pub file_path: String,
    #[serde(rename = "fileType")]
    pub file_type: String,
    #[serde(rename = "fileSize")]
    pub file_size: u64,
    pub status: String,
    #[serde(rename = "chunkCount")]
    pub chunk_count: usize,
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
}

impl From<Document> for DocumentResponse {
    fn from(doc: Document) -> Self {
        Self {
            id: doc.id,
            title: doc.title,
            file_path: doc.file_path,
            file_type: doc.file_type,
            file_size: doc.file_size,
            status: match doc.status {
                DocumentStatus::Pending => "pending".to_string(),
                DocumentStatus::Processing => "processing".to_string(),
                DocumentStatus::Ready => "ready".to_string(),
                DocumentStatus::Failed => "failed".to_string(),
            },
            chunk_count: doc.chunk_count,
            created_at: doc.created_at,
            updated_at: doc.updated_at,
        }
    }
}