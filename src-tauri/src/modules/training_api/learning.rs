use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::*;

// ============================================================================
// Learning & Feedback Ingestion
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnRequest {
    pub session_id: Option<String>,
    pub model: Option<String>,
    pub prompt_type: Option<String>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub latency_ms: Option<i64>,
    pub success: Option<bool>,
    pub score: Option<f64>,
    pub feedback: Option<String>,
    pub adjust_routing: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LearnResponse {
    pub status: String,
    pub log_id: String,
    pub timestamp: i64,
    pub message: String,
    pub model: Option<String>,
    pub updated_routing: bool,
}

pub fn ingest_learning(req: LearnRequest) -> Result<LearnResponse, String> {
    let conn = connect_training_db()?;
    let log_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();
    let success_int = if req.success.unwrap_or(true) { 1 } else { 0 };
    let adjust_int = if req.adjust_routing.unwrap_or(false) {
        1
    } else {
        0
    };

    conn.execute(
        "INSERT INTO training_logs (
            id, session_id, model, prompt_type, input_tokens, output_tokens,
            latency_ms, success, score, feedback, adjust_routing, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            log_id,
            req.session_id,
            req.model,
            req.prompt_type,
            req.input_tokens,
            req.output_tokens,
            req.latency_ms,
            success_int,
            req.score,
            req.feedback,
            adjust_int,
            now
        ],
    )
    .map_err(|e| format!("Failed to persist training log: {}", e))?;

    let mut updated_routing = false;
    if req.adjust_routing.unwrap_or(false) {
        if let Some(ref target_model) = req.model {
            if let Ok(mut app_cfg) = crate::modules::config::load_app_config() {
                app_cfg.auto_profile_switcher.target_model = target_model.clone();
                // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
                crate::error::record_ignored(
                    crate::modules::config::save_app_config(&app_cfg),
                    "save_app_config",
                );
                updated_routing = true;
                crate::modules::logger::log_info(&format!(
                    "[TrainingAPI] Feedback dynamically updated auto-switcher target model to '{}'",
                    target_model
                ));
            }
        }
    }

    Ok(LearnResponse {
        status: "ok".to_string(),
        log_id,
        timestamp: now,
        message: "Training signal ingested and logged successfully".to_string(),
        model: req.model,
        updated_routing,
    })
}

/// Axum POST /api/v1/training/learn
pub async fn handle_training_learn(Json(payload): Json<LearnRequest>) -> Response {
    if let Err(resp) = guard_training_api() {
        return resp;
    }

    match ingest_learning(payload) {
        Ok(resp) => (StatusCode::OK, Json(resp)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": format!("Failed to ingest learning: {}", e),
                "status": 500
            })),
        )
            .into_response(),
    }
}
