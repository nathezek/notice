use regex::Regex;
use std::sync::OnceLock;

// ---- Math Evaluation ----

/// Translate natural language math phrases into meval-compatible expressions.
fn normalize_math(query: &str) -> String {
    let q = query.trim().to_lowercase();

    // Strip conversational prefixes
    let q = regex::Regex::new(r"^(what\s+is|what's|calculate|compute|evaluate|find)\s+")
        .unwrap()
        .replace(&q, "");

    // Natural language operators
    let q = q
        .replace(" plus ", "+")
        .replace(" minus ", "-")
        .replace(" times ", "*")
        .replace(" multiplied by ", "*")
        .replace(" divided by ", "/")
        .replace(" mod ", "%")
        .replace(" to the power of ", "^");

    // "square root of X" -> "sqrt(X)"
    let q = regex::Regex::new(r"square\s+root\s+of\s+(\d+(?:\.\d+)?)")
        .unwrap()
        .replace_all(&q, "sqrt($1)");

    // "cube root of X" -> "X^(1/3)"
    let q = regex::Regex::new(r"cube\s+root\s+of\s+(\d+(?:\.\d+)?)")
        .unwrap()
        .replace_all(&q, "($1)^(1/3)");

    q.to_string()
}

pub fn eval_math(query: &str) -> Result<String, String> {
    let expr = normalize_math(query);
    match meval::eval_str(&expr) {
        Ok(result) => {
            // Format cleanly: no trailing .0 for whole numbers
            if result == result.floor() && result.abs() < 1e15 {
                Ok(format!("{}", result as i64))
            } else {
                Ok(format!("{:.6}", result).trim_end_matches('0').trim_end_matches('.').to_string())
            }
        }
        Err(e) => Err(format!("Cannot evaluate: {}", e)),
    }
}

// ---- Unit Conversion ----

fn unit_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)(\d+(?:\.\d+)?)\s*(km|mi|miles|m|meters|ft|feet|in|inches|cm|mm|kg|lbs|pounds|g|grams|oz|ounces|l|liters|ml|gal|gallons|km/h|mph|m/s|°c|°f|celsius|fahrenheit|kelvin|k)\s*(?:to|in|into|as)\s*(km|mi|miles|m|meters|ft|feet|in|inches|cm|mm|kg|lbs|pounds|g|grams|oz|ounces|l|liters|ml|gal|gallons|km/h|mph|m/s|°c|°f|celsius|fahrenheit|kelvin|k)"
        ).unwrap()
    })
}

pub struct ConversionResult {
    pub amount: f64,
    pub from: String,
    pub to: String,
    pub result: f64,
    pub category: String,
}

pub fn convert_unit(query: &str) -> Option<ConversionResult> {
    let re = unit_re();
    let caps = re.captures(query)?;

    let amount: f64 = caps[1].parse().ok()?;
    let from = caps[2].to_lowercase();
    let to = caps[3].to_lowercase();

    let (result, category) = do_convert(amount, &from, &to)?;

    Some(ConversionResult {
        amount,
        from: caps[2].to_string(),
        to: caps[3].to_string(),
        result,
        category: category.to_string(),
    })
}

fn do_convert(amount: f64, from: &str, to: &str) -> Option<(f64, &'static str)> {
    // Normalise aliases
    let from = normalise(from);
    let to = normalise(to);

    // --- Length --- (base: meters)
    let length = [("km", 1000.0), ("m", 1.0), ("cm", 0.01), ("mm", 0.001),
                  ("mi", 1609.344), ("ft", 0.3048), ("in", 0.0254)];
    if let (Some(f), Some(t)) = (find(&length, from), find(&length, to)) {
        return Some((amount * f / t, "Length"));
    }

    // --- Mass --- (base: grams)
    let mass = [("kg", 1000.0), ("g", 1.0), ("lbs", 453.592), ("oz", 28.3495)];
    if let (Some(f), Some(t)) = (find(&mass, from), find(&mass, to)) {
        return Some((amount * f / t, "Mass"));
    }

    // --- Volume --- (base: liters)
    let volume = [("l", 1.0), ("ml", 0.001), ("gal", 3.78541), ("oz", 0.0295735)];
    if let (Some(f), Some(t)) = (find(&volume, from), find(&volume, to)) {
        return Some((amount * f / t, "Volume"));
    }

    // --- Speed --- (base: m/s)
    let speed = [("m/s", 1.0), ("km/h", 1.0 / 3.6), ("mph", 0.44704)];
    if let (Some(f), Some(t)) = (find(&speed, from), find(&speed, to)) {
        return Some((amount * f / t, "Speed"));
    }

    // --- Temperature (special case) ---
    if (from == "c" || from == "f" || from == "k") && (to == "c" || to == "f" || to == "k") {
        let celsius = match from {
            "c" => amount,
            "f" => (amount - 32.0) * 5.0 / 9.0,
            "k" => amount - 273.15,
            _ => return None,
        };
        let result = match to {
            "c" => celsius,
            "f" => celsius * 9.0 / 5.0 + 32.0,
            "k" => celsius + 273.15,
            _ => return None,
        };
        return Some((result, "Temperature"));
    }

    None
}

fn normalise(unit: &str) -> &str {
    match unit {
        "miles" | "mi" => "mi",
        "meters" => "m",
        "feet" => "ft",
        "inches" => "in",
        "pounds" => "lbs",
        "grams" => "g",
        "ounces" => "oz",
        "liters" => "l",
        "gallons" => "gal",
        "celsius" | "°c" => "c",
        "fahrenheit" | "°f" => "f",
        "kelvin" | "k" => "k",
        _ => unit,
    }
}

fn find(table: &[(&str, f64)], key: &str) -> Option<f64> {
    table.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
}
