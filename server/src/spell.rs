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
    // Skip correction for very short queries and queries that look like
    // math/currency (numbers + symbols) to avoid mangling them
    if query.len() <= 2 || query.chars().next().map_or(false, |c| c.is_ascii_digit()) {
        return None;
    }

    let checker = spellchecker();

    // lookup_compound handles multi-word corrections in one pass
    let suggestions = checker.lookup_compound(query, 2);

    if let Some(suggestion) = suggestions.first() {
        let corrected = &suggestion.term;
        if corrected.to_lowercase() != query.to_lowercase().trim() {
            return Some(corrected.clone());
        }
    }

    None
}
