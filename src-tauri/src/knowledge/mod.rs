mod parser;
mod retriever;

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
    pub created_at: i64,
    pub updated_at: i64,
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
    pub created_at: i64,
}

impl DocumentChunk {
    pub fn new(document_id: String, chunk_index: usize, content: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            document_id,
            chunk_index,
            content,
            created_at: Utc::now().timestamp(),
        }
    }
}