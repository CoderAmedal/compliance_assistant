use compliance_assistant::{
    AppState,
    SessionResponse,
    MessageResponse,
    DocumentResponse,
    ChatMessage,
    MessageRole,
    Document,
    ModelProvider,
    LlmClient,
    EmbeddingService,
    ToolRegistry,
    ToolExecutor,
    PermissionManager,
    tools::{FileReadTool, KnowledgeSearchTool, CalculatorTool},
    AppResult,
    AppError,
};

use tauri::State;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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
    
    let mut llm_messages: Vec<compliance_assistant::llm::ChatMessage> = vec![];
    
    let config = state.config.read().await;
    let embedding_service = EmbeddingService::new(config.embedding_model.clone());
    let retriever = compliance_assistant::knowledge::Retriever::new(state.db_pool.clone())
        .with_embedding(embedding_service.clone());
    
    let knowledge_chunks = match retriever.search(&content, 5).await {
        Ok(chunks) => {
            eprintln!("[DEBUG] Found {} knowledge chunks for query: {}", chunks.len(), content);
            chunks
        },
        Err(e) => {
            eprintln!("[ERROR] Failed to search knowledge: {}", e);
            vec![]
        }
    };
    
    let mut tool_results_metadata: Vec<serde_json::Value> = vec![];
    
    if !knowledge_chunks.is_empty() {
        let knowledge_context: String = knowledge_chunks
            .iter()
            .map(|c| format!("- {}", c.content))
            .collect::<Vec<_>>()
            .join("\n\n");
        
        eprintln!("[DEBUG] Knowledge context length: {} chars", knowledge_context.len());
        
        let system_prompt = format!(
            "你是一个智能助手，可以参考以下知识库内容来回答用户问题。如果知识库内容与问题相关，请优先基于知识库内容回答；如果不相关或知识库中没有相关信息，请基于你的知识回答。\n\n知识库内容：\n{}\n\n请根据上述知识库内容回答用户的问题。",
            knowledge_context
        );
        
        llm_messages.push(compliance_assistant::llm::ChatMessage::system(system_prompt));
    } else {
        llm_messages.push(compliance_assistant::llm::ChatMessage::system(
            "你是一个智能助手，可以帮助用户回答问题。如果需要查询知识库，请使用 search_knowledge 工具。".to_string()
        ));
    }
    
    let messages = session_manager
        .get_messages(&session_id)
        .await
        .map_err(|e| e.to_string())?;
    
    let chat_messages: Vec<compliance_assistant::llm::ChatMessage> = messages
        .iter()
        .map(|m| match m.role {
            MessageRole::User => compliance_assistant::llm::ChatMessage::user(m.content.clone()),
            MessageRole::Assistant => compliance_assistant::llm::ChatMessage::assistant(m.content.clone()),
            MessageRole::System => compliance_assistant::llm::ChatMessage::system(m.content.clone()),
        })
        .collect();
    
    llm_messages.extend(chat_messages);
    
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(FileReadTool));
    registry.register(Arc::new(KnowledgeSearchTool::new(state.db_pool.clone(), embedding_service)));
    registry.register(Arc::new(CalculatorTool));
    
    let permission_manager = PermissionManager::new(state.db_pool.clone());
    let executor = ToolExecutor::new(registry, permission_manager);
    
    let tools: Vec<compliance_assistant::llm::ToolDefinition> = executor
        .registry()
        .list()
        .into_iter()
        .map(|t| compliance_assistant::llm::ToolDefinition::from_tool(&t))
        .collect();
    
    let response = match &config.chat_model.provider {
        ModelProvider::Ollama => {
            let client = compliance_assistant::OllamaClient::new(
                config.chat_model.ollama.base_url.clone(),
                config.chat_model.ollama.model.clone(),
                config.chat_model.ollama.temperature,
                config.chat_model.ollama.top_p,
            );
            execute_with_tools(&client, llm_messages, tools, &executor, &mut tool_results_metadata).await
                .map_err(|e| e.to_string())?
        }
        ModelProvider::OpenAI => {
            let client = compliance_assistant::OpenAIClient::new(
                config.chat_model.openai.base_url.clone(),
                config.chat_model.openai.api_key.clone(),
                config.chat_model.openai.model.clone(),
                config.chat_model.openai.temperature,
                config.chat_model.openai.top_p,
            );
            execute_with_tools(&client, llm_messages, tools, &executor, &mut tool_results_metadata).await
                .map_err(|e| e.to_string())?
        }
    };
    drop(config);
    
    let metadata = if !tool_results_metadata.is_empty() {
        Some(serde_json::json!({ "tool_calls": tool_results_metadata }))
    } else {
        None
    };
    
    let assistant_message = ChatMessage::new_with_metadata(
        session_id.clone(),
        MessageRole::Assistant,
        response,
        metadata,
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

async fn execute_with_tools<C: LlmClient>(
    client: &C,
    mut messages: Vec<compliance_assistant::llm::ChatMessage>,
    tools: Vec<compliance_assistant::llm::ToolDefinition>,
    executor: &ToolExecutor,
    tool_results_metadata: &mut Vec<serde_json::Value>,
) -> AppResult<String> {
    let max_iterations = 10;
    let mut iteration = 0;
    
    loop {
        iteration += 1;
        if iteration > max_iterations {
            return Err(AppError::LlmRequest("Too many tool call iterations".to_string()));
        }
        
        let response = client.chat_with_tools(messages.clone(), tools.clone()).await?;
        
        if !response.has_tool_calls() {
            return Ok(response.content);
        }
        
        let tool_calls = response.tool_calls.unwrap();
        eprintln!("[DEBUG] LLM requested {} tool calls", tool_calls.len());
        
        let mut tool_results = Vec::new();
        
        for tool_call in &tool_calls {
            let tool_name = &tool_call.function.name;
            let args_str = &tool_call.function.arguments;
            
            eprintln!("[DEBUG] Executing tool: {} with args: {}", tool_name, args_str);
            
            let params: serde_json::Value = serde_json::from_str(args_str)
                .unwrap_or_else(|_| serde_json::json!({}));
            
            let result = executor.execute(tool_name, params).await;
            
            let result_json = match result {
                Ok(r) => {
                    eprintln!("[DEBUG] Tool {} result: {:?}", tool_name, r);
                    tool_results_metadata.push(serde_json::json!({
                        "tool": tool_name,
                        "arguments": args_str,
                        "result": r
                    }));
                    serde_json::to_string(&r).unwrap_or_else(|_| "{}".to_string())
                }
                Err(e) => {
                    eprintln!("[ERROR] Tool {} failed: {}", tool_name, e);
                    tool_results_metadata.push(serde_json::json!({
                        "tool": tool_name,
                        "arguments": args_str,
                        "error": e.to_string()
                    }));
                    format!("{{\"error\": \"{}\"}}", e)
                }
            };
            
            tool_results.push(compliance_assistant::llm::ChatMessage::tool_result(
                tool_call.id.clone(),
                result_json,
            ));
        }
        
        messages.push(compliance_assistant::llm::ChatMessage::assistant_with_tools(
            response.content.clone(),
            tool_calls,
        ));
        
        for result in tool_results {
            messages.push(result);
        }
    }
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
    
    let retriever = compliance_assistant::knowledge::Retriever::new(state.db_pool.clone());
    retriever
        .add_document(&document)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(DocumentResponse::from(document))
}

#[tauri::command]
pub async fn process_document(
    id: String,
    state: State<'_, AppState>,
) -> Result<DocumentResponse, String> {
    let config = state.config.read().await;
    let embedding_service = EmbeddingService::new(config.embedding_model.clone());
    drop(config);
    
    let retriever = compliance_assistant::knowledge::Retriever::new(state.db_pool.clone())
        .with_embedding(embedding_service);
    
    let document = retriever
        .get_document(&id)
        .await
        .map_err(|e| format!("Failed to get document: {}", e))?
        .ok_or_else(|| "Document not found".to_string())?;
    
    retriever
        .process_document(&document)
        .await
        .map_err(|e| format!("Failed to process document '{}': {}", document.file_path, e))?;
    
    let updated = retriever
        .get_document(&id)
        .await
        .map_err(|e| format!("Failed to get updated document: {}", e))?
        .ok_or_else(|| "Document not found".to_string())?;
    
    Ok(DocumentResponse::from(updated))
}

#[tauri::command]
pub async fn get_document(
    id: String,
    state: State<'_, AppState>,
) -> Result<DocumentResponse, String> {
    let retriever = compliance_assistant::knowledge::Retriever::new(state.db_pool.clone());
    retriever
        .get_document(&id)
        .await
        .map_err(|e| e.to_string())?
        .map(DocumentResponse::from)
        .ok_or_else(|| "Document not found".to_string())
}

#[tauri::command]
pub async fn get_document_chunks(
    document_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let retriever = compliance_assistant::knowledge::Retriever::new(state.db_pool.clone());
    retriever
        .get_chunks(&document_id)
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
    let config = state.config.read().await;
    let embedding_service = EmbeddingService::new(config.embedding_model.clone());
    let retriever = compliance_assistant::knowledge::Retriever::new(state.db_pool.clone())
        .with_embedding(embedding_service);
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
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let config = state.config.read().await;
    let embedding_service = EmbeddingService::new(config.embedding_model.clone());
    drop(config);
    
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(FileReadTool));
    registry.register(Arc::new(KnowledgeSearchTool::new(state.db_pool.clone(), embedding_service)));
    registry.register(Arc::new(CalculatorTool));
    
    let tools: Vec<serde_json::Value> = registry
        .list()
        .into_iter()
        .map(|t| {
            serde_json::json!({
                "name": t.name,
                "description": t.description,
                "parameters": t.parameters,
                "requireConfirmation": t.require_confirmation
            })
        })
        .collect();
    
    Ok(tools)
}

#[tauri::command]
pub async fn execute_tool(
    tool_name: String,
    params: serde_json::Value,
    state: State<'_, AppState>,
) -> Result<compliance_assistant::tools::ToolResult, String> {
    let config = state.config.read().await;
    let embedding_service = EmbeddingService::new(config.embedding_model.clone());
    drop(config);
    
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(FileReadTool));
    registry.register(Arc::new(KnowledgeSearchTool::new(state.db_pool.clone(), embedding_service)));
    registry.register(Arc::new(CalculatorTool));
    
    let permission_manager = PermissionManager::new(state.db_pool.clone());
    let executor = ToolExecutor::new(registry, permission_manager);
    
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