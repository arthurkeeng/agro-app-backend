use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    
    println!("Running database migrations...");
    
    // You can add manual migrations here or use sqlx migrate
    // For now, let's just test the connection
    sqlx::query("SELECT 1").fetch_one(&pool).await?;
    
    println!("Database connection successful!");
    
    Ok(())
}
