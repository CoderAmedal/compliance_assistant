mod session;
mod message;

pub use session::Session;
pub use message::{Message, MessageRole};

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub title: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl ChatSession {
    pub fn new(title: String) -> Self {
        let now = Utc::now().timestamp();
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub session_id: String,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: i64,
    pub metadata: Option<serde_json::Value>,
}

impl ChatMessage {
    pub fn new(session_id: String, role: MessageRole, content: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            session_id,
            role,
            content,
            timestamp: Utc::now().timestamp(),
            metadata: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_session_creation() {
        let session = ChatSession::new("Test Session".to_string());
        assert_eq!(session.title, "Test Session");
        assert!(!session.id.is_empty());
    }
    
    #[test]
    fn test_message_creation() {
        let msg = ChatMessage::new(
            "session-1".to_string(),
            MessageRole::User,
            "Hello".to_string(),
        );
        assert_eq!(msg.content, "Hello");
        assert_eq!(msg.role, MessageRole::User);
    }
}