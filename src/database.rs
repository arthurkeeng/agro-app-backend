use sqlx::{PgPool, Row};
use anyhow::Result;

#[derive(Clone)]
pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url).await?;
        
        // Test the connection
        sqlx::query("SELECT 1").fetch_one(&pool).await?;
        
        log::info!("Database connected successfully");
        
        Ok(Database { pool })
    }
}
