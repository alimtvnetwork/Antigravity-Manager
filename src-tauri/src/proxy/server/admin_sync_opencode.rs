//! OpenCode config-sync admin handlers.
use super::dto::ErrorResponse;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpencodeSyncStatusRequest {
    pub(crate) proxy_url: String,
}

pub(crate) async fn admin_get_opencode_sync_status(
    Json(payload): Json<OpencodeSyncStatusRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    crate::proxy::opencode_sync::get_opencode_sync_status(payload.proxy_url)
        .await
        .map(Json)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })
}

pub(crate) async fn admin_get_opencode_families() -> impl IntoResponse {
    Json(crate::proxy::opencode_sync::get_canonical_families())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpencodeSyncRequest {
    pub(crate) proxy_url: String,
    pub(crate) api_key: String,
    #[serde(default)]
    pub(crate) sync_accounts: bool,
    pub(crate) models: Option<Vec<crate::proxy::opencode_sync::ModelInput>>,
}

pub(crate) async fn admin_execute_opencode_sync(
    Json(payload): Json<OpencodeSyncRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    crate::proxy::opencode_sync::execute_opencode_sync(
        payload.proxy_url,
        payload.api_key,
        Some(payload.sync_accounts),
        payload.models,
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpencodeOpenaiSyncRequest {
    pub(crate) proxy_url: String,
    pub(crate) api_key: String,
    #[serde(default)]
    pub(crate) provider_id: Option<String>,
    #[serde(default)]
    pub(crate) provider_name: Option<String>,
    pub(crate) models: Option<Vec<crate::proxy::opencode_sync::ModelInput>>,
}

pub(crate) async fn admin_execute_opencode_openai_sync(
    Json(payload): Json<OpencodeOpenaiSyncRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    crate::proxy::opencode_sync::execute_opencode_openai_sync(
        payload.proxy_url,
        payload.api_key,
        payload.provider_id,
        payload.provider_name,
        payload.models,
    )
    .await
    .map(|_| StatusCode::OK)
    .map_err(|e| {
        let status = if crate::proxy::opencode_sync::is_provider_validation_error(&e) {
            StatusCode::BAD_REQUEST
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        };
        (status, Json(ErrorResponse { error: e }))
    })
}

pub(crate) async fn admin_get_opencode_providers(
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    crate::proxy::opencode_sync::get_opencode_providers()
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
pub(crate) struct OpencodeRemoveProviderRequest {
    pub(crate) provider_id: String,
}

pub(crate) async fn admin_execute_opencode_remove_provider(
    Json(payload): Json<OpencodeRemoveProviderRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    crate::proxy::opencode_sync::execute_opencode_remove_provider(payload.provider_id)
        .await
        .map(|_| StatusCode::OK)
        .map_err(|e| {
            let status = if crate::proxy::opencode_sync::is_provider_validation_error(&e) {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (status, Json(ErrorResponse { error: e }))
        })
}

pub(crate) async fn admin_execute_opencode_restore(
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    crate::proxy::opencode_sync::execute_opencode_restore()
        .await
        .map(|_| StatusCode::OK)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetOpencodeConfigRequest {
    pub(crate) file_name: Option<String>,
}

pub(crate) async fn admin_get_opencode_config_content(
    Json(payload): Json<GetOpencodeConfigRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let file_name = payload.file_name;
    tokio::task::spawn_blocking(move || {
        crate::proxy::opencode_sync::read_opencode_config_content(file_name)
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
pub(crate) struct OpencodeClearRequest {
    pub(crate) proxy_url: Option<String>,
    pub(crate) clear_legacy: Option<bool>,
}

pub(crate) async fn admin_execute_opencode_clear(
    Json(payload): Json<OpencodeClearRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    crate::proxy::opencode_sync::execute_opencode_clear(payload.proxy_url, payload.clear_legacy)
        .await
        .map(|_| StatusCode::OK)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })
}
