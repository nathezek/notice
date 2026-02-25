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
