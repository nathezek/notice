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

pub async fn ask_gemini(user_query: &str, api_key: &str) -> String {
    let client = reqwest::Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
        api_key
    );

    let prompt = format!(
        "You are a smart search engine. Analyze the query and return JSON.
        Query: '{}'

        1. CATEGORIZE the query into one of: 'who', 'what', 'how', 'when', 'where'.
        2. Based on the category, use the EXACT schema below.

        SCENARIO 'who' (Person/Group):
        {{
            \"type\": \"who\",
            \"name\": \"Full Name\",
            \"lifespan\": \"Born - Died (or Present)\",
            \"known_for\": \"One sentence summary of fame\",
            \"achievements\": [\"Major feat 1\", \"Major feat 2\", \"Major feat 3\"]
        }}

        SCENARIO 'how' (Process/Math/Recipe):
        {{
            \"type\": \"how\",
            \"title\": \"Process Name\",
            \"difficulty\": \"Easy/Medium/Hard\",
            \"steps\": [
                {{ \"step\": 1, \"instruction\": \"Do this first\" }},
                {{ \"step\": 2, \"instruction\": \"Then do this\" }}
            ]
        }}

        SCENARIO 'what' (Definition/Concept):
        {{
            \"type\": \"what\",
            \"concept\": \"Concept Name\",
            \"definition\": \"Official definition\",
            \"application\": \"Real-world use case\",
            \"origin\": \"Brief history/origin\"
        }}

        SCENARIO 'when' (Time/Event):
        {{
            \"type\": \"when\",
            \"event\": \"Event Name\",
            \"date\": \"Exact Date/Era\",
            \"significance\": \"Why it matters today\",
            \"timeline\": [\"Pre-event\", \"During event\", \"Post-event\"]
        }}

        SCENARIO 'where' (Place/Location):
        {{
            \"type\": \"where\",
            \"location\": \"Location Name\",
            \"region\": \"Country/Continent\",
            \"facts\": [\"Fact 1\", \"Fact 2\"],
            \"climate\": \"Climate/Vibe\"
        }}",
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
            json["candidates"][0]["content"]["parts"][0]["text"]
                .as_str()
                .unwrap_or("{}")
                .to_string()
        }
        Err(_) => "{\"error\": \"offline\"}".to_string(),
    }
}
