use crate::extractor;
use sqlx::PgPool;
use std::collections::HashSet;
use uuid::Uuid;

/// Session context derived from recent search history.
/// Helps disambiguate queries based on what the user was searching for recently.
#[derive(Debug, Clone)]
pub struct SessionContext {
    /// Topics extracted from recent queries.
    pub recent_topics: Vec<String>,
    /// Whether there is meaningful session context.
    pub has_context: bool,
}

impl SessionContext {
    pub fn empty() -> Self {
        Self {
            recent_topics: vec![],
            has_context: false,
        }
    }
}

/// Analyze recent search history to build session context.
/// Looks at queries from the last 30 minutes.
pub async fn build_session_context(
    pool: &PgPool,
    user_id: Option<Uuid>,
    session_id: Option<&str>,
) -> SessionContext {
    // Get recent queries (last 30 minutes)
    let recent_queries = match notice_db::search_history::get_recent_queries(
        pool, user_id, session_id, 30, // minutes
        20, // max queries to analyze
    )
    .await
    {
        Ok(q) => q,
        Err(e) => {
            tracing::debug!(error = %e, "Failed to fetch session history");
            return SessionContext::empty();
        }
    };

    if recent_queries.is_empty() {
        return SessionContext::empty();
    }

    // Extract entities from all recent queries
    let mut topics: HashSet<String> = HashSet::new();

    for query in &recent_queries {
        let entities = extractor::extract_entities(query);
        for entity in entities {
            topics.insert(entity.name);
        }
    }

    let recent_topics: Vec<String> = topics.into_iter().collect();

    tracing::debug!(
        recent_queries = recent_queries.len(),
        topics = ?recent_topics,
        "Session context built"
    );

    SessionContext {
        has_context: !recent_topics.is_empty(),
        recent_topics,
    }
}

/// Use session context to suggest disambiguation terms.
/// If the current query contains ambiguous terms that also appear
/// in the session context, we can infer the user's intent.
///
/// Example:
///   Session queries: ["rust traits", "rust borrowing"]
///   Current query: "ownership"
///   Session topics: ["rust", "traits", "borrowing"]
///   → "rust" is a strong session signal, add it as context
pub fn get_session_boost_terms(current_query: &str, session: &SessionContext) -> Vec<String> {
    if !session.has_context {
        return vec![];
    }

    let current_entities = extractor::extract_entities(current_query);
    let current_names: HashSet<String> = current_entities.iter().map(|e| e.name.clone()).collect();

    // Find session topics NOT already in the current query
    // that appear frequently (proxy: just return all non-overlapping session topics)
    let mut boost: Vec<String> = vec![];

    for topic in &session.recent_topics {
        if !current_names.contains(topic) {
            boost.push(topic.clone());
        }
        if boost.len() >= 2 {
            break;
        }
    }

    boost
}
