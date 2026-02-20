use std::sync::OnceLock;
use regex::Regex;

// --- Compiled once at startup ---

fn math_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        // Symbolic: "2 + 2", "sqrt(9)", "(100*3)/4"
        // Natural: "square root of 9", "5 plus 3", "what is 10 divided by 2", "calculate 50 mod 7"
        Regex::new(concat!(
            r"(?i)^\s*[\d\s\.\+\-\*\/\^\%\(\)]+$",
            r"|^\s*(sqrt|sin|cos|tan|log|ln|abs|ceil|floor)\s*\(",
            r"|\b(square\s+root\s+of|cube\s+root\s+of)\b",
            r"|\b\d+(\.\d+)?\s*(plus|minus|times|multiplied\s+by|divided\s+by|mod|to\s+the\s+power\s+of)\s*\d+",
            r"|^\s*(what\s+is|calculate|compute|evaluate|what'?s)\s+[\d\s\+\-\*\/\^\%\.\(\)]+$"
        ))
        .unwrap()
    })
}

fn unit_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        // e.g. "5 km to miles", "100 kg in lbs", "72°F to Celsius"
        Regex::new(
            r"(?i)\b\d+(\.\d+)?\s*(km|mi|miles|m|meters|ft|feet|inches|in|cm|mm|kg|lbs|pounds|g|grams|oz|ounces|l|liters|ml|gal|gallons|km\/h|mph|m\/s|°c|°f|celsius|fahrenheit|kelvin|k)\b.{0,10}\b(to|in|into|as)\b",
        )
        .unwrap()
    })
}

fn currency_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        // Requires uppercase ISO-4217 codes (USD, ETB, EUR) — no (?i) on code groups
        // so unit abbreviations like mph/kph won't accidentally match
        Regex::new(
            r"\b\d+(?:\.\d+)?\s*([A-Z]{3})\b.{0,15}\b(?i:to|in|into|as)\b\s*([A-Z]{3})\b",
        )
        .unwrap()
    })
}

// -------------------------------------

#[derive(Debug, PartialEq)]
pub enum QueryType {
    Math,
    UnitConversion,
    CurrencyConversion,
    General,
}

pub fn classify(query: &str) -> QueryType {
    // Unit must be checked before currency — unit abbreviations (mph, kph, etc.)
    // are 3 letters and would otherwise match the currency pattern first.
    if unit_re().is_match(query) {
        return QueryType::UnitConversion;
    }
    // Currency requires uppercase codes so it won't accidentally match unit queries
    if currency_re().is_match(query) {
        return QueryType::CurrencyConversion;
    }
    if math_re().is_match(query) {
        return QueryType::Math;
    }
    QueryType::General
}
