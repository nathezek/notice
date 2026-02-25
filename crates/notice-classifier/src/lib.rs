use notice_ai::GeminiClient;
use regex::Regex;

/// What the user wants to do.
#[derive(Debug, Clone)]
pub enum QueryIntent {
    /// Run through the search pipeline
    Search(String),
    /// Evaluate a math expression
    Calculate(String),
    /// Look up a word definition
    Define(String),
    /// Set a timer
    Timer(String),
}

/// Classify a user query into an intent.
/// Tries fast rule-based matching first, falls back to Gemini for ambiguous cases.
pub async fn classify(query: &str, gemini: &GeminiClient) -> QueryIntent {
    let trimmed = query.trim();

    // ── Rule-based (fast path) ──

    // Math expressions: "150 * 6 + 7", "sqrt(144)", "2^10"
    let math_re = Regex::new(r"^[\d\s\+\-\*/\.\(\)\^%]+$").unwrap();
    if math_re.is_match(trimmed) {
        return QueryIntent::Calculate(trimmed.to_string());
    }

    // Definitions: "define entropy", "what does osmosis mean"
    let define_re =
        Regex::new(r"(?i)^(define |what does .+ mean|meaning of |definition of )").unwrap();
    if define_re.is_match(trimmed) {
        return QueryIntent::Define(trimmed.to_string());
    }

    // Timers: "set a timer for 10 minutes", "timer 5m"
    let timer_re = Regex::new(r"(?i)(set .* timer|timer .*(min|sec|hour)|countdown)").unwrap();
    if timer_re.is_match(trimmed) {
        return QueryIntent::Timer(trimmed.to_string());
    }

    // ── Gemini fallback (for ambiguous queries) ──
    // Only called when rules don't match and the query looks non-obvious.
    // For now, default to Search for everything else.
    // We can enable the Gemini fallback when needed:
    //
    // match gemini.classify_intent(trimmed).await {
    //     Ok(category) => match category.trim().to_lowercase().as_str() {
    //         "calculate" => QueryIntent::Calculate(trimmed.to_string()),
    //         "define" => QueryIntent::Define(trimmed.to_string()),
    //         "timer" => QueryIntent::Timer(trimmed.to_string()),
    //         _ => QueryIntent::Search(trimmed.to_string()),
    //     },
    //     Err(_) => QueryIntent::Search(trimmed.to_string()),
    // }

    let _ = gemini; // suppress unused warning until we enable the fallback

    QueryIntent::Search(trimmed.to_string())
}
