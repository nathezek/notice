use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct PageData {
    pub url: String,
    pub title: String,
    pub raw_html: String,
    pub cleaned_text: String,
    pub summary: Option<String>,
    pub crawled_at: NaiveDateTime,
}

pub type DbPool = Pool<Postgres>;

pub async fn init_db(database_url: &str) -> sqlx::Result<DbPool> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;
    
    // Create the pages table if it doesn't exist.
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS pages (
            url TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            raw_html TEXT NOT NULL,
            cleaned_text TEXT NOT NULL,
            summary TEXT,
            crawled_at TIMESTAMP NOT NULL DEFAULT NOW()
        );
        "#
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}

pub async fn insert_page(pool: &DbPool, page: &PageData) -> sqlx::Result<()> {
    // Insert or update on conflict (if the vault already has this URL, we overwrite it)
    sqlx::query(
        r#"
        INSERT INTO pages (url, title, raw_html, cleaned_text, summary, crawled_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (url) DO UPDATE 
        SET title = EXCLUDED.title,
            raw_html = EXCLUDED.raw_html,
            cleaned_text = EXCLUDED.cleaned_text,
            summary = EXCLUDED.summary,
            crawled_at = EXCLUDED.crawled_at;
        "#
    )
    .bind(&page.url)
    .bind(&page.title)
    .bind(&page.raw_html)
    .bind(&page.cleaned_text)
    .bind(&page.summary)
    .bind(&page.crawled_at)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_page(pool: &DbPool, url: &str) -> sqlx::Result<Option<PageData>> {
    let row = sqlx::query_as::<_, PageData>(
        r#"
        SELECT url, title, raw_html, cleaned_text, summary, crawled_at
        FROM pages
        WHERE url = $1
        "#
    )
    .bind(url)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

