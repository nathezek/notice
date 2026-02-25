use sqlx::postgres::{PgPool, PgPoolOptions};

/// Create a PostgreSQL connection pool.
pub async fn create_pool(database_url: &str) -> Result<PgPool, notice_core::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .map_err(|e| notice_core::Error::Database(e.to_string()))?;

    tracing::info!("Connected to PostgreSQL");

    // Verify the connection works
    sqlx::query("SELECT 1")
        .execute(&pool)
        .await
        .map_err(|e| notice_core::Error::Database(e.to_string()))?;

    tracing::info!("PostgreSQL connection verified");

    Ok(pool)
}

// Future: query functions for documents, users, knowledge graph, etc.
// Those will be added when we define the schema in Step 2.
