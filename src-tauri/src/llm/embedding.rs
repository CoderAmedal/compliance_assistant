use crate::{AppError, AppResult, config::{EmbeddingModelConfig, ModelProvider}};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct EmbeddingService {
    config: EmbeddingModelConfig,
    client: Client,
}

impl EmbeddingService {
    pub fn new(config: EmbeddingModelConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }
    
    pub async fn embed(&self, texts: &[String]) -> AppResult<Vec<Vec<f32>>> {
        match self.config.provider {
            ModelProvider::Ollama => self.embed_ollama(texts).await,
            ModelProvider::OpenAI => self.embed_openai(texts).await,
        }
    }
    
    pub async fn embed_single(&self, text: &str) -> AppResult<Vec<f32>> {
        let embeddings = self.embed(&[text.to_string()]).await?;
        Ok(embeddings.into_iter().next().unwrap_or_default())
    }
    
    fn sanitize_text(text: &str) -> String {
        let cleaned: String = text
            .chars()
            .filter(|c| {
                !c.is_control() 
                    || *c == '\n' 
                    || *c == '\t'
            })
            .collect();
        
        let normalized = cleaned
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        
        if normalized.len() < 10 {
            return String::new();
        }
        
        if Self::is_garbled_text(&normalized) {
            eprintln!("[WARN] Detected garbled text, compacting...");
            return Self::compact_garbled_text(&normalized);
        }
        
        normalized
    }
    
    fn is_garbled_text(text: &str) -> bool {
        let chars: Vec<char> = text.chars().collect();
        if chars.len() < 20 {
            return false;
        }
        
        let mut single_char_count = 0;
        let mut check_len = 0;
        
        for i in 0..chars.len().saturating_sub(2) {
            if chars[i].is_alphanumeric() 
                && chars[i + 1] == ' ' 
                && chars[i + 2].is_alphanumeric() {
                single_char_count += 1;
            }
            check_len += 1;
        }
        
        if check_len > 0 {
            let ratio = single_char_count as f32 / check_len as f32;
            ratio > 0.3
        } else {
            false
        }
    }
    
    fn compact_garbled_text(text: &str) -> String {
        let chars: Vec<char> = text.chars().collect();
        let mut result = Vec::new();
        let mut i = 0;
        
        while i < chars.len() {
            if i + 2 < chars.len() 
                && chars[i].is_alphanumeric() 
                && chars[i + 1] == ' ' 
                && chars[i + 2].is_alphanumeric() {
                result.push(chars[i]);
                i += 2;
            } else if chars[i] != ' ' {
                result.push(chars[i]);
                i += 1;
            } else {
                if result.last().map_or(true, |c| *c != ' ') {
                    result.push(' ');
                }
                i += 1;
            }
        }
        
        let compacted: String = result.into_iter().collect();
        compacted.split_whitespace().collect::<Vec<_>>().join(" ")
    }
    
    async fn embed_ollama(&self, texts: &[String]) -> AppResult<Vec<Vec<f32>>> {
        let url = format!("{}/api/embeddings", self.config.ollama.base_url);
        eprintln!("[DEBUG] Ollama embedding URL: {}", url);
        eprintln!("[DEBUG] Ollama model: {}", self.config.ollama.model);
        eprintln!("[DEBUG] Total texts to embed: {}", texts.len());
        
        let mut all_embeddings = Vec::new();
        
        for (idx, text) in texts.iter().enumerate() {
            let sanitized = Self::sanitize_text(text);
            
            eprintln!("[DEBUG] === Embedding request {} ===", idx);
            eprintln!("[DEBUG] Original text length: {} chars", text.len());
            eprintln!("[DEBUG] Sanitized text length: {} chars", sanitized.len());
            
            if sanitized.is_empty() {
                eprintln!("[WARN] Empty text after sanitization at index {}, using zero embedding", idx);
                all_embeddings.push(vec![0.0; self.embedding_dimension()]);
                continue;
            }
            
            if sanitized.len() > 8000 {
                eprintln!("[WARN] Text too long at index {} ({} chars), truncating", idx, sanitized.len());
            }
            let truncated: String = sanitized.chars().take(8000).collect();
            
            let request = OllamaEmbeddingRequest {
                model: self.config.ollama.model.clone(),
                prompt: truncated.clone(),
            };
            
            let request_json = serde_json::to_string_pretty(&request)
                .unwrap_or_else(|e| format!("Failed to serialize: {}", e));
            eprintln!("[DEBUG] Request JSON:\n{}", request_json);
            
            let response = self
                .client
                .post(&url)
                .json(&request)
                .send()
                .await
                .map_err(|e| AppError::LlmRequest(e.to_string()))?;
            
            let status = response.status();
            eprintln!("[DEBUG] Response status: {}", status);
            
            if !status.is_success() {
                let error_body = response.text().await.unwrap_or_default();
                eprintln!("[ERROR] Ollama embedding failed with status {}", status);
                eprintln!("[ERROR] Response body: {}", error_body);
                eprintln!("[ERROR] Request prompt (first 500 chars): {}", &truncated[..truncated.len().min(500)]);
                all_embeddings.push(vec![0.0; self.embedding_dimension()]);
                continue;
            }
            
            let response_text = response.text().await
                .map_err(|e| AppError::LlmRequest(e.to_string()))?;
            eprintln!("[DEBUG] Response body length: {} bytes", response_text.len());
            
            let embedding_response: OllamaEmbeddingResponse = serde_json::from_str(&response_text)
                .map_err(|e| {
                    eprintln!("[ERROR] Failed to parse response: {}", e);
                    eprintln!("[ERROR] Response text: {}", &response_text[..response_text.len().min(1000)]);
                    AppError::LlmRequest(format!("JSON parse error: {}", e))
                })?;
            
            let embedding = embedding_response.embedding;
            eprintln!("[DEBUG] Embedding dimension: {}", embedding.len());
            
            if embedding.iter().any(|&v| v.is_nan() || v.is_infinite()) {
                eprintln!("[WARN] Embedding contains NaN/Inf at index {}", idx);
                all_embeddings.push(vec![0.0; self.embedding_dimension()]);
                continue;
            }
            
            all_embeddings.push(embedding);
        }
        
        Ok(all_embeddings)
    }
    
    async fn embed_openai(&self, texts: &[String]) -> AppResult<Vec<Vec<f32>>> {
        let url = format!("{}/embeddings", self.config.openai.base_url);
        
        let api_key = self.config.openai.api_key.as_ref()
            .ok_or_else(|| AppError::Config("OpenAI API key not set".to_string()))?;
        
        let request = OpenAiEmbeddingRequest {
            model: self.config.openai.model.clone(),
            input: texts.to_vec(),
        };
        
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::LlmRequest(e.to_string()))?;
        
        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(AppError::LlmRequest(format!("OpenAI embedding error: {}", error)));
        }
        
        let embedding_response: OpenAiEmbeddingResponse = response
            .json()
            .await
            .map_err(|e| AppError::LlmRequest(e.to_string()))?;
        
        Ok(embedding_response
            .data
            .into_iter()
            .map(|d| d.embedding)
            .collect())
    }
    
    pub fn embedding_dimension(&self) -> usize {
        match self.config.provider {
            ModelProvider::Ollama => 768,
            ModelProvider::OpenAI => match self.config.openai.model.as_str() {
                "text-embedding-3-large" => 3072,
                _ => 1536,
            },
        }
    }
}

#[derive(Debug, Serialize)]
struct OllamaEmbeddingRequest {
    model: String,
    prompt: String,
}

#[derive(Debug, Deserialize)]
struct OllamaEmbeddingResponse {
    embedding: Vec<f32>,
}

#[derive(Debug, Serialize)]
struct OpenAiEmbeddingRequest {
    model: String,
    input: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiEmbeddingResponse {
    data: Vec<OpenAiEmbeddingData>,
}

#[derive(Debug, Deserialize)]
struct OpenAiEmbeddingData {
    embedding: Vec<f32>,
}