use crate::storage::DatabasePool;
use crate::{AppError, AppResult};
use super::{ChatSession, ChatMessage, MessageRole};
use sqlx::Row;

pub struct Session {
    pool: DatabasePool,
}

impl Session {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
    
    pub async fn create(&self, title: String) -> AppResult<ChatSession> {
        let session = ChatSession::new(title);
        
        sqlx::query(
            r#"
            INSERT INTO sessions (id, title, created_at, updated_at)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(&session.id)
        .bind(&session.title)
        .bind(session.created_at)
        .bind(session.updated_at)
        .execute(&self.pool)
        .await?;
        
        Ok(session)
    }
    
    pub async fn get(&self, id: &str) -> AppResult<Option<ChatSession>> {
        let row = sqlx::query(
            r#"
            SELECT id, title, created_at, updated_at
            FROM sessions
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(row.map(|r| ChatSession {
            id: r.get("id"),
            title: r.get("title"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }
    
    pub async fn list(&self) -> AppResult<Vec<ChatSession>> {
        let rows = sqlx::query(
            r#"
            SELECT id, title, created_at, updated_at
            FROM sessions
            ORDER BY updated_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(rows
            .into_iter()
            .map(|r| ChatSession {
                id: r.get("id"),
                title: r.get("title"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            })
            .collect())
    }
    
    pub async fn delete(&self, id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM sessions WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
    
    pub async fn update_title(&self, id: &str, title: &str) -> AppResult<()> {
        sqlx::query("UPDATE sessions SET title = ?, updated_at = ? WHERE id = ?")
            .bind(title)
            .bind(chrono::Utc::now().timestamp())
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
    
    pub async fn add_message(&self, message: ChatMessage) -> AppResult<ChatMessage> {
        sqlx::query(
            r#"
            INSERT INTO messages (id, session_id, role, content, timestamp, metadata)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&message.id)
        .bind(&message.session_id)
        .bind(message.role.as_str())
        .bind(&message.content)
        .bind(message.timestamp)
        .bind(&message.metadata)
        .execute(&self.pool)
        .await?;
        
        sqlx::query("UPDATE sessions SET updated_at = ? WHERE id = ?")
            .bind(message.timestamp)
            .bind(&message.session_id)
            .execute(&self.pool)
            .await?;
        
        Ok(message)
    }
    
    pub async fn get_messages(&self, session_id: &str) -> AppResult<Vec<ChatMessage>> {
        let rows = sqlx::query(
            r#"
            SELECT id, session_id, role, content, timestamp, metadata
            FROM messages
            WHERE session_id = ?
            ORDER BY timestamp ASC
            "#,
        )
        .bind(session_id)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(rows
            .into_iter()
            .map(|r| {
                let role_str: String = r.get("role");
                ChatMessage {
                    id: r.get("id"),
                    session_id: r.get("session_id"),
                    role: MessageRole::from_str(&role_str).unwrap_or(MessageRole::User),
                    content: r.get("content"),
                    timestamp: r.get("timestamp"),
                    metadata: r.get("metadata"),
                }
            })
            .collect())
    }
}