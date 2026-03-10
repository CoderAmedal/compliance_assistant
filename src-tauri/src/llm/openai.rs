use super::{ChatMessage, ChatRequest, ChatResponse, LlmClient, LlmStream};
use crate::{AppError, AppResult};
use async_trait::async_trait;
use futures_util::{Stream, StreamExt};
use reqwest::Client;
use serde::Deserialize;

pub struct OpenAIClient {
    base_url: String,
    api_key: Option<String>,
    model: String,
    temperature: f32,
    top_p: f32,
    client: Client,
}

impl OpenAIClient {
    pub fn new(
        base_url: String,
        api_key: Option<String>,
        model: String,
        temperature: f32,
        top_p: f32,
    ) -> Self {
        Self {
            base_url,
            api_key,
            model,
            temperature,
            top_p,
            client: Client::new(),
        }
    }
    
    pub fn default_client() -> Self {
        Self::new(
            "https://api.openai.com/v1".to_string(),
            None,
            "gpt-3.5-turbo".to_string(),
            0.7,
            0.9,
        )
    }
}

#[async_trait]
impl LlmClient for OpenAIClient {
    async fn chat(&self, messages: Vec<ChatMessage>) -> AppResult<String> {
        let url = format!("{}/chat/completions", self.base_url);
        
        let request = ChatRequest {
            model: self.model.clone(),
            messages,
            temperature: Some(self.temperature),
            top_p: Some(self.top_p),
            stream: Some(false),
        };
        
        let mut builder = self.client.post(&url).json(&request);
        
        if let Some(ref api_key) = self.api_key {
            builder = builder.header("Authorization", format!("Bearer {}", api_key));
        }
        
        let response = builder
            .send()
            .await
            .map_err(|e| AppError::LlmRequest(e.to_string()))?;
        
        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(AppError::LlmRequest(format!("OpenAI API error: {}", error)));
        }
        
        let openai_response: OpenAIChatResponse = response
            .json()
            .await
            .map_err(|e| AppError::LlmRequest(e.to_string()))?;
        
        Ok(openai_response.choices[0].message.content.clone())
    }
    
    async fn chat_stream(&self, messages: Vec<ChatMessage>) -> AppResult<LlmStream> {
        let url = format!("{}/chat/completions", self.base_url);
        
        let request = ChatRequest {
            model: self.model.clone(),
            messages,
            temperature: Some(self.temperature),
            top_p: Some(self.top_p),
            stream: Some(true),
        };
        
        let mut builder = self.client.post(&url).json(&request);
        
        if let Some(ref api_key) = self.api_key {
            builder = builder.header("Authorization", format!("Bearer {}", api_key));
        }
        
        let response = builder
            .send()
            .await
            .map_err(|e| AppError::LlmRequest(e.to_string()))?;
        
        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(AppError::LlmRequest(format!("OpenAI API error: {}", error)));
        }
        
        let stream = response.bytes_stream().map(move |result| {
            match result {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    
                    for line in text.lines() {
                        if line.starts_with("data: ") {
                            let data = &line[6..];
                            if data == "[DONE]" {
                                return Ok(String::new());
                            }
                            
                            if let Ok(chunk) = serde_json::from_str::<OpenAIStreamChunk>(data) {
                                if let Some(delta) = chunk.choices.first() {
                                    if let Some(content) = &delta.delta.content {
                                        return Ok(content.clone());
                                    }
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
struct OpenAIChatResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAIMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamChunk {
    choices: Vec<OpenAIStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamChoice {
    delta: OpenAIDelta,
}

#[derive(Debug, Deserialize)]
struct OpenAIDelta {
    content: Option<String>,
}