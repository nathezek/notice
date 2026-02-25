use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

// ─── Row Types ───

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct KgEntityRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub entity_type: String,
    pub weight: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct KgRelationshipRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub from_entity_id: Uuid,
    pub to_entity_id: Uuid,
    pub relationship_type: String,
    pub weight: f64,
    pub created_at: DateTime<Utc>,
}

// ─── Queries ───

/// Insert a new entity or increment its weight if it already exists.
/// Uses PostgreSQL's ON CONFLICT (upsert).
pub async fn upsert_entity(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
    entity_type: &str,
) -> Result<KgEntityRow, notice_core::Error> {
    sqlx::query_as::<_, KgEntityRow>(
        r#"
        INSERT INTO kg_entities (user_id, name, entity_type, weight)
        VALUES ($1, $2, $3, 1.0)
        ON CONFLICT (user_id, name, entity_type)
        DO UPDATE SET weight = kg_entities.weight + 1.0
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(name)
    .bind(entity_type)
    .fetch_one(pool)
    .await
    .map_err(|e| notice_core::Error::Database(e.to_string()))
}

/// Add a relationship between two entities.
/// If the relationship already exists, increment its weight.
pub async fn upsert_relationship(
    pool: &PgPool,
    user_id: Uuid,
    from_entity_id: Uuid,
    to_entity_id: Uuid,
    relationship_type: &str,
) -> Result<KgRelationshipRow, notice_core::Error> {
    sqlx::query_as::<_, KgRelationshipRow>(
        r#"
        INSERT INTO kg_relationships (user_id, from_entity_id, to_entity_id, relationship_type)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (user_id, from_entity_id, to_entity_id, relationship_type)
        DO UPDATE SET weight = kg_relationships.weight + 1.0
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(from_entity_id)
    .bind(to_entity_id)
    .bind(relationship_type)
    .fetch_one(pool)
    .await
    .map_err(|e| notice_core::Error::Database(e.to_string()))
}

/// Get the top N entities by weight for a user (strongest interests).
pub async fn get_top_entities(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
) -> Result<Vec<KgEntityRow>, notice_core::Error> {
    sqlx::query_as::<_, KgEntityRow>(
        r#"
        SELECT * FROM kg_entities
        WHERE user_id = $1
        ORDER BY weight DESC
        LIMIT $2
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| notice_core::Error::Database(e.to_string()))
}

/// Get context terms: the names of the user's top entities.
/// Used to augment search queries with personalized context.
pub async fn get_context_terms(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
) -> Result<Vec<String>, notice_core::Error> {
    let rows = sqlx::query_as::<_, (String,)>(
        r#"
        SELECT name FROM kg_entities
        WHERE user_id = $1
        ORDER BY weight DESC
        LIMIT $2
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| notice_core::Error::Database(e.to_string()))?;

    Ok(rows.into_iter().map(|(name,)| name).collect())
}

/// Find entities matching a list of names for a specific user.
/// Used for finding overlap between query terms and existing KG entities.
pub async fn find_entities_by_names(
    pool: &PgPool,
    user_id: Uuid,
    names: &[String],
) -> Result<Vec<KgEntityRow>, notice_core::Error> {
    if names.is_empty() {
        return Ok(vec![]);
    }

    // Build a query with ANY() for the names list
    sqlx::query_as::<_, KgEntityRow>(
        r#"
        SELECT * FROM kg_entities
        WHERE user_id = $1
        AND LOWER(name) = ANY($2)
        ORDER BY weight DESC
        "#,
    )
    .bind(user_id)
    .bind(&names.iter().map(|n| n.to_lowercase()).collect::<Vec<_>>())
    .fetch_all(pool)
    .await
    .map_err(|e| notice_core::Error::Database(e.to_string()))
}

/// Get all entities for a user (for inspection/debugging).
pub async fn get_all_entities(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<KgEntityRow>, notice_core::Error> {
    sqlx::query_as::<_, KgEntityRow>(
        r#"
        SELECT * FROM kg_entities
        WHERE user_id = $1
        ORDER BY weight DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(|e| notice_core::Error::Database(e.to_string()))
}

/// Get all relationships for a user (for inspection/debugging).
pub async fn get_all_relationships(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<KgRelationshipRow>, notice_core::Error> {
    sqlx::query_as::<_, KgRelationshipRow>(
        r#"
        SELECT * FROM kg_relationships
        WHERE user_id = $1
        ORDER BY weight DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(|e| notice_core::Error::Database(e.to_string()))
}

/// Get entities that are strongly related to the given entity names
/// through co_searched relationships. Used for query expansion.
///
/// Example: if "ownership" has a co_searched relationship with "rust" (weight 3.0),
/// and we query for related entities of ["ownership"], we get ["rust"].
pub async fn get_related_entities(
    pool: &PgPool,
    user_id: Uuid,
    entity_names: &[String],
    min_weight: f64,
    limit: i64,
) -> Result<Vec<KgEntityRow>, notice_core::Error> {
    if entity_names.is_empty() {
        return Ok(vec![]);
    }

    let names_lower: Vec<String> = entity_names.iter().map(|n| n.to_lowercase()).collect();

    // Find entities connected to the given names through relationships
    sqlx::query_as::<_, KgEntityRow>(
        r#"
        SELECT DISTINCT e2.*
        FROM kg_entities e1
        JOIN kg_relationships r ON (
            (r.from_entity_id = e1.id OR r.to_entity_id = e1.id)
            AND r.user_id = $1
            AND r.weight >= $3
        )
        JOIN kg_entities e2 ON (
            (e2.id = r.from_entity_id OR e2.id = r.to_entity_id)
            AND e2.id != e1.id
            AND e2.user_id = $1
        )
        WHERE e1.user_id = $1
        AND LOWER(e1.name) = ANY($2)
        AND LOWER(e2.name) != ALL($2)
        ORDER BY e2.weight DESC
        LIMIT $4
        "#,
    )
    .bind(user_id)
    .bind(&names_lower)
    .bind(min_weight)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| notice_core::Error::Database(e.to_string()))
}
