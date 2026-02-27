use serde::{Deserialize, Serialize};

const GEMINI_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta/models";
const DEFAULT_MODEL: &str = "gemini-2.5-flash";

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
    error: Option<GeminiError>,
}

#[derive(Deserialize)]
struct GeminiError {
    message: String,
    #[allow(dead_code)]
    code: Option<i32>,
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

// ─── Top-level error response (different shape) ───

#[derive(Deserialize)]
struct GeminiErrorResponse {
    error: GeminiErrorDetail,
}

#[derive(Deserialize)]
struct GeminiErrorDetail {
    message: String,
    #[allow(dead_code)]
    code: Option<i32>,
    #[allow(dead_code)]
    status: Option<String>,
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

    /// Test the connection to Gemini with a minimal prompt.
    /// Returns Ok(()) if the API key is valid and the model responds.
    pub async fn test_connection(&self) -> Result<(), notice_core::Error> {
        let response = self.generate("Respond with only the word: OK").await?;
        if response.is_empty() {
            return Err(notice_core::Error::Ai(
                "Gemini returned empty response".into(),
            ));
        }
        Ok(())
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

        let response =
            self.http.post(&url).json(&body).send().await.map_err(|e| {
                notice_core::Error::Ai(format!("Failed to reach Gemini API: {}", e))
            })?;

        let status = response.status();

        if !status.is_success() {
            let body_text = response.text().await.unwrap_or_default();

            // Try to parse structured error
            let error_msg = if let Ok(err) = serde_json::from_str::<GeminiErrorResponse>(&body_text)
            {
                format!("Gemini API error (HTTP {}): {}", status, err.error.message)
            } else {
                format!("Gemini API error (HTTP {}): {}", status, body_text)
            };

            tracing::error!("{}", error_msg);
            return Err(notice_core::Error::Ai(error_msg));
        }

        let body_text = response.text().await.map_err(|e| {
            notice_core::Error::Ai(format!("Failed to read Gemini response: {}", e))
        })?;

        let gemini_response: GeminiResponse = serde_json::from_str(&body_text).map_err(|e| {
            notice_core::Error::Ai(format!(
                "Failed to parse Gemini response: {} — body: {}",
                e,
                &body_text[..body_text.len().min(500)]
            ))
        })?;

        // Check for inline error
        if let Some(err) = gemini_response.error {
            return Err(notice_core::Error::Ai(format!(
                "Gemini API returned error: {}",
                err.message
            )));
        }

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

        if text.is_empty() {
            tracing::warn!(
                "Gemini returned empty text for prompt (first 100 chars): {}",
                &prompt[..prompt.len().min(100)]
            );
        }

        Ok(text)
    }

    /// Summarize web content.
    pub async fn summarize(&self, content: &str) -> Result<String, notice_core::Error> {
        let prompt = format!(
            "Summarize the following web page content in 2-4 concise sentences. \
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

    /// Generate an answer to a query based on retrieved document snippets (RAG).
    pub async fn answer_query(
        &self,
        query: &str,
        contexts: &[String],
    ) -> Result<String, notice_core::Error> {
        if contexts.is_empty() {
            return Ok("No relevant context found to answer this query.".to_string());
        }

        let combined_context = contexts
            .iter()
            .enumerate()
            .map(|(i, c)| format!("Source [{}]:\n{}\n", i + 1, c))
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            "You are Notice, an intelligent search assistant. \
             Answer the user's query based ONLY on the provided source snippets. \
             If the sources do not contain the answer, say that you don't have enough information. \
             Keep your answer professional, concise (2-4 sentences), and use markdown for formatting.\n\n\
             RELEVANT SOURCES:\n{}\n\n\
             USER QUERY: \"{}\"\n\n\
             NOTICE ANSWER:",
            combined_context, query
        );

        self.generate(&prompt).await
    }
}
