use sqlx::SqlitePool;

pub type DatabasePool = SqlitePool;

pub struct Database {
    pool: DatabasePool,
}

impl Database {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
    
    pub fn pool(&self) -> &DatabasePool {
        &self.pool
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_database_connection() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect(":memory:")
            .await
            .unwrap();
        
        let db = Database::new(pool);
        assert!(db.pool().is_closed() == false);
    }
}