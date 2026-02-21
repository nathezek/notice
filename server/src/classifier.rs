use std::sync::OnceLock;
use regex::Regex;

// --- Compiled once at startup ---

use crate::calculator::normalize_math;

fn math_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(concat!(
            r"(?i)^\s*[\d\s\.\+\-\*\/\^\%\(\)]+$",
            r"|^\s*(sqrt|cbrt|sin|cos|tan|log|ln|abs|ceil|floor)\s*[\(\s]",
            r"|\b(square\s+root|cube\s+root|sqrt|cbrt)\s+(of\s+)?[\w\d]",
            r"|\b(log|ln|sin|cos|tan|abs|ceil|floor)\s+(of\s+)?[\d]",
            r"|\b\d+(\.\d+)?\s*(plus|minus|times|multiplied\s+by|divided\s+by|over|mod|modulo|to\s+the\s+power)",
            r"|\b\d+\s+(squared|cubed)\b",
            r"|^\s*(what\s+is|calculate|compute|evaluate|what'?s|solve)\s+[\d\s\+\-\*\/\^\%\.\(\)]+$",
            r"|^\s*(?i)(calculator|calc)\s*$"
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
        Regex::new(
            concat!(
                r"\b\d+(?:\.\d+)?\s*([A-Z]{3})\b.{0,15}\b(?i:to|in|into|as)\b\s*([A-Z]{3})\b",
                r"|^\s*(?i)(converter|currency converter|exchange rates|exchange rate)\s*$"
            )
        )
        .unwrap()
    })
}

fn timer_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            concat!(
                r"(?i)^(?:set a |start a |create a )?timer (?:for )?\d+(?:\.\d+)?\s*(?:s|sec|secs|second|seconds|m|min|mins|minute|minutes|h|hr|hrs|hour|hours)\b",
                r"|^\s*(?i)(timer|stopwatch)\s*$"
            )
        ).unwrap()
    })
}

// -------------------------------------

#[derive(Debug, PartialEq)]
pub enum QueryType {
    Math,
    UnitConversion,
    CurrencyConversion,
    Timer,
    General,
}

pub fn classify(query: &str) -> QueryType {
    if timer_re().is_match(query) {
        return QueryType::Timer;
    }
    // Unit must be checked before currency — unit abbreviations (mph, kph, etc.)
    // are 3 letters and would otherwise match the currency pattern first.
    if unit_re().is_match(query) {
        return QueryType::UnitConversion;
    }
    // Currency requires uppercase codes so it won't accidentally match unit queries
    if currency_re().is_match(query) {
        return QueryType::CurrencyConversion;
    }
    // Normalize natural language THEN check for math patterns
    // e.g. "sqrt of nine" -> "sqrt(9)" -> matches math_re
    let normalized = normalize_math(query);
    if math_re().is_match(query) || math_re().is_match(&normalized) {
        return QueryType::Math;
    }
    QueryType::General
}
