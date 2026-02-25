use serde::{Deserialize, Serialize};

const GEMINI_BASE_URL: &str =
    "https://generativelanguage.googleapis.com/v1beta/models";
const DEFAULT_MODEL: &str = "gemini-2.0-flash";

/// Client for the Gemini API.
#[derive(Clone)]
pub struct GeminiClient {
    http: reqwest::Client,
    api_key: String,
    model: String,
}

// ─── Request / Response types for the Gemini REST API ───

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
}

#[derive(Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize)]
struct Part {
    text: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<Candidate>>,
}

#[derive(Deserialize)]
struct Candidate {
    content: CandidateContent,
}

#[derive(Deserialize)]
struct CandidateContent {
    parts: Vec<CandidatePart>,
}

#[derive(Deserialize)]
struct CandidatePart {
    text: String,
}

// ─── Implementation ───

impl GeminiClient {
    pub fn new(api_key: &str) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_key: api_key.to_string(),
            model: DEFAULT_MODEL.to_string(),
        }
    }

    /// Send a prompt to Gemini and return the text response.
    async fn generate(&self, prompt: &str) -> Result<String, notice_core::Error> {
        let url = format!(
            "{}/{}:generateContent?key={}",
            GEMINI_BASE_URL, self.model, self.api_key
        );

        let body = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part {
                    text: prompt.to_string(),
                }],
            }],
        };

        let response = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| notice_core::Error::Ai(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(notice_core::Error::Ai(format!(
                "Gemini API returned {}: {}",
                status, body
            )));
        }

        let gemini_response: GeminiResponse = response
            .json()
            .await
            .map_err(|e| notice_core::Error::Ai(e.to_string()))?;

        let text = gemini_response
            .candidates
            .and_then(|c| c.into_iter().next())
            .map(|c| {
                c.content
                    .parts
                    .into_iter()
                    .map(|p| p.text)
                    .collect::<Vec<_>>()
                    .join("")
            })
            .unwrap_or_default();

        Ok(text)
    }

    /// Summarize a piece of web content.
    pub async fn summarize(&self, content: &str) -> Result<String, notice_core::Error> {
        let prompt = format!(
            "Summarize the following web page content in 2-3 concise sentences. \
             Focus on the key information. Do not include any preamble.\n\n{}",
            content
        );
        self.generate(&prompt).await
    }

    /// Classify a query's intent (fallback when rules don't match).
    pub async fn classify_intent(&self, query: &str) -> Result<String, notice_core::Error> {
        let prompt = format!(
            "Classify the following user search query into exactly one category.\n\
             Categories: search, calculate, define, timer, convert\n\
             Respond with ONLY the category name, nothing else.\n\n\
             Query: \"{}\"",
            query
        );
        self.generate(&prompt).await
    }
}
