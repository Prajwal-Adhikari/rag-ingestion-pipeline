use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
pub async fn create_pool() -> Result<PgPool, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL NOT SET");
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
}

#[derive(Debug)]
pub struct Job {
    pub id: uuid::Uuid,
    pub file_name: String,
    pub file_path: String,
    pub status: String,
    pub retries: i32,
}
