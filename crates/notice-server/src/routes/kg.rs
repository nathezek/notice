use axum::Json;
use axum::extract::{Path, State};
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;

/// GET /api/users/{user_id}/kg — View a user's knowledge graph.
pub async fn get_user_kg(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Verify user exists
    notice_db::users::get_by_id(&state.db, user_id)
        .await?
        .ok_or_else(|| notice_core::Error::NotFound("User not found".into()))?;

    let entities = notice_db::knowledge_graph::get_all_entities(&state.db, user_id).await?;
    let relationships =
        notice_db::knowledge_graph::get_all_relationships(&state.db, user_id).await?;

    // Build a readable graph view
    let entity_list: Vec<serde_json::Value> = entities
        .iter()
        .map(|e| {
            serde_json::json!({
                "id": e.id,
                "name": e.name,
                "type": e.entity_type,
                "weight": e.weight
            })
        })
        .collect();

    // Resolve relationship names
    let entity_map: std::collections::HashMap<Uuid, &str> =
        entities.iter().map(|e| (e.id, e.name.as_str())).collect();

    let relationship_list: Vec<serde_json::Value> = relationships
        .iter()
        .filter_map(|r| {
            let from = entity_map.get(&r.from_entity_id)?;
            let to = entity_map.get(&r.to_entity_id)?;
            Some(serde_json::json!({
                "from": from,
                "to": to,
                "type": r.relationship_type,
                "weight": r.weight
            }))
        })
        .collect();

    Ok(Json(serde_json::json!({
        "user_id": user_id,
        "entities": entity_list,
        "relationships": relationship_list,
        "entity_count": entity_list.len(),
        "relationship_count": relationship_list.len()
    })))
}

/// GET /api/users/{user_id}/kg/context — View what context would be injected into a search.
pub async fn get_user_context(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let context = notice_kg::context::load_user_context(&state.db, user_id, 10).await?;

    let interests: Vec<serde_json::Value> = context
        .top_interests
        .iter()
        .map(|t| {
            serde_json::json!({
                "term": t.term,
                "weight": t.weight,
                "type": t.entity_type
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "user_id": user_id,
        "has_context": context.has_context,
        "top_interests": interests
    })))
}
