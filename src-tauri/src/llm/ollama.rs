use super::{ChatMessage, ChatRequest, ChatResponse, LlmClient, LlmStream};
use crate::{AppError, AppResult};
use async_trait::async_trait;
use futures_util::{Stream, StreamExt};
use reqwest::Client;
use serde::Deserialize;

pub struct OllamaClient {
    base_url: String,
    model: String,
    temperature: f32,
    top_p: f32,
    client: Client,
}

impl OllamaClient {
    pub fn new(base_url: String, model: String, temperature: f32, top_p: f32) -> Self {
        Self {
            base_url,
            model,
            temperature,
            top_p,
            client: Client::new(),
        }
    }
    
    pub fn default_client() -> Self {
        Self::new(
            "http://localhost:11434".to_string(),
            "llama3.2".to_string(),
            0.7,
            0.9,
        )
    }
}

#[async_trait]
impl LlmClient for OllamaClient {
    async fn chat(&self, messages: Vec<ChatMessage>) -> AppResult<String> {
        let url = format!("{}/api/chat", self.base_url);
        
        let request = ChatRequest {
            model: self.model.clone(),
            messages,
            temperature: Some(self.temperature),
            top_p: Some(self.top_p),
            stream: Some(false),
        };
        
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::LlmRequest(e.to_string()))?;
        
        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(AppError::LlmRequest(format!("Ollama API error: {}", error)));
        }
        
        let ollama_response: OllamaChatResponse = response
            .json()
            .await
            .map_err(|e| AppError::LlmRequest(e.to_string()))?;
        
        Ok(ollama_response.message.content)
    }
    
    async fn chat_stream(&self, messages: Vec<ChatMessage>) -> AppResult<LlmStream> {
        let url = format!("{}/api/chat", self.base_url);
        
        let request = ChatRequest {
            model: self.model.clone(),
            messages,
            temperature: Some(self.temperature),
            top_p: Some(self.top_p),
            stream: Some(true),
        };
        
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::LlmRequest(e.to_string()))?;
        
        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(AppError::LlmRequest(format!("Ollama API error: {}", error)));
        }
        
        let stream = response.bytes_stream().map(move |result| {
            match result {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    
                    for line in text.lines() {
                        if line.starts_with('{') {
                            if let Ok(chunk) = serde_json::from_str::<OllamaStreamChunk>(line) {
                                if !chunk.message.content.is_empty() {
                                    return Ok(chunk.message.content);
                                }
                            }
                        }
                    }
                    Ok(String::new())
                }
                Err(e) => Err(AppError::LlmRequest(e.to_string())),
            }
        });
        
        Ok(Box::pin(stream))
    }
}

#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    message: OllamaMessage,
}

#[derive(Debug, Deserialize)]
struct OllamaMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct OllamaStreamChunk {
    message: OllamaMessage,
}