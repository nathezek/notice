use sqlx::PgPool;
use uuid::Uuid;

/// Context extracted from a user's knowledge graph.
#[derive(Debug, Clone)]
pub struct UserContext {
    pub top_interests: Vec<WeightedTerm>,
    pub has_context: bool,
}

#[derive(Debug, Clone)]
pub struct WeightedTerm {
    pub term: String,
    pub weight: f64,
    pub entity_type: String,
}

impl UserContext {
    pub fn anonymous() -> Self {
        Self {
            top_interests: vec![],
            has_context: false,
        }
    }
}

/// Load a user's context from their knowledge graph.
pub async fn load_user_context(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
) -> Result<UserContext, notice_core::Error> {
    let entities = notice_db::knowledge_graph::get_top_entities(pool, user_id, limit).await?;

    if entities.is_empty() {
        return Ok(UserContext {
            top_interests: vec![],
            has_context: false,
        });
    }

    let top_interests = entities
        .into_iter()
        .map(|e| WeightedTerm {
            term: e.name,
            weight: e.weight,
            entity_type: e.entity_type,
        })
        .collect();

    Ok(UserContext {
        top_interests,
        has_context: true,
    })
}

/// Find which query terms overlap with the user's existing KG entities.
pub async fn find_overlapping_entities(
    pool: &PgPool,
    user_id: Uuid,
    query_terms: &[String],
) -> Result<Vec<WeightedTerm>, notice_core::Error> {
    if query_terms.is_empty() {
        return Ok(vec![]);
    }

    let entities =
        notice_db::knowledge_graph::find_entities_by_names(pool, user_id, query_terms).await?;

    Ok(entities
        .into_iter()
        .map(|e| WeightedTerm {
            term: e.name,
            weight: e.weight,
            entity_type: e.entity_type,
        })
        .collect())
}

/// Get entities related to query terms through KG relationships.
/// This enables expansion: "ownership" → "rust" (because they're co_searched).
pub async fn get_kg_expansion_terms(
    pool: &PgPool,
    user_id: Uuid,
    query_terms: &[String],
) -> Result<Vec<WeightedTerm>, notice_core::Error> {
    let related = notice_db::knowledge_graph::get_related_entities(
        pool,
        user_id,
        query_terms,
        2.0, // minimum relationship weight
        5,   // max related entities
    )
    .await?;

    Ok(related
        .into_iter()
        .map(|e| WeightedTerm {
            term: e.name,
            weight: e.weight,
            entity_type: e.entity_type,
        })
        .collect())
}

/// Build the final augmented query from all context sources.
///
/// Priority order:
/// 1. KG high-weight interests (weight >= 3.0) — strongest signal
/// 2. KG relationship expansion — related entities
/// 3. Session context — recent search topics
///
/// Limits total boost terms to 3 to avoid diluting the query.
pub fn augment_query(
    original_query: &str,
    kg_context: &UserContext,
    kg_overlapping: &[WeightedTerm],
    kg_expansion: &[WeightedTerm],
    session_boost: &[String],
) -> String {
    let original_lower = original_query.to_lowercase();
    let original_words: std::collections::HashSet<&str> =
        original_lower.split_whitespace().collect();

    let mut boost_terms: Vec<String> = vec![];
    let max_boost = 3;

    // Source 1: KG high-weight interests
    // Only add if the user has overlapping entities (they've searched related terms before)
    if !kg_overlapping.is_empty() {
        for interest in &kg_context.top_interests {
            if boost_terms.len() >= max_boost {
                break;
            }

            let interest_lower = interest.term.to_lowercase();

            // Don't add terms already in the query
            if original_words.contains(interest_lower.as_str()) {
                continue;
            }

            // Only add high-weight interests
            if interest.weight >= 3.0 {
                boost_terms.push(interest.term.clone());
            }
        }
    }

    // Source 2: KG relationship expansion
    for related in kg_expansion {
        if boost_terms.len() >= max_boost {
            break;
        }

        let related_lower = related.term.to_lowercase();
        if original_words.contains(related_lower.as_str()) {
            continue;
        }

        // Don't duplicate
        if boost_terms
            .iter()
            .any(|b| b.to_lowercase() == related_lower)
        {
            continue;
        }

        // Only add if the related entity itself has decent weight
        if related.weight >= 2.0 {
            boost_terms.push(related.term.clone());
        }
    }

    // Source 3: Session context
    for session_term in session_boost {
        if boost_terms.len() >= max_boost {
            break;
        }

        let term_lower = session_term.to_lowercase();
        if original_words.contains(term_lower.as_str()) {
            continue;
        }

        if boost_terms.iter().any(|b| b.to_lowercase() == term_lower) {
            continue;
        }

        boost_terms.push(session_term.clone());
    }

    if boost_terms.is_empty() {
        return original_query.to_string();
    }

    let augmented = format!("{} {}", original_query, boost_terms.join(" "));

    tracing::debug!(
        original = original_query,
        augmented = %augmented,
        kg_boost = ?boost_terms.iter().take(3).collect::<Vec<_>>(),
        "Query augmented"
    );

    augmented
}
