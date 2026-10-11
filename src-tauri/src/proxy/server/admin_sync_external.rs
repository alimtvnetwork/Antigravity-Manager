//! External agent-tool sync admin handlers (Hermes, OpenClaw, Droid).
use super::dto::ErrorResponse;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::Deserialize;
use serde_json::json;

// ── Hermes Agent Sync Admin Handlers ──

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HermesSyncStatusRequest {
    #[serde(default)]
    pub(crate) proxy_url: Option<String>,
}

pub(crate) async fn admin_get_hermes_sync_status(
    Json(payload): Json<HermesSyncStatusRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    crate::proxy::hermes_sync::get_hermes_sync_status(payload.proxy_url)
        .await
        .map(Json)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HermesSyncRequest {
    pub(crate) proxy_url: String,
    pub(crate) api_key: String,
    pub(crate) discover_models: bool,
    #[serde(default)]
    pub(crate) models: Vec<String>,
    #[serde(default)]
    pub(crate) activate: bool,
    #[serde(default)]
    pub(crate) default_model: Option<String>,
}

pub(crate) async fn admin_execute_hermes_sync(
    Json(payload): Json<HermesSyncRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    crate::proxy::hermes_sync::execute_hermes_sync(
        payload.proxy_url,
        payload.api_key,
        payload.discover_models,
        payload.models,
        payload.activate,
        payload.default_model,
    )
    .await
    .map(|_| StatusCode::OK)
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })
}

pub(crate) async fn admin_execute_hermes_restore(
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    crate::proxy::hermes_sync::execute_hermes_restore()
        .await
        .map(|_| StatusCode::OK)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })
}

pub(crate) async fn admin_execute_hermes_clear(
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    crate::proxy::hermes_sync::execute_hermes_clear()
        .await
        .map(|_| StatusCode::OK)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })
}

pub(crate) async fn admin_get_hermes_config_content(
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    crate::proxy::hermes_sync::get_hermes_config_content()
        .await
        .map(Json)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })
}

// ── OpenClaw Sync Admin Handlers ──

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenClawSyncStatusRequest {
    #[serde(default)]
    pub(crate) proxy_url: Option<String>,
}

pub(crate) async fn admin_get_openclaw_sync_status(
    Json(payload): Json<OpenClawSyncStatusRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    crate::proxy::openclaw_sync::get_openclaw_sync_status(payload.proxy_url)
        .await
        .map(Json)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenClawSyncRequest {
    pub(crate) proxy_url: String,
    pub(crate) api_key: String,
    #[serde(default = "default_openclaw_target_version")]
    pub(crate) target_version: String,
    #[serde(default)]
    pub(crate) models: Vec<String>,
    #[serde(default)]
    pub(crate) activate: bool,
    #[serde(default)]
    pub(crate) default_model: Option<String>,
}

fn default_openclaw_target_version() -> String {
    "v2".to_string()
}

pub(crate) async fn admin_execute_openclaw_sync(
    Json(payload): Json<OpenClawSyncRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    crate::proxy::openclaw_sync::execute_openclaw_sync(
        payload.proxy_url,
        payload.api_key,
        payload.target_version,
        payload.models,
        payload.activate,
        payload.default_model,
    )
    .await
    .map(|_| StatusCode::OK)
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })
}

pub(crate) async fn admin_execute_openclaw_restore(
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    crate::proxy::openclaw_sync::execute_openclaw_restore()
        .await
        .map(|_| StatusCode::OK)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })
}

pub(crate) async fn admin_execute_openclaw_clear(
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    crate::proxy::openclaw_sync::execute_openclaw_clear()
        .await
        .map(|_| StatusCode::OK)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })
}

pub(crate) async fn admin_get_openclaw_config_content(
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    crate::proxy::openclaw_sync::get_openclaw_config_content()
        .await
        .map(Json)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })
}

// ── Droid (Factory CLI) Sync Admin Handlers ──

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DroidSyncStatusRequest {
    pub(crate) proxy_url: String,
}

pub(crate) async fn admin_get_droid_sync_status(
    Json(payload): Json<DroidSyncStatusRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    crate::proxy::droid_sync::get_droid_sync_status(payload.proxy_url)
        .await
        .map(Json)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DroidSyncRequest {
    pub(crate) custom_models: Vec<serde_json::Value>,
}

pub(crate) async fn admin_execute_droid_sync(
    Json(payload): Json<DroidSyncRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    crate::proxy::droid_sync::execute_droid_sync(payload.custom_models)
        .await
        .map(|count| Json(serde_json::json!({ "added": count })))
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })
}

pub(crate) async fn admin_execute_droid_restore(
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    crate::proxy::droid_sync::execute_droid_restore()
        .await
        .map(|_| StatusCode::OK)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })
}

pub(crate) async fn admin_get_droid_config_content(
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    crate::proxy::droid_sync::get_droid_config_content()
        .await
        .map(Json)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })
}
