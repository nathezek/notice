use symspell::{AsciiStringStrategy, SymSpell};
use std::sync::OnceLock;

// Embedded dictionary at compile time — no file I/O at runtime
static DICT: &str = include_str!("../data/frequency_dictionary_en.txt");

fn spellchecker() -> &'static SymSpell<AsciiStringStrategy> {
    static INSTANCE: OnceLock<SymSpell<AsciiStringStrategy>> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        let mut symspell: SymSpell<AsciiStringStrategy> = SymSpell::default();
        for line in DICT.lines() {
            // Each line: "word count" separated by whitespace
            symspell.load_dictionary_line(&line.replace('\t', " "), 0, 1, " ");
        }
        symspell
    })
}

/// Returns the corrected query if corrections were made, otherwise None.
pub fn correct_query(query: &str) -> Option<String> {
    let trimmed = query.trim();

    // Skip correction for very short queries, mathematical/currency queries,
    // queries with quotes (exact matches), or queries that contain uppercase letters
    if trimmed.len() <= 2 
        || trimmed.chars().next().map_or(false, |c| c.is_ascii_digit())
        || trimmed.contains('"')
        || trimmed.contains('\'')
    {
        return None;
    }

    // Strip trailing punctuation for the lookup, but keep it to re-attach later
    let mut clean_query = trimmed.to_string();
    let mut trailing_punct = "";
    if trimmed.ends_with('?') {
        clean_query.pop();
        trailing_punct = "?";
    } else if trimmed.ends_with('.') {
        clean_query.pop();
        trailing_punct = ".";
    }

    let has_inner_caps = clean_query.chars().skip(1).any(|c| c.is_ascii_uppercase());
    if has_inner_caps {
        return None;
    }

    let checker = spellchecker();

    // lookup_compound handles multi-word corrections in one pass
    let suggestions = checker.lookup_compound(&clean_query.to_lowercase(), 2);

    if let Some(suggestion) = suggestions.first() {
        let corrected = suggestion.term.trim();
        let clean_lower = clean_query.to_lowercase();
        let clean_lower = clean_lower.trim();
        
        // If the correction isn't identical to the lowercase input
        if corrected.to_lowercase() != clean_lower {
            // Restore original capitalization for the first letter if needed
            let mut final_correction = corrected.to_string();
            if let Some(first_char) = trimmed.chars().next() {
                if first_char.is_ascii_uppercase() {
                    let mut c = final_correction.chars();
                    if let Some(f) = c.next() {
                        final_correction = f.to_uppercase().collect::<String>() + c.as_str();
                    }
                }
            }
            
            // Re-attach punctuation
            final_correction.push_str(trailing_punct);
            return Some(final_correction);
        }
    }

    None
}
