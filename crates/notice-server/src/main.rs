mod error;
mod routes;
mod state;

use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ── 1. Load .env ──
    dotenvy::dotenv().ok();

    // ── 2. Initialize tracing ──
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("notice_server=debug,tower_http=debug")),
        )
        .init();

    tracing::info!("Starting Notice V2");

    // ── 3. Load configuration ──
    let config = notice_core::config::AppConfig::from_env();

    // ── 4. Connect to PostgreSQL ──
    let db_pool = notice_db::create_pool(&config.database_url).await?;

    // ── 5. Run migrations ──
    notice_db::run_migrations(&db_pool).await?;

    // ── 6. Connect to Meilisearch ──
    let search_client = notice_search::SearchClient::new(&config.meili_url, &config.meili_api_key)?;
    search_client.health().await?;
    tracing::info!("Meilisearch is healthy");

    // ── 7. Configure Meilisearch index ──
    search_client.configure_index().await?;

    // Log current document count
    match search_client.document_count().await {
        Ok(count) => tracing::info!("Meilisearch documents index: {} documents", count),
        Err(e) => tracing::warn!("Could not get Meilisearch document count: {}", e),
    }

    // ── 8. Create Gemini client ──
    let gemini_client = notice_ai::GeminiClient::new(&config.gemini_api_key);

    // Test Gemini connectivity (warn but don't block startup)
    match gemini_client.test_connection().await {
        Ok(()) => tracing::info!("Gemini API connection verified"),
        Err(e) => tracing::warn!(
            "Gemini API test failed: {} — Summarization will not work until this is fixed",
            e
        ),
    }

    // ── 9. Build app state ──
    let app_state = state::AppState {
        db: db_pool,
        search: search_client,
        gemini: gemini_client,
        jwt_secret: config.jwt_secret.clone(),
    };

    // ── 10. Build router ──
    let app = routes::create_router(app_state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    // ── 11. Start server ──
    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Server listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
