use crate::extractor::{self, ExtractedEntity};
use sqlx::PgPool;
use uuid::Uuid;

/// Update a user's knowledge graph based on a search query.
/// This runs AFTER the search response is sent — fire and forget.
///
/// What it does:
/// 1. Extract entities from the query
/// 2. Upsert each entity (create or increment weight)
/// 3. Create relationships between co-occurring entities
pub async fn update_from_search(pool: &PgPool, user_id: Uuid, query: &str) {
    let entities = extractor::extract_entities(query);

    if entities.is_empty() {
        return;
    }

    tracing::debug!(
        user_id = %user_id,
        entities = ?entities.iter().map(|e| &e.name).collect::<Vec<_>>(),
        "Updating KG from search"
    );

    // Step 1: Upsert all entities
    let mut entity_ids: Vec<(Uuid, ExtractedEntity)> = vec![];

    for entity in &entities {
        match notice_db::knowledge_graph::upsert_entity(
            pool,
            user_id,
            &entity.name,
            &entity.entity_type,
        )
        .await
        {
            Ok(row) => {
                entity_ids.push((row.id, entity.clone()));
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    entity = %entity.name,
                    "Failed to upsert KG entity"
                );
            }
        }
    }

    // Step 2: Create relationships between co-occurring entities
    // If a user searches "rust ownership", we create a relationship
    // between "rust" and "ownership" (they appear together).
    if entity_ids.len() >= 2 {
        for i in 0..entity_ids.len() {
            for j in (i + 1)..entity_ids.len() {
                let (from_id, _) = &entity_ids[i];
                let (to_id, _) = &entity_ids[j];

                if let Err(e) = notice_db::knowledge_graph::upsert_relationship(
                    pool,
                    user_id,
                    *from_id,
                    *to_id,
                    "co_searched",
                )
                .await
                {
                    tracing::debug!(error = %e, "Failed to upsert KG relationship");
                }
            }
        }
    }
}

/// Spawn the KG update as a background task (non-blocking).
/// If the update fails, it's logged but never affects the user.
pub fn spawn_kg_update(pool: PgPool, user_id: Uuid, query: String) {
    tokio::spawn(async move {
        update_from_search(&pool, user_id, &query).await;
    });
}
