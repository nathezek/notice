use axum::Json;
use axum::extract::State;

use crate::error::ApiError;
use crate::middleware::AuthUser;
use crate::state::AppState;

/// GET /api/me/kg — View your own knowledge graph.
/// Requires authentication.
pub async fn get_my_kg(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, ApiError> {
    let entities = notice_db::knowledge_graph::get_all_entities(&state.db, auth.user_id).await?;
    let relationships =
        notice_db::knowledge_graph::get_all_relationships(&state.db, auth.user_id).await?;

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

    let entity_map: std::collections::HashMap<uuid::Uuid, &str> =
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
        "user_id": auth.user_id,
        "username": auth.username,
        "entities": entity_list,
        "relationships": relationship_list,
        "entity_count": entity_list.len(),
        "relationship_count": relationship_list.len()
    })))
}

/// GET /api/me/kg/context — View what context would be injected into your searches.
/// Requires authentication.
pub async fn get_my_context(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, ApiError> {
    let context = notice_kg::context::load_user_context(&state.db, auth.user_id, 10).await?;

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
        "user_id": auth.user_id,
        "username": auth.username,
        "has_context": context.has_context,
        "top_interests": interests
    })))
}
