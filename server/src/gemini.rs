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

pub async fn ask_gemini(user_query: &str, api_key: &str, context: Option<&str>) -> String {
    let client = reqwest::Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
        api_key
    );

    let prompt = if let Some(ctx) = context.filter(|c| !c.is_empty()) {
        format!(
            "You are a smart search engine assistant. Based on the scraped web content below, \
            answer the query with a focused, concise summary.

            QUERY: '{query}'

            SCRAPED CONTENT:
            {ctx}

            --- END OF SCRAPED CONTENT ---

            Instructions:
            - Extract only the most important facts directly relevant to the query. Do NOT reproduce everything.
            - Be concise: 2-4 sentences for the summary, plus optional facts/sections if helpful.
            - Use ### headers only for multiple distinct sections. Prefer flowing prose if 1 topic.
            - Do not say 'According to the sources' or 'The scraped content says' — write naturally.

            RETURN JSON STRUCTURE:
            {{
                \"title\": \"Concise title (required)\",
                \"summary\": \"Markdown summary — highlight key info, use ### Section if needed\",
                \"facts\": [ {{ \"label\": \"Key\", \"value\": \"Value\" }} ],
                \"related_topics\": [ \"Topic 1\", \"Topic 2\" ],
                \"websites\": [ {{ \"url\": \"https://...\", \"title\": \"Page title\" }} ]
            }}",
            query = user_query,
            ctx = ctx
        )
    } else {
        format!(
            "You are a smart search engine. Answer the query with a concise, focused response.

            QUERY: '{}'

            Instructions:
            - Highlight the 3-5 most important facts about this topic.
            - Be concise: 2-4 sentences or bullet points. Do NOT write a Wikipedia article.
            - Use ### headers only if there are multiple distinct sections.

            RETURN JSON STRUCTURE:
            {{
                \"title\": \"Concise title (required)\",
                \"summary\": \"Markdown summary — highlight core info\",
                \"facts\": [ {{ \"label\": \"Key\", \"value\": \"Value\" }} ],
                \"related_topics\": [ \"Topic 1\", \"Topic 2\" ],
                \"widgets\": [ {{ \"type\": \"map\", \"query\": \"Location Name\" }} ]
            }}",
            user_query
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
            if !res.status().is_success() {
                println!("Gemini API Error Status: {}", res.status());
            }
            let raw_text = res.text().await.unwrap_or_default();
            println!("Gemini Raw Response: {}", raw_text);

            let json: serde_json::Value = serde_json::from_str(&raw_text).unwrap_or_default();
            
            let extracted_text = json["candidates"][0]["content"]["parts"][0]["text"]
                .as_str()
                .unwrap_or("{\"error\": \"Empty response from Gemini\"}"); // Return JSON error instead of empty object
            
            println!("Extracted Text: {}", extracted_text);
            
            // formatting check: remove markdown code blocks if present
            let clean_text = extracted_text
                .trim()
                .trim_start_matches("```json")
                .trim_start_matches("```")
                .trim_end_matches("```");

            clean_text.to_string()
        }
        Err(e) => {
            println!("Network Error: {}", e);
            "{\"error\": \"offline\"}".to_string()
        },
    }
}
