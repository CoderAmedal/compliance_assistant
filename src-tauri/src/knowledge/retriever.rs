use crate::storage::DatabasePool;
use crate::llm::EmbeddingService;
use crate::AppResult;
use sqlx::Row;
use super::{Document, DocumentChunk, DocumentStatus, chunk_document, DocumentParser};

pub struct Retriever {
    pool: DatabasePool,
    embedding_service: Option<EmbeddingService>,
}

impl Retriever {
    pub fn new(pool: DatabasePool) -> Self {
        Self {
            pool,
            embedding_service: None,
        }
    }
    
    pub fn with_embedding(mut self, service: EmbeddingService) -> Self {
        self.embedding_service = Some(service);
        self
    }
    
    pub async fn add_document(&self, document: &Document) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO documents (id, title, file_path, file_type, file_size, status, chunk_count, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&document.id)
        .bind(&document.title)
        .bind(&document.file_path)
        .bind(&document.file_type)
        .bind(document.file_size as i64)
        .bind("pending")
        .bind(0i64)
        .bind(document.created_at)
        .bind(document.updated_at)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn update_document_status(&self, id: &str, status: DocumentStatus, chunk_count: usize) -> AppResult<()> {
        let status_str = match status {
            DocumentStatus::Pending => "pending",
            DocumentStatus::Processing => "processing",
            DocumentStatus::Ready => "ready",
            DocumentStatus::Failed => "failed",
        };
        
        sqlx::query(
            r#"
            UPDATE documents SET status = ?, chunk_count = ?, updated_at = ? WHERE id = ?
            "#,
        )
        .bind(status_str)
        .bind(chunk_count as i64)
        .bind(chrono::Utc::now().timestamp())
        .bind(id)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn add_chunk(&self, chunk: &DocumentChunk) -> AppResult<()> {
        let embedding_json = chunk.embedding.as_ref()
            .map(|e| serde_json::to_string(e).unwrap_or_default());
        
        sqlx::query(
            r#"
            INSERT INTO vectors (id, document_id, chunk_index, content, embedding, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&chunk.id)
        .bind(&chunk.document_id)
        .bind(chunk.chunk_index as i64)
        .bind(&chunk.content)
        .bind(embedding_json)
        .bind(chunk.created_at)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn add_chunks(&self, chunks: &[DocumentChunk]) -> AppResult<()> {
        for chunk in chunks {
            self.add_chunk(chunk).await?;
        }
        Ok(())
    }
    
    pub async fn process_document(&self, document: &Document) -> AppResult<Vec<DocumentChunk>> {
        self.update_document_status(&document.id, DocumentStatus::Processing, 0).await?;
        
        let path = std::path::Path::new(&document.file_path);
        let parsed = match DocumentParser::parse(path) {
            Ok(p) => {
                eprintln!("[DEBUG] Successfully parsed document: {} ({} chars)", document.file_path, p.content.len());
                p
            },
            Err(e) => {
                eprintln!("[ERROR] Failed to parse document '{}': {}", document.file_path, e);
                self.update_document_status(&document.id, DocumentStatus::Failed, 0).await?;
                return Err(e);
            }
        };
        
        let chunks = chunk_document(&document.id, &parsed.content);
        eprintln!("[DEBUG] Created {} chunks for document: {}", chunks.len(), document.file_path);
        
        if let Some(ref service) = self.embedding_service {
            let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
            match service.embed(&texts).await {
                Ok(embeddings) => {
                    eprintln!("[DEBUG] Generated {} embeddings", embeddings.len());
                    let chunks_with_embeddings: Vec<DocumentChunk> = chunks
                        .into_iter()
                        .zip(embeddings.into_iter())
                        .map(|(chunk, embedding)| chunk.with_embedding(embedding))
                        .collect();
                    
                    self.add_chunks(&chunks_with_embeddings).await?;
                    self.update_document_status(&document.id, DocumentStatus::Ready, chunks_with_embeddings.len()).await?;
                    
                    Ok(chunks_with_embeddings)
                }
                Err(e) => {
                    eprintln!("[ERROR] Failed to generate embeddings for document '{}': {}", document.id, e);
                    self.update_document_status(&document.id, DocumentStatus::Failed, 0).await?;
                    return Err(e);
                }
            }
        } else {
            eprintln!("[DEBUG] No embedding service configured");
            self.add_chunks(&chunks).await?;
            self.update_document_status(&document.id, DocumentStatus::Ready, chunks.len()).await?;
            Ok(chunks)
        }
    }
    
    pub async fn list_documents(&self) -> AppResult<Vec<Document>> {
        let rows = sqlx::query(
            r#"
            SELECT id, title, file_path, file_type, file_size, status, chunk_count, created_at, updated_at
            FROM documents
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(rows
            .into_iter()
            .map(|r| {
                let status_str: String = r.get("status");
                let status = match status_str.as_str() {
                    "processing" => DocumentStatus::Processing,
                    "ready" => DocumentStatus::Ready,
                    "failed" => DocumentStatus::Failed,
                    _ => DocumentStatus::Pending,
                };
                Document {
                    id: r.get("id"),
                    title: r.get("title"),
                    file_path: r.get("file_path"),
                    file_type: r.get("file_type"),
                    file_size: r.get::<i64, _>("file_size") as u64,
                    status,
                    chunk_count: r.get::<i64, _>("chunk_count") as usize,
                    created_at: r.get("created_at"),
                    updated_at: r.get("updated_at"),
                }
            })
            .collect())
    }
    
    pub async fn get_document(&self, id: &str) -> AppResult<Option<Document>> {
        let row = sqlx::query(
            r#"
            SELECT id, title, file_path, file_type, file_size, status, chunk_count, created_at, updated_at
            FROM documents WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(row.map(|r| {
            let status_str: String = r.get("status");
            let status = match status_str.as_str() {
                "processing" => DocumentStatus::Processing,
                "ready" => DocumentStatus::Ready,
                "failed" => DocumentStatus::Failed,
                _ => DocumentStatus::Pending,
            };
            Document {
                id: r.get("id"),
                title: r.get("title"),
                file_path: r.get("file_path"),
                file_type: r.get("file_type"),
                file_size: r.get::<i64, _>("file_size") as u64,
                status,
                chunk_count: r.get::<i64, _>("chunk_count") as usize,
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            }
        }))
    }
    
    pub async fn delete_document(&self, id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM vectors WHERE document_id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        sqlx::query("DELETE FROM documents WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
    
    pub async fn get_chunks(&self, document_id: &str) -> AppResult<Vec<DocumentChunk>> {
        let rows = sqlx::query(
            r#"
            SELECT id, document_id, chunk_index, content, embedding, created_at
            FROM vectors
            WHERE document_id = ?
            ORDER BY chunk_index
            "#,
        )
        .bind(document_id)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(rows
            .into_iter()
            .map(|r| {
                let embedding_json: Option<String> = r.get("embedding");
                let embedding = embedding_json.and_then(|json| serde_json::from_str(&json).ok());
                DocumentChunk {
                    id: r.get("id"),
                    document_id: r.get("document_id"),
                    chunk_index: r.get::<i64, _>("chunk_index") as usize,
                    content: r.get("content"),
                    embedding,
                    created_at: r.get("created_at"),
                }
            })
            .collect())
    }
    
    pub async fn search(&self, query: &str, limit: usize) -> AppResult<Vec<DocumentChunk>> {
        if let Some(ref service) = self.embedding_service {
            let query_embedding = service.embed_single(query).await?;
            self.search_by_embedding(&query_embedding, limit).await
        } else {
            self.search_by_text(query, limit).await
        }
    }
    
    pub async fn search_by_text(&self, query: &str, limit: usize) -> AppResult<Vec<DocumentChunk>> {
        let rows = sqlx::query(
            r#"
            SELECT id, document_id, chunk_index, content, embedding, created_at
            FROM vectors
            WHERE content LIKE ?
            LIMIT ?
            "#,
        )
        .bind(format!("%{}%", query))
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(rows
            .into_iter()
            .map(|r| {
                let embedding_json: Option<String> = r.get("embedding");
                let embedding = embedding_json.and_then(|json| serde_json::from_str(&json).ok());
                DocumentChunk {
                    id: r.get("id"),
                    document_id: r.get("document_id"),
                    chunk_index: r.get::<i64, _>("chunk_index") as usize,
                    content: r.get("content"),
                    embedding,
                    created_at: r.get("created_at"),
                }
            })
            .collect())
    }
    
    pub async fn search_by_embedding(&self, query_embedding: &[f32], limit: usize) -> AppResult<Vec<DocumentChunk>> {
        let rows = sqlx::query(
            r#"
            SELECT id, document_id, chunk_index, content, embedding, created_at
            FROM vectors
            WHERE embedding IS NOT NULL
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        
        let mut chunks_with_scores: Vec<(DocumentChunk, f32)> = rows
            .into_iter()
            .filter_map(|r| {
                let embedding_json: Option<String> = r.get("embedding");
                let embedding: Vec<f32> = embedding_json.and_then(|json| serde_json::from_str(&json).ok())?;
                
                let score = cosine_similarity(query_embedding, &embedding);
                
                let chunk = DocumentChunk {
                    id: r.get("id"),
                    document_id: r.get("document_id"),
                    chunk_index: r.get::<i64, _>("chunk_index") as usize,
                    content: r.get("content"),
                    embedding: Some(embedding),
                    created_at: r.get("created_at"),
                };
                
                Some((chunk, score))
            })
            .collect();
        
        chunks_with_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        chunks_with_scores.truncate(limit);
        
        Ok(chunks_with_scores.into_iter().map(|(c, _)| c).collect())
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    
    dot_product / (norm_a * norm_b)
}