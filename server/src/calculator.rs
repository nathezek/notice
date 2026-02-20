use regex::Regex;
use std::sync::OnceLock;

// ---- Math Evaluation ----

/// Maps number words to digit strings.
fn words_to_digits(s: &str) -> String {
    let pairs: &[(&str, &str)] = &[
        ("zero", "0"), ("one", "1"), ("two", "2"), ("three", "3"),
        ("four", "4"), ("five", "5"), ("six", "6"), ("seven", "7"),
        ("eight", "8"), ("nine", "9"), ("ten", "10"), ("eleven", "11"),
        ("twelve", "12"), ("thirteen", "13"), ("fourteen", "14"),
        ("fifteen", "15"), ("sixteen", "16"), ("seventeen", "17"),
        ("eighteen", "18"), ("nineteen", "19"), ("twenty", "20"),
        ("thirty", "30"), ("forty", "40"), ("fifty", "50"),
        ("sixty", "60"), ("seventy", "70"), ("eighty", "80"),
        ("ninety", "90"), ("hundred", "100"), ("thousand", "1000"),
        ("million", "1000000"),
    ];
    let mut result = s.to_string();
    // Replace whole-word occurrences only (surround with \b)
    for (word, digit) in pairs {
        let re = regex::Regex::new(&format!(r"(?i)\b{}\b", word)).unwrap();
        result = re.replace_all(&result, *digit).to_string();
    }
    result
}

/// Translate natural language math phrases into meval-compatible expressions.
pub fn normalize_math(query: &str) -> String {
    let q = query.trim().to_lowercase();

    // Strip conversational prefixes
    let q = regex::Regex::new(r"(?i)^(what\s+is|what's|calculate|compute|evaluate|find|solve)\s+")
        .unwrap()
        .replace(&q, "")
        .to_string();

    // Convert number words to digits first (so "nine" → "9" before further processing)
    let q = words_to_digits(&q);

    // --- Function aliases with "of" or space: "sqrt of X", "log of X", "sin 45" ---
    let fn_aliases: &[(&str, &str)] = &[
        (r"(?i)\bsquare\s+root\s+of\s+", "sqrt("),
        (r"(?i)\bcube\s+root\s+of\s+", "cbrt("),
        (r"(?i)\bsqrt\s+of\s+", "sqrt("),
        (r"(?i)\bsqrt\s+", "sqrt("),
        (r"(?i)\bcbrt\s+of\s+", "cbrt("),
        (r"(?i)\bln\s+of\s+", "ln("),
        (r"(?i)\bln\s+", "ln("),
        (r"(?i)\blog\s+of\s+", "log("),
        (r"(?i)\blog\s+", "log("),
        (r"(?i)\bsin\s+of\s+", "sin("),
        (r"(?i)\bsin\s+", "sin("),
        (r"(?i)\bcos\s+of\s+", "cos("),
        (r"(?i)\bcos\s+", "cos("),
        (r"(?i)\btan\s+of\s+", "tan("),
        (r"(?i)\btan\s+", "tan("),
        (r"(?i)\babs\s+of\s+", "abs("),
        (r"(?i)\bceil\s+of\s+", "ceil("),
        (r"(?i)\bfloor\s+of\s+", "floor("),
    ];

    let mut q = q;
    for (pattern, replacement) in fn_aliases {
        // Match pattern followed by a number (to auto-close the paren)
        let open_re = regex::Regex::new(&format!(r"{}\s*(\d+(?:\.\d+)?)", pattern)).unwrap();
        q = open_re.replace_all(&q, format!("{}$1)", replacement)).to_string();
    }

    // Natural language operators (after number-word conversion)
    let q = q
        .replace(" plus ", "+")
        .replace(" minus ", "-")
        .replace(" times ", "*")
        .replace(" multiplied by ", "*")
        .replace(" divided by ", "/")
        .replace(" over ", "/")
        .replace(" mod ", "%")
        .replace(" modulo ", "%")
        .replace(" to the power of ", "^")
        .replace(" to the power ", "^")
        .replace(" raised to ", "^")
        .replace(" squared", "^2")
        .replace(" cubed", "^3")
        .replace("^2nd power", "^2")
        .replace("^3rd power", "^3");

    // "X to the Nth power" → "X^N"
    let q = regex::Regex::new(r"(\d+(?:\.\d+)?)\s*to the\s+(\d+)\s*(st|nd|rd|th)?\s*power")
        .unwrap()
        .replace_all(&q, "$1^$2")
        .to_string();

    q.trim().to_string()
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
