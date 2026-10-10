// `GET /v1/models`.
use axum::{extract::State, response::IntoResponse, Json};
use serde_json::json;

use crate::proxy::server::AppState;

pub async fn handle_list_models(State(state): State<AppState>) -> impl IntoResponse {
    use crate::proxy::common::model_mapping::get_all_dynamic_models;

    let only_raw = *state.only_raw_quota_models.read().await;
    let model_ids =
        get_all_dynamic_models(&state.custom_mapping, Some(&state.token_manager), only_raw).await;

    let data: Vec<_> = model_ids
        .into_iter()
        .map(|id| {
            json!({
                "id": id,
                "object": "model",
                "created": 1706745600,
                "owned_by": "antigravity"
            })
        })
        .collect();

    Json(json!({
        "object": "list",
        "data": data
    }))
}
