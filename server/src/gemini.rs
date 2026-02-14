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
            // Extract the text from the deeply nested response:
            // candidates[0].content.parts[0].text
            println!("Full Gemini Response: {:#?}", json);
            json["candidates"][0]["content"]["parts"][0]["text"]
                .as_str()
                .unwrap_or("Sorry, I couldn't get an answer.")
                .to_string()
        }
        Err(_) => "Error connecting to Gemini".to_string(),
    }
}
