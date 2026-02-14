use serde::Serialize;

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

// The actual logic to talk to Gemini
pub async fn ask_gemini(user_query: &str, api_key: &str) -> String {
    let client = reqwest::Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
        api_key
    );

    let prompt = format!(
        "You are a helpful search engine assistant.
         Provide a concise, factual answer to the following query: {}",
        user_query
    );

    let body = GeminiRequest {
        contents: vec![Content {
            parts: vec![Part { text: prompt }],
        }],
    };

    let response = client.post(url).json(&body).send().await;

    match response {
        Ok(res) => {
            let json: serde_json::Value = res.json().await.unwrap_or_default();

            // We use .get() to navigate safely and .and_then() to chain the checks
            let text_result = json
                .get("candidates")
                .and_then(|c| c.get(0))
                .and_then(|first_candidate| first_candidate.get("content"))
                .and_then(|content| content.get("parts"))
                .and_then(|parts| parts.get(0))
                .and_then(|part| part.get("text"))
                .and_then(|text| text.as_str());

            match text_result {
                Some(text) => text.to_string(),
                None => {
                    println!("JSON structure mismatch: {:?}", json);
                    "Sorry, I couldn't parse the response.".to_string()
                }
            }
        }
        Err(e) => format!("Network error: {}", e),
    }
}
