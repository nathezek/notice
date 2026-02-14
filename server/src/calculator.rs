use regex::Regex;

pub fn extract_conversion(query: &str) -> Option<(f64, &str, &str)> {
    let pattern = r"(\d+)\s*(kg|lbs|g|oz)\s+to\s+(kg|lbs|g|oz)";
    let re = Regex::new(pattern).unwrap();

    if let Some(caps) = re.captures(query) {
        let amount = caps[1].parse::<f64>().unwrap_or(0.0);
        let from = caps.get(2)?.as_str();
        let to = caps.get(3)?.as_str();
        Some((amount, from, to))
    } else {
        None
    }
}

pub fn calculate(amount: f64, from: &str, to: &str) -> f64 {
    match (from, to) {
        ("kg", "lbs") => amount * 2.20462,
        ("lbs", "kg") => amount / 2.20462,
        ("g", "oz") => amount / 28.3495,
        ("oz", "g") => amount * 28.3495,
        _ => amount,
    }
}
