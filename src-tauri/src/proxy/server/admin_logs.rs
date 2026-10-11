//! Proxy logs, thinking store, z.ai model fetch, and data-dir admin handlers.
use super::app_state::AppState;
use super::dto::ErrorResponse;
use crate::modules::http_api::types::LogsRequest;
use crate::modules::{account, logger, proxy_db};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::Deserialize;
use serde_json::json;

pub(crate) async fn admin_fetch_zai_models(
    Path(_id): Path<String>,
    Json(payload): Json<serde_json::Value>, //
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    // ， ，  zai
    // fetch_zai_models  ，
    // reqwest  。
    let zai_config = payload.get("zai").ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Missing zai config".to_string(),
            }),
        )
    })?;

    let api_key = zai_config
        .get("api_key")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let base_url = zai_config
        .get("base_url")
        .and_then(|v| v.as_str())
        .unwrap_or("https://api.z.ai");

    // z.ai
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/v1/models", base_url))
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    let data: serde_json::Value = resp.json().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    // ID
    let models = data
        .get("data")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| {
                    m.get("id")
                        .and_then(|id| id.as_str().map(|s| s.to_string()))
                })
                .collect::<Vec<String>>()
        })
        .unwrap_or_default();

    Ok(Json(models))
}

pub(crate) async fn admin_set_proxy_monitor_enabled(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let enabled = payload
        .get("enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // [FIX #1269]  ， " "
    if state.monitor.is_enabled() != enabled {
        state.monitor.set_enabled(enabled);
        logger::log_info(&format!("[API] Monitor state updated to: {}", enabled));
    }

    StatusCode::OK
}

pub(crate) async fn admin_get_proxy_logs_count_filtered(
    Query(params): Query<LogsRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let res: Result<Result<u64, String>, tokio::task::JoinError> =
        tokio::task::spawn_blocking(move || {
            proxy_db::get_logs_count_filtered(&params.filter, params.errors_only)
        })
        .await;

    match res {
        Ok(Ok(count)) => Ok(Json(count)),
        Ok(Err(e)) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

pub(crate) async fn admin_clear_proxy_logs() -> impl IntoResponse {
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(
        tokio::task::spawn_blocking(|| {
            if let Err(e) = proxy_db::clear_logs() {
                logger::log_error(&format!("[API] Failed to clear proxy logs: {}", e));
            }
        })
        .await,
        "spawn_blocking",
    );
    logger::log_info("[API] Cleared all proxy logs");
    StatusCode::OK
}

pub(crate) async fn admin_clear_thinking_store() -> impl IntoResponse {
    crate::proxy::thinking_store::ThinkingStore::global().clear();
    crate::proxy::SignatureCache::global().clear();
    let res = tokio::task::spawn_blocking(crate::modules::proxy_db::clear_all_thinking_data).await;
    match res {
        Ok(Ok(deleted)) => {
            logger::log_info(&format!(
                "[API] Cleared thinking block storage (total {} records deleted)",
                deleted
            ));
            (StatusCode::OK, Json(json!({ "deleted": deleted })))
        }
        Ok(Err(e)) => {
            logger::log_error(&format!(
                "[API] Failed to clear thinking block storage: {}",
                e
            ));
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

pub(crate) async fn admin_get_thinking_store_count() -> impl IntoResponse {
    let res =
        tokio::task::spawn_blocking(crate::modules::proxy_db::get_thinking_records_count).await;
    match res {
        Ok(Ok(count)) => (StatusCode::OK, Json(json!({ "count": count }))),
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

pub(crate) async fn admin_get_proxy_db_disk_size(
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let res: Result<Result<u64, String>, tokio::task::JoinError> =
        tokio::task::spawn_blocking(move || proxy_db::get_proxy_db_disk_bytes()).await;

    match res {
        Ok(Ok(bytes)) => Ok(Json(bytes)),
        Ok(Err(e)) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}
pub(crate) async fn admin_get_proxy_log_detail(
    Path(log_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let res: Result<
        Result<crate::proxy::monitor::ProxyRequestLog, String>,
        tokio::task::JoinError,
    > = tokio::task::spawn_blocking(move || crate::modules::proxy_db::get_log_detail(&log_id))
        .await;

    match res {
        Ok(Ok(log)) => Ok(Json(log)),
        Ok(Err(e)) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LogsFilterQuery {
    #[serde(default)]
    pub(crate) filter: String,
    #[serde(default)]
    pub(crate) errors_only: bool,
    #[serde(default)]
    pub(crate) limit: usize,
    #[serde(default)]
    pub(crate) offset: usize,
}

pub(crate) async fn admin_get_proxy_logs_filtered(
    Query(params): Query<LogsFilterQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let res: Result<
        Result<Vec<crate::proxy::monitor::ProxyRequestLog>, String>,
        tokio::task::JoinError,
    > = tokio::task::spawn_blocking(move || {
        crate::modules::proxy_db::get_logs_filtered(
            &params.filter,
            params.errors_only,
            params.limit,
            params.offset,
        )
    })
    .await;

    match res {
        Ok(Ok(logs)) => Ok(Json(logs)),
        Ok(Err(e)) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

pub(crate) async fn admin_get_proxy_stats(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let stats = state.monitor.get_stats().await;
    Ok(Json(stats))
}

pub(crate) async fn admin_get_data_dir_path() -> impl IntoResponse {
    match crate::modules::account::get_data_dir() {
        Ok(p) => Json(crate::modules::account::format_data_dir_path(&p)),
        Err(e) => Json(format!("Error: {}", e)),
    }
}

#[derive(Deserialize)]
pub(crate) struct SetDataDirRequest {
    pub(crate) path: String,
}

pub(crate) async fn admin_set_data_dir(
    Json(payload): Json<SetDataDirRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let path = payload.path;
    let new_path = tokio::task::spawn_blocking(move || {
        crate::modules::account::migrate_data_dir(std::path::PathBuf::from(path))
    })
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    Ok(Json(crate::modules::account::format_data_dir_path(
        &new_path,
    )))
}
