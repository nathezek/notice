use sqlx::PgPool;
use uuid::Uuid;

/// Context extracted from a user's knowledge graph.
/// Used to augment search queries for personalization.
#[derive(Debug, Clone)]
pub struct UserContext {
    /// The user's strongest interest areas (entity names).
    pub top_interests: Vec<WeightedTerm>,
    /// Whether the user has any KG data at all.
    pub has_context: bool,
}

#[derive(Debug, Clone)]
pub struct WeightedTerm {
    pub term: String,
    pub weight: f64,
    pub entity_type: String,
}

impl UserContext {
    /// Empty context for anonymous users.
    pub fn anonymous() -> Self {
        Self {
            top_interests: vec![],
            has_context: false,
        }
    }
}

/// Load a user's context from their knowledge graph.
/// Returns the top N entities that represent the user's interests.
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

/// Determine if any query terms overlap with the user's known entities.
/// Returns matching entity types, useful for disambiguation.
///
/// Example: user has "Rust" as a "language" entity.
/// Query contains "rust" → we know they mean the programming language.
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

/// Build an augmented search query by injecting KG context.
///
/// Strategy:
/// - If the user has strong interests that relate to the query, boost those terms.
/// - This helps disambiguate and personalize results.
///
/// Example:
///   Original query: "ownership"
///   User's top interests: ["Rust", "programming", "systems"]
///   Augmented query: "ownership Rust programming"
///   → Meilisearch will rank Rust ownership docs higher
pub fn augment_query(
    original_query: &str,
    context: &UserContext,
    overlapping: &[WeightedTerm],
) -> String {
    if !context.has_context {
        return original_query.to_string();
    }

    let original_lower = original_query.to_lowercase();
    let original_words: Vec<&str> = original_lower.split_whitespace().collect();

    // Collect context terms to inject
    let mut boost_terms: Vec<String> = vec![];

    // If we found overlapping entities (terms the user has searched before),
    // add related high-weight interests as context
    if !overlapping.is_empty() {
        // The user has searched for some of these terms before.
        // Add their top interests as a gentle boost.
        for interest in &context.top_interests {
            let interest_lower = interest.term.to_lowercase();

            // Don't add terms already in the query
            if original_words.contains(&interest_lower.as_str()) {
                continue;
            }

            // Only add high-confidence interests (weight >= 3)
            if interest.weight >= 3.0 {
                boost_terms.push(interest.term.clone());
            }

            // Limit to 3 boost terms to avoid diluting the query
            if boost_terms.len() >= 3 {
                break;
            }
        }
    }

    if boost_terms.is_empty() {
        return original_query.to_string();
    }

    // Append boost terms with lower weight by using optionalWords-style approach
    // Meilisearch doesn't have explicit boosting, but adding terms helps ranking
    let augmented = format!("{} {}", original_query, boost_terms.join(" "));

    tracing::debug!(
        original = original_query,
        augmented = %augmented,
        boost_terms = ?boost_terms,
        "Query augmented with KG context"
    );

    augmented
}
