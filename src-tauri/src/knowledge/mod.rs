mod chunker;
mod parser;
mod retriever;

pub use chunker::{Chunker, chunk_document};
pub use parser::{DocumentParser, DocumentType, ParsedDocument};
pub use retriever::Retriever;

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub title: String,
    pub file_path: String,
    pub file_type: String,
    pub file_size: u64,
    pub status: DocumentStatus,
    pub chunk_count: usize,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DocumentStatus {
    Pending,
    Processing,
    Ready,
    Failed,
}

impl Default for DocumentStatus {
    fn default() -> Self {
        Self::Pending
    }
}

impl Document {
    pub fn new(title: String, file_path: String, file_type: String, file_size: u64) -> Self {
        let now = Utc::now().timestamp();
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            file_path,
            file_type,
            file_size,
            status: DocumentStatus::Pending,
            chunk_count: 0,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    pub id: String,
    pub document_id: String,
    pub chunk_index: usize,
    pub content: String,
    pub embedding: Option<Vec<f32>>,
    pub created_at: i64,
}

impl DocumentChunk {
    pub fn new(document_id: String, chunk_index: usize, content: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            document_id,
            chunk_index,
            content,
            embedding: None,
            created_at: Utc::now().timestamp(),
        }
    }
    
    pub fn with_embedding(mut self, embedding: Vec<f32>) -> Self {
        self.embedding = Some(embedding);
        self
    }
}