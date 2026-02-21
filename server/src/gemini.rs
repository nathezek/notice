use serde::Serialize;

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
    generation_config: GenerationConfig,
}

#[derive(Serialize)]
struct GenerationConfig {
    response_mime_type: String,
}

#[derive(Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize)]
struct Part {
    text: String,
}

const PROMPT_INSTRUCTIONS: &str = "
Instructions:
- Extract and summarize the most important facts. Be concise (2-4 sentences).
- EXTENSIVELY use **bold text** to highlight key names, dates, amounts, and critical concepts.
- Use ### headers only for distinct sections.
- Do not use phrases like 'The text says' or 'According to the source'. Write naturally.";

pub async fn ask_gemini(user_query: &str, api_key: &str, context: Option<&str>, urls: Vec<String>) -> String {
    let client = reqwest::Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
        api_key
    );

    let prompt = if let Some(ctx) = context.filter(|c| !c.is_empty()) {
        format!(
            "You are a smart search engine assistant. Answer the query based on the content.
            
            QUERY: '{query}'
            CONTENT: {ctx}
            
            {instructions}

            RETURN JSON:
            {{
                \"title\": \"Title\",
                \"summary\": \"Markdown summary\",
                \"facts\": [ {{ \"label\": \"Key\", \"value\": \"Value\" }} ],
                \"related_topics\": [ \"Topic\" ],
                \"websites\": [ {{ \"url\": \"url\", \"title\": \"title\" }} ]
            }}",
            query = user_query,
            ctx = ctx,
            instructions = PROMPT_INSTRUCTIONS
        )
    } else {
        format!(
            "You are a smart search engine. Answer concisely.
            
            QUERY: '{query}'
            
            {instructions}

            RETURN JSON:
            {{
                \"title\": \"Title\",
                \"summary\": \"Markdown summary\",
                \"facts\": [ {{ \"label\": \"Key\", \"value\": \"Value\" }} ],
                \"related_topics\": [ \"Topic\" ],
                \"widgets\": [ {{ \"type\": \"map\", \"query\": \"Location\" }} ]
            }}",
            query = user_query,
            instructions = PROMPT_INSTRUCTIONS
        )
    };

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
            let status = res.status();
            if !status.is_success() {
                if status.as_u16() == 429 {
                    return r#"{"error": "Rate limit exceeded. Try again in 60s."}"#.to_string();
                }
            }
            let raw_text = res.text().await.unwrap_or_default();
            let json: serde_json::Value = serde_json::from_str(&raw_text).unwrap_or_default();
            
            let extracted_text = json["candidates"][0]["content"]["parts"][0]["text"]
                .as_str()
                .unwrap_or("{\"error\": \"Empty response\"}");
            
            let clean_text = extracted_text
                .trim()
                .trim_start_matches("```json")
                .trim_start_matches("```")
                .trim_end_matches("```");

            let mut final_json: serde_json::Value = serde_json::from_str(clean_text).unwrap_or(serde_json::json!({
                "title": "Search Error",
                "summary": "AI parsing failed."
            }));

            if !urls.is_empty() {
                let websites_arr: Vec<serde_json::Value> = urls.into_iter().map(|url| {
                    serde_json::json!({ "url": url, "title": url })
                }).collect();
                final_json["websites"] = serde_json::Value::Array(websites_arr);
            }

            final_json.to_string()
        }
        Err(e) => format!("{{\"error\": \"Network error: {}\"}}", e),
    }
}

pub async fn summarize_page(api_key: &str, title: &str, text: &str) -> String {
    let client = reqwest::Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
        api_key
    );

    let prompt = format!(
        "You are an expert summarizer. Summarize this page.
        
        TITLE: {title}
        CONTENT: {text}

        {instructions}

        - Output the summary as a single block of text.
        - Maximum 4 sentences.",
        title = title,
        text = text,
        instructions = PROMPT_INSTRUCTIONS
    );

    let body = GeminiRequest {
        contents: vec![Content {
            parts: vec![Part { text: prompt }],
        }],
        generation_config: GenerationConfig {
            response_mime_type: "text/plain".to_string(),
        },
    };

    let mut attempts = 0;
    while attempts < 5 {
        match client.post(&url).json(&body).send().await {
            Ok(res) if res.status().is_success() => {
                let raw_text = res.text().await.unwrap_or_default();
                let json: serde_json::Value = serde_json::from_str(&raw_text).unwrap_or_default();
                
                return json["candidates"][0]["content"]["parts"][0]["text"]
                    .as_str()
                    .unwrap_or("Failed to generate summary.")
                    .trim()
                    .to_string();
            }
            Ok(res) if res.status().as_u16() == 429 => {
                attempts += 1;
                let wait_secs = 5 + attempts;
                tracing::warn!("Gemini 429 Rate Limit hit. Retrying in {}s (Attempt {}/5)", wait_secs, attempts);
                tokio::time::sleep(std::time::Duration::from_secs(wait_secs)).await;
            }
            Ok(res) => {
                tracing::error!("Gemini summarize API Error Status: {}", res.status());
                return "Summary generation failed due to API error.".to_string();
            }
            Err(e) => {
                tracing::error!("Gemini summarize Network Error: {}", e);
                return "Summary generation failed due to network error.".to_string();
            }
        }
    }
    "Summary generation failed due to persistent rate limiting.".to_string()
}
