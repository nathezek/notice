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

        Goal: Provide a comprehensive, structured answer using the following FLEXIBLE schema.
        Do not force the query into a specific 'type' if it doesn't fit.
        
        RETURN JSON STRUCTURE:
        {{
            \"summary\": \"Markdown text explaining the answer. use bolding, lists, and clear paragraphs.\",
            
            \"facts\": [  // Optional: Key attributes if applicable (e.g. for people, places, events)
                {{ \"label\": \"Born\", \"value\": \"1879\" }},
                {{ \"label\": \"Height\", \"value\": \"5ft 9in\" }}
            ],

            \"related_topics\": [ // Optional: 3-5 related search terms
                \"Topic 1\", \"Topic 2\" 
            ],

            \"widgets\": [ // Optional: Special display blocks if the query warrants it
                // Supported types: 'map', 'image'
                {{ \"type\": \"map\", \"query\": \"Location Name\" }} 
            ]
        }}
        
        Examples:
        - Query: 'Who is Elon Musk?' -> summary: '...', facts: [{{label: 'Born', value: '...'}}, ...], related_topics: ['Tesla', ...]
        - Query: 'How to make cake?' -> summary: 'Markdown recipe...', facts: [], related_topics: ['Baking', ...]
        - Query: 'Paris' -> summary: '...', facts: [{{label: 'Country', value: 'France'}}], widgets: [{{type: 'map', query: 'Paris'}}]
        ",
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
