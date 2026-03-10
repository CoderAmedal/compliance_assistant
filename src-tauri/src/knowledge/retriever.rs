use crate::storage::DatabasePool;
use crate::AppResult;
use sqlx::Row;
use super::{Document, DocumentChunk};

pub struct Retriever {
    pool: DatabasePool,
}

impl Retriever {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
    
    pub async fn add_document(&self, document: Document, chunks: Vec<DocumentChunk>) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO documents (id, title, file_path, file_type, file_size, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&document.id)
        .bind(&document.title)
        .bind(&document.file_path)
        .bind(&document.file_type)
        .bind(document.file_size as i64)
        .bind(document.created_at)
        .bind(document.updated_at)
        .execute(&self.pool)
        .await?;
        
        for chunk in chunks {
            sqlx::query(
                r#"
                INSERT INTO vectors (id, document_id, chunk_index, content, created_at)
                VALUES (?, ?, ?, ?, ?)
                "#,
            )
            .bind(&chunk.id)
            .bind(&chunk.document_id)
            .bind(chunk.chunk_index as i64)
            .bind(&chunk.content)
            .bind(chunk.created_at)
            .execute(&self.pool)
            .await?;
        }
        
        Ok(())
    }
    
    pub async fn list_documents(&self) -> AppResult<Vec<Document>> {
        let rows = sqlx::query(
            r#"
            SELECT id, title, file_path, file_type, file_size, created_at, updated_at
            FROM documents
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(rows
            .into_iter()
            .map(|r| Document {
                id: r.get("id"),
                title: r.get("title"),
                file_path: r.get("file_path"),
                file_type: r.get("file_type"),
                file_size: r.get::<i64, _>("file_size") as u64,
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            })
            .collect())
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
    
    pub async fn search(&self, query: &str, limit: usize) -> AppResult<Vec<DocumentChunk>> {
        let rows = sqlx::query(
            r#"
            SELECT id, document_id, chunk_index, content, created_at
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
            .map(|r| DocumentChunk {
                id: r.get("id"),
                document_id: r.get("document_id"),
                chunk_index: r.get::<i64, _>("chunk_index") as usize,
                content: r.get("content"),
                created_at: r.get("created_at"),
            })
            .collect())
    }
}