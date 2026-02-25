pub mod crawl_queue;
pub mod documents;
pub mod knowledge_graph;
pub mod search_history;
pub mod users;

use sqlx::postgres::{PgPool, PgPoolOptions};

/// Create a PostgreSQL connection pool.
pub async fn create_pool(database_url: &str) -> Result<PgPool, notice_core::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .map_err(|e| notice_core::Error::Database(e.to_string()))?;

    tracing::info!("Connected to PostgreSQL");

    sqlx::query("SELECT 1")
        .execute(&pool)
        .await
        .map_err(|e| notice_core::Error::Database(e.to_string()))?;

    tracing::info!("PostgreSQL connection verified");

    Ok(pool)
}

/// Run embedded migrations.
pub async fn run_migrations(pool: &PgPool) -> Result<(), notice_core::Error> {
    sqlx::migrate!("../../migrations")
        .run(pool)
        .await
        .map_err(|e| notice_core::Error::Database(e.to_string()))?;

    tracing::info!("Database migrations applied");
    Ok(())
}
