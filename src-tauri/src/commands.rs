use compliance_assistant::{
    AppState,
    SessionResponse,
    MessageResponse,
    DocumentResponse,
    ChatMessage,
    MessageRole,
    Document,
    DocumentChunk,
    ModelProvider,
    LlmClient,
};

use tauri::State;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatModelConfigResponse {
    pub provider: String,
    #[serde(rename = "baseUrl")]
    pub base_url: String,
    pub model: String,
    pub temperature: f32,
    #[serde(rename = "topP")]
    pub top_p: f32,
    #[serde(rename = "apiKey")]
    pub api_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingModelConfigResponse {
    pub provider: String,
    #[serde(rename = "baseUrl")]
    pub base_url: String,
    pub model: String,
    #[serde(rename = "apiKey")]
    pub api_key: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateChatModelConfigRequest {
    pub provider: String,
    #[serde(rename = "baseUrl")]
    pub base_url: String,
    pub model: String,
    pub temperature: f32,
    #[serde(rename = "topP")]
    pub top_p: f32,
    #[serde(rename = "apiKey")]
    pub api_key: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEmbeddingModelConfigRequest {
    pub provider: String,
    #[serde(rename = "baseUrl")]
    pub base_url: String,
    pub model: String,
    #[serde(rename = "apiKey")]
    pub api_key: Option<String>,
}

#[tauri::command]
pub async fn get_sessions(
    state: State<'_, AppState>,
) -> Result<Vec<SessionResponse>, String> {
    let session_manager = compliance_assistant::chat::Session::new(state.db_pool.clone());
    session_manager
        .list()
        .await
        .map(|sessions| sessions.into_iter().map(SessionResponse::from).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_session(
    title: String,
    state: State<'_, AppState>,
) -> Result<SessionResponse, String> {
    let session_manager = compliance_assistant::chat::Session::new(state.db_pool.clone());
    session_manager
        .create(title)
        .await
        .map(SessionResponse::from)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_session(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let session_manager = compliance_assistant::chat::Session::new(state.db_pool.clone());
    session_manager
        .delete(&id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_session_title(
    id: String,
    title: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let session_manager = compliance_assistant::chat::Session::new(state.db_pool.clone());
    session_manager
        .update_title(&id, &title)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_messages(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<MessageResponse>, String> {
    let session_manager = compliance_assistant::chat::Session::new(state.db_pool.clone());
    session_manager
        .get_messages(&session_id)
        .await
        .map(|messages| {
            messages
                .into_iter()
                .map(|m| MessageResponse {
                    id: m.id,
                    session_id: m.session_id,
                    role: m.role.as_str().to_string(),
                    content: m.content,
                    timestamp: m.timestamp,
                    metadata: m.metadata,
                })
                .collect()
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn send_message(
    session_id: String,
    content: String,
    state: State<'_, AppState>,
) -> Result<MessageResponse, String> {
    let session_manager = compliance_assistant::chat::Session::new(state.db_pool.clone());
    
    let user_message = ChatMessage::new(
        session_id.clone(),
        MessageRole::User,
        content.clone(),
    );
    
    session_manager
        .add_message(user_message.clone())
        .await
        .map_err(|e| e.to_string())?;
    
    let messages = session_manager
        .get_messages(&session_id)
        .await
        .map_err(|e| e.to_string())?;
    
    let llm_messages: Vec<compliance_assistant::llm::ChatMessage> = messages
        .iter()
        .map(|m| match m.role {
            MessageRole::User => compliance_assistant::llm::ChatMessage::user(m.content.clone()),
            MessageRole::Assistant => compliance_assistant::llm::ChatMessage::assistant(m.content.clone()),
            MessageRole::System => compliance_assistant::llm::ChatMessage::system(m.content.clone()),
        })
        .collect();
    
    let config = state.config.read().await;
    let response = match &config.chat_model.provider {
        ModelProvider::Ollama => {
            let client = compliance_assistant::OllamaClient::new(
                config.chat_model.ollama.base_url.clone(),
                config.chat_model.ollama.model.clone(),
                config.chat_model.ollama.temperature,
                config.chat_model.ollama.top_p,
            );
            client.chat(llm_messages).await.map_err(|e| e.to_string())?
        }
        ModelProvider::OpenAI => {
            let client = compliance_assistant::OpenAIClient::new(
                config.chat_model.openai.base_url.clone(),
                config.chat_model.openai.api_key.clone(),
                config.chat_model.openai.model.clone(),
                config.chat_model.openai.temperature,
                config.chat_model.openai.top_p,
            );
            client.chat(llm_messages).await.map_err(|e| e.to_string())?
        }
    };
    drop(config);
    
    let assistant_message = ChatMessage::new(
        session_id.clone(),
        MessageRole::Assistant,
        response,
    );
    
    session_manager
        .add_message(assistant_message.clone())
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(MessageResponse {
        id: assistant_message.id,
        session_id: assistant_message.session_id,
        role: assistant_message.role.as_str().to_string(),
        content: assistant_message.content,
        timestamp: assistant_message.timestamp,
        metadata: assistant_message.metadata,
    })
}

#[tauri::command]
pub async fn list_documents(
    state: State<'_, AppState>,
) -> Result<Vec<DocumentResponse>, String> {
    let retriever = compliance_assistant::knowledge::Retriever::new(state.db_pool.clone());
    retriever
        .list_documents()
        .await
        .map(|docs| docs.into_iter().map(DocumentResponse::from).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_document(
    file_path: String,
    state: State<'_, AppState>,
) -> Result<DocumentResponse, String> {
    let path = std::path::Path::new(&file_path);
    
    let parsed = compliance_assistant::knowledge::DocumentParser::parse(path)
        .map_err(|e| e.to_string())?;
    
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown")
        .to_string();
    
    let file_type = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_string();
    
    let file_size = std::fs::metadata(path)
        .map(|m| m.len())
        .unwrap_or(0);
    
    let document = Document::new(file_name, file_path, file_type, file_size);
    
    let chunks: Vec<DocumentChunk> = parsed
        .content
        .split("\n\n")
        .enumerate()
        .map(|(i, chunk)| DocumentChunk::new(document.id.clone(), i, chunk.to_string()))
        .collect();
    
    let retriever = compliance_assistant::knowledge::Retriever::new(state.db_pool.clone());
    retriever
        .add_document(document.clone(), chunks)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(DocumentResponse::from(document))
}

#[tauri::command]
pub async fn delete_document(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let retriever = compliance_assistant::knowledge::Retriever::new(state.db_pool.clone());
    retriever
        .delete_document(&id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_knowledge(
    query: String,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let retriever = compliance_assistant::knowledge::Retriever::new(state.db_pool.clone());
    retriever
        .search(&query, limit.unwrap_or(5))
        .await
        .map(|chunks| {
            chunks
                .into_iter()
                .map(|c| serde_json::json!({
                    "id": c.id,
                    "documentId": c.document_id,
                    "chunkIndex": c.chunk_index,
                    "content": c.content,
                    "createdAt": c.created_at
                }))
                .collect()
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_tools(
    _state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    Ok(vec![
        serde_json::json!({
            "name": "file_read",
            "description": "Read file content",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {"type": "string"}
                }
            }
        }),
        serde_json::json!({
            "name": "search_knowledge",
            "description": "Search knowledge base",
            "parameters": {
                "type": "object",
                "properties": {
                    "query": {"type": "string"}
                }
            }
        }),
    ])
}

#[tauri::command]
pub async fn execute_tool(
    tool_name: String,
    params: serde_json::Value,
    state: State<'_, AppState>,
) -> Result<compliance_assistant::tools::ToolResult, String> {
    let permission_manager = compliance_assistant::tools::PermissionManager::new(state.db_pool.clone());
    let registry = compliance_assistant::tools::ToolRegistry::new();
    let executor = compliance_assistant::tools::ToolExecutor::new(registry, permission_manager);
    
    executor
        .execute(&tool_name, params)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_tool_permissions(
    state: State<'_, AppState>,
) -> Result<Vec<compliance_assistant::tools::permission::ToolPermission>, String> {
    let permission_manager = compliance_assistant::tools::PermissionManager::new(state.db_pool.clone());
    permission_manager
        .list_permissions()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_tool_permission(
    tool_name: String,
    enabled: bool,
    require_confirmation: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let permission_manager = compliance_assistant::tools::PermissionManager::new(state.db_pool.clone());
    permission_manager
        .set_permission(&tool_name, enabled, require_confirmation)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_chat_model_config(
    state: State<'_, AppState>,
) -> Result<ChatModelConfigResponse, String> {
    let config = state.config.read().await;
    let (provider, base_url, model, temperature, top_p, api_key) = match &config.chat_model.provider {
        ModelProvider::Ollama => {
            ("ollama".to_string(), 
             config.chat_model.ollama.base_url.clone(),
             config.chat_model.ollama.model.clone(),
             config.chat_model.ollama.temperature,
             config.chat_model.ollama.top_p,
             None)
        }
        ModelProvider::OpenAI => {
            ("openai".to_string(),
             config.chat_model.openai.base_url.clone(),
             config.chat_model.openai.model.clone(),
             config.chat_model.openai.temperature,
             config.chat_model.openai.top_p,
             config.chat_model.openai.api_key.clone())
        }
    };
    
    Ok(ChatModelConfigResponse {
        provider,
        base_url,
        model,
        temperature,
        top_p,
        api_key,
    })
}

#[tauri::command]
pub async fn get_embedding_model_config(
    state: State<'_, AppState>,
) -> Result<EmbeddingModelConfigResponse, String> {
    let config = state.config.read().await;
    let (provider, base_url, model, api_key) = match &config.embedding_model.provider {
        ModelProvider::Ollama => {
            ("ollama".to_string(), 
             config.embedding_model.ollama.base_url.clone(),
             config.embedding_model.ollama.model.clone(),
             None)
        }
        ModelProvider::OpenAI => {
            ("openai".to_string(),
             config.embedding_model.openai.base_url.clone(),
             config.embedding_model.openai.model.clone(),
             config.embedding_model.openai.api_key.clone())
        }
    };
    
    Ok(EmbeddingModelConfigResponse {
        provider,
        base_url,
        model,
        api_key,
    })
}

#[tauri::command]
pub async fn update_chat_model_config(
    request: UpdateChatModelConfigRequest,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut config = state.config.write().await;
    
    config.chat_model.provider = match request.provider.as_str() {
        "openai" => ModelProvider::OpenAI,
        _ => ModelProvider::Ollama,
    };
    
    match config.chat_model.provider {
        ModelProvider::Ollama => {
            config.chat_model.ollama.base_url = request.base_url;
            config.chat_model.ollama.model = request.model;
            config.chat_model.ollama.temperature = request.temperature;
            config.chat_model.ollama.top_p = request.top_p;
        }
        ModelProvider::OpenAI => {
            config.chat_model.openai.base_url = request.base_url;
            config.chat_model.openai.model = request.model;
            config.chat_model.openai.temperature = request.temperature;
            config.chat_model.openai.top_p = request.top_p;
            config.chat_model.openai.api_key = request.api_key;
        }
    }
    
    let pool = state.db_pool.clone();
    config.save(&pool).await.map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub async fn update_embedding_model_config(
    request: UpdateEmbeddingModelConfigRequest,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut config = state.config.write().await;
    
    config.embedding_model.provider = match request.provider.as_str() {
        "openai" => ModelProvider::OpenAI,
        _ => ModelProvider::Ollama,
    };
    
    match config.embedding_model.provider {
        ModelProvider::Ollama => {
            config.embedding_model.ollama.base_url = request.base_url;
            config.embedding_model.ollama.model = request.model;
        }
        ModelProvider::OpenAI => {
            config.embedding_model.openai.base_url = request.base_url;
            config.embedding_model.openai.model = request.model;
            config.embedding_model.openai.api_key = request.api_key;
        }
    }
    
    let pool = state.db_pool.clone();
    config.save(&pool).await.map_err(|e| e.to_string())?;
    
    Ok(())
}