use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Search API ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub user_id: Option<Uuid>,
    pub session_id: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: Uuid,
    pub url: String,
    pub title: Option<String>,
    pub snippet: String,
    pub score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub query: String,
    pub results: Vec<SearchResult>,
    pub total: usize,
    pub instant_answer: Option<InstantAnswer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstantAnswer {
    pub answer_type: String,
    pub value: String,
}

// ─── Auth API ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: Uuid,
    pub username: String,
}

// ─── Content Submission API ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitUrlRequest {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitUrlResponse {
    pub id: Uuid,
    pub url: String,
    pub status: String,
    pub message: String,
}

// ─── Crawler Internal ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapedPage {
    pub url: String,
    pub title: Option<String>,
    pub text_content: String,
    pub scraped_at: DateTime<Utc>,
}
