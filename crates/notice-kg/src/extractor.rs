use std::collections::HashSet;

/// An extracted entity from a query.
#[derive(Debug, Clone)]
pub struct ExtractedEntity {
    pub name: String,
    pub entity_type: String,
}

/// Extract meaningful entities from a search query.
/// Uses stop word removal, normalization, and bigram detection.
pub fn extract_entities(query: &str) -> Vec<ExtractedEntity> {
    let normalized = query.to_lowercase();
    let words: Vec<&str> = normalized
        .split_whitespace()
        .filter(|w| w.len() >= 2)
        .collect();

    if words.is_empty() {
        return vec![];
    }

    let stop_words = stop_words();
    let mut entities: Vec<ExtractedEntity> = vec![];
    let mut seen = HashSet::new();

    // Step 1: Extract bigrams (compound terms)
    // "programming language" is more meaningful than "programming" + "language" separately
    for window in words.windows(2) {
        let bigram = format!("{} {}", window[0], window[1]);
        if !stop_words.contains(window[0])
            && !stop_words.contains(window[1])
            && is_meaningful_bigram(&bigram)
        {
            if seen.insert(bigram.clone()) {
                entities.push(ExtractedEntity {
                    name: bigram,
                    entity_type: "concept".to_string(),
                });
            }
        }
    }

    // Step 2: Extract individual meaningful words
    for word in &words {
        if stop_words.contains(*word) {
            continue;
        }

        let word_str = word.to_string();
        if seen.insert(word_str.clone()) {
            let entity_type = classify_word(&word_str);
            entities.push(ExtractedEntity {
                name: word_str,
                entity_type,
            });
        }
    }

    entities
}

/// Classify what type of entity a word likely is.
fn classify_word(word: &str) -> String {
    // Known programming languages
    let languages = [
        "rust",
        "python",
        "javascript",
        "typescript",
        "java",
        "go",
        "golang",
        "ruby",
        "php",
        "swift",
        "kotlin",
        "scala",
        "haskell",
        "elixir",
        "clojure",
        "lua",
        "perl",
        "dart",
        "zig",
        "nim",
        "c",
        "cpp",
    ];

    // Known technologies / tools
    let technologies = [
        "postgresql",
        "postgres",
        "mysql",
        "mongodb",
        "redis",
        "docker",
        "kubernetes",
        "nginx",
        "apache",
        "linux",
        "windows",
        "macos",
        "git",
        "github",
        "gitlab",
        "aws",
        "azure",
        "gcp",
        "node",
        "react",
        "vue",
        "angular",
        "svelte",
        "nextjs",
        "django",
        "flask",
        "rails",
        "spring",
        "express",
        "axum",
        "actix",
        "meilisearch",
        "elasticsearch",
        "solr",
        "kafka",
        "rabbitmq",
        "graphql",
        "rest",
        "grpc",
        "wasm",
        "webassembly",
    ];

    // Known CS concepts
    let concepts = [
        "algorithm",
        "data",
        "structure",
        "pattern",
        "design",
        "architecture",
        "api",
        "database",
        "server",
        "client",
        "frontend",
        "backend",
        "fullstack",
        "devops",
        "security",
        "cryptography",
        "encryption",
        "authentication",
        "authorization",
        "concurrency",
        "parallelism",
        "async",
        "thread",
        "process",
        "memory",
        "allocation",
        "garbage",
        "collection",
        "ownership",
        "borrowing",
        "lifetime",
        "trait",
        "interface",
        "abstract",
        "polymorphism",
        "inheritance",
        "composition",
        "functional",
        "imperative",
        "declarative",
        "compiler",
        "interpreter",
        "runtime",
        "virtual",
        "machine",
        "container",
        "microservice",
    ];

    if languages.contains(&word) {
        "language".to_string()
    } else if technologies.contains(&word) {
        "technology".to_string()
    } else if concepts.contains(&word) {
        "concept".to_string()
    } else {
        "topic".to_string()
    }
}

/// Check if a bigram is likely a meaningful compound term.
fn is_meaningful_bigram(bigram: &str) -> bool {
    let known_bigrams = [
        "programming language",
        "data structure",
        "machine learning",
        "deep learning",
        "artificial intelligence",
        "operating system",
        "web development",
        "open source",
        "version control",
        "design pattern",
        "type system",
        "garbage collection",
        "memory management",
        "error handling",
        "pattern matching",
        "functional programming",
        "object oriented",
        "linked list",
        "binary tree",
        "hash map",
        "hash table",
        "stack overflow",
        "buffer overflow",
        "null pointer",
        "race condition",
        "dead lock",
        "load balancer",
        "search engine",
        "knowledge graph",
        "neural network",
        "natural language",
        "command line",
        "file system",
        "system call",
        "virtual machine",
        "smart pointer",
        "move semantics",
        "zero cost",
        "compile time",
        "run time",
    ];

    known_bigrams.contains(&bigram)
}

/// Common English stop words that don't carry topical meaning.
fn stop_words() -> HashSet<&'static str> {
    [
        "a", "an", "the", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had",
        "do", "does", "did", "will", "would", "shall", "should", "may", "might", "must", "can",
        "could", "am", "it", "its", "i", "me", "my", "we", "our", "you", "your", "he", "him",
        "his", "she", "her", "they", "them", "their", "this", "that", "these", "those", "what",
        "which", "who", "whom", "when", "where", "why", "how", "all", "each", "every", "both",
        "few", "more", "most", "other", "some", "such", "no", "not", "only", "own", "same", "so",
        "than", "too", "very", "just", "because", "as", "until", "while", "of", "at", "by", "for",
        "with", "about", "against", "between", "through", "during", "before", "after", "above",
        "below", "to", "from", "up", "down", "in", "out", "on", "off", "over", "under", "again",
        "further", "then", "once", "here", "there", "and", "but", "or", "nor", "if", "else",
        "also", "any",
    ]
    .into_iter()
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_simple_query() {
        let entities = extract_entities("rust ownership rules");
        let names: Vec<&str> = entities.iter().map(|e| e.name.as_str()).collect();

        assert!(names.contains(&"rust"));
        assert!(names.contains(&"ownership"));
        assert!(names.contains(&"rules"));
    }

    #[test]
    fn extract_bigrams() {
        let entities = extract_entities("rust programming language features");
        let names: Vec<&str> = entities.iter().map(|e| e.name.as_str()).collect();

        assert!(names.contains(&"programming language"));
        assert!(names.contains(&"rust"));
    }

    #[test]
    fn filter_stop_words() {
        let entities = extract_entities("what is the rust programming language");
        let names: Vec<&str> = entities.iter().map(|e| e.name.as_str()).collect();

        assert!(names.contains(&"rust"));
        assert!(names.contains(&"programming language"));
        assert!(!names.contains(&"what"));
        assert!(!names.contains(&"is"));
        assert!(!names.contains(&"the"));
    }

    #[test]
    fn classify_types() {
        let entities = extract_entities("rust postgresql docker");

        let rust = entities.iter().find(|e| e.name == "rust").unwrap();
        assert_eq!(rust.entity_type, "language");

        let pg = entities.iter().find(|e| e.name == "postgresql").unwrap();
        assert_eq!(pg.entity_type, "technology");

        let docker = entities.iter().find(|e| e.name == "docker").unwrap();
        assert_eq!(docker.entity_type, "technology");
    }

    #[test]
    fn empty_query() {
        let entities = extract_entities("");
        assert!(entities.is_empty());
    }

    #[test]
    fn all_stop_words() {
        let entities = extract_entities("what is the in a");
        assert!(entities.is_empty());
    }
}
