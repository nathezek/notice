use serde::Serialize;

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
    generation_config: GenerationConfig, // Enforces JSON output
}

#[derive(Serialize)]
struct GenerationConfig {
    response_mime_type: String, // Set to "application/json"
}

#[derive(Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize)]
struct Part {
    text: String,
}

pub async fn ask_gemini(user_query: &str, api_key: &str) -> String {
    let client = reqwest::Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
        api_key
    );

    // Explicit schema instructions for the AI
    let prompt = format!(
            "You are a Wikipedia-style engine. Categorize the query and respond in JSON.

            Query: {}

            If category is 'WHO' (Person):
            {{ \"type\": \"who\", \"name\": \"\", \"lifespan\": \"\", \"legacy\": \"\", \"quick_facts\": [] }}

            If category is 'HOW' (Process/Math):
            {{ \"type\": \"how\", \"steps\": [{{ \"title\": \"\", \"desc\": \"\" }}], \"difficulty\": \"\" }}

            If category is 'WHAT' (General/Concept):
            {{ \"type\": \"what\", \"definition\": \"\", \"applications\": [], \"history\": \"\" }}",
            user_query
        );

    let body = GeminiRequest {
        contents: vec![Content {
            parts: vec![Part { text: prompt }],
        }],
        generation_config: GenerationConfig {
            response_mime_type: "application/json".to_string(),
        },
    };

    let response = client.post(url).json(&body).send().await;

    match response {
        Ok(res) => {
            let json: serde_json::Value = res.json().await.unwrap_or_default();
            let text_result = json
                .get("candidates")
                .and_then(|c| c.get(0))
                .and_then(|c| c.get("content"))
                .and_then(|c| c.get("parts"))
                .and_then(|p| p.get(0))
                .and_then(|p| p.get("text"))
                .and_then(|t| t.as_str());

            match text_result {
                Some(text) => text.to_string(),
                None => "{\"error\": \"Parsing failed\"}".to_string(),
            }
        }
        Err(e) => format!("{{\"error\": \"Network error: {}\"}}", e),
    }
}
