use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub chat_model: ChatModelConfig,
    pub embedding_model: EmbeddingModelConfig,
    pub tools: ToolConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatModelConfig {
    pub provider: ModelProvider,
    pub ollama: OllamaConfig,
    pub openai: OpenAiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingModelConfig {
    pub provider: ModelProvider,
    pub ollama: OllamaEmbeddingConfig,
    pub openai: OpenAiEmbeddingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelProvider {
    Ollama,
    OpenAI,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub base_url: String,
    pub model: String,
    pub temperature: f32,
    pub top_p: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaEmbeddingConfig {
    pub base_url: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiConfig {
    pub base_url: String,
    pub api_key: Option<String>,
    pub model: String,
    pub temperature: f32,
    pub top_p: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiEmbeddingConfig {
    pub base_url: String,
    pub api_key: Option<String>,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    pub require_confirmation: bool,
    pub enabled_tools: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            chat_model: ChatModelConfig {
                provider: ModelProvider::Ollama,
                ollama: OllamaConfig {
                    base_url: "http://localhost:11434".to_string(),
                    model: "llama3.2".to_string(),
                    temperature: 0.7,
                    top_p: 0.9,
                },
                openai: OpenAiConfig {
                    base_url: "https://api.openai.com/v1".to_string(),
                    api_key: None,
                    model: "gpt-3.5-turbo".to_string(),
                    temperature: 0.7,
                    top_p: 0.9,
                },
            },
            embedding_model: EmbeddingModelConfig {
                provider: ModelProvider::Ollama,
                ollama: OllamaEmbeddingConfig {
                    base_url: "http://localhost:11434".to_string(),
                    model: "bge-m3".to_string(),
                },
                openai: OpenAiEmbeddingConfig {
                    base_url: "https://api.openai.com/v1".to_string(),
                    api_key: None,
                    model: "text-embedding-3-small".to_string(),
                },
            },
            tools: ToolConfig {
                require_confirmation: true,
                enabled_tools: vec!["file_read".to_string(), "search_knowledge".to_string()],
            },
        }
    }
}

impl AppConfig {
    pub async fn load(pool: &SqlitePool) -> crate::AppResult<Self> {
        let config_value: Option<String> = sqlx::query_scalar(
            "SELECT value FROM config WHERE key = 'app_config'"
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| crate::AppError::Database(e.to_string()))?;

        match config_value {
            Some(json) => {
                let config: AppConfig = serde_json::from_str(&json)
                    .map_err(|e| crate::AppError::Serialization(e.to_string()))?;
                Ok(config)
            }
            None => {
                let config = Self::default();
                config.save(pool).await?;
                Ok(config)
            }
        }
    }

    pub async fn save(&self, pool: &SqlitePool) -> crate::AppResult<()> {
        let json = serde_json::to_string(self)
            .map_err(|e| crate::AppError::Serialization(e.to_string()))?;

        sqlx::query(
            "INSERT OR REPLACE INTO config (key, value) VALUES ('app_config', ?)"
        )
        .bind(&json)
        .execute(pool)
        .await
        .map_err(|e| crate::AppError::Database(e.to_string()))?;

        Ok(())
    }
}