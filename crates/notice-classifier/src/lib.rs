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
/// Uses rule-based matching for deterministic queries.
/// Everything else goes to search.
pub fn classify(query: &str) -> QueryIntent {
    let trimmed = query.trim();

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

    QueryIntent::Search(trimmed.to_string())
}
