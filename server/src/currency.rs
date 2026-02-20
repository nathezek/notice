use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;

fn currency_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)(\d+(?:\.\d+)?)\s*([A-Z]{3})\s*(?:to|in|into|as)\s*([A-Z]{3})"
        ).unwrap()
    })
}

#[derive(Deserialize)]
struct FrankfurterResponse {
    rates: HashMap<String, f64>,
}

pub struct CurrencyResult {
    pub amount: f64,
    pub from: String,
    pub to: String,
    pub result: f64,
    pub rate: f64,
}

pub async fn convert_currency(query: &str) -> Result<CurrencyResult, String> {
    let re = currency_re();
    let caps = re.captures(query).ok_or("Could not parse currency query")?;

    let amount: f64 = caps[1].parse().map_err(|_| "Invalid amount")?;
    let from = caps[2].to_uppercase();
    let to = caps[3].to_uppercase();

    let url = format!(
        "https://api.frankfurter.dev/v1/latest?from={}&to={}",
        from, to
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Frankfurter API error: {}", response.status()));
    }

    let data: FrankfurterResponse = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    let rate = data
        .rates
        .get(&to)
        .copied()
        .ok_or_else(|| format!("No rate found for {}", to))?;

    Ok(CurrencyResult {
        amount,
        from,
        to,
        result: amount * rate,
        rate,
    })
}
