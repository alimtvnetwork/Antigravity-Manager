//! Cloudflared tunnel admin handlers.
use super::app_state::AppState;
use super::dto::ErrorResponse;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::Deserialize;

// Cloudflared Handlers
pub(crate) async fn admin_cloudflared_get_status(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    state
        .cloudflared_state
        .ensure_manager()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })?;

    let lock = state.cloudflared_state.manager.read().await;
    if let Some(manager) = lock.as_ref() {
        let (installed, version) = manager.check_installed().await;
        let mut status = manager.get_status().await;
        status.installed = installed;
        status.version = version;
        if !installed {
            status.running = false;
            status.url = None;
        }
        Ok(Json(status))
    } else {
        Ok(Json(
            crate::modules::cloudflared::CloudflaredStatus::default(),
        ))
    }
}

pub(crate) async fn admin_cloudflared_install(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    state
        .cloudflared_state
        .ensure_manager()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })?;

    let lock = state.cloudflared_state.manager.read().await;
    if let Some(manager) = lock.as_ref() {
        let status = manager.install().await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })?;
        Ok(Json(status))
    } else {
        Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Manager not initialized".to_string(),
            }),
        ))
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CloudflaredStartRequest {
    pub(crate) config: crate::modules::cloudflared::CloudflaredConfig,
}

pub(crate) async fn admin_cloudflared_start(
    State(state): State<AppState>,
    Json(payload): Json<CloudflaredStartRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    state
        .cloudflared_state
        .ensure_manager()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })?;

    let lock = state.cloudflared_state.manager.read().await;
    if let Some(manager) = lock.as_ref() {
        let status = manager.start(payload.config).await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })?;
        Ok(Json(status))
    } else {
        Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Manager not initialized".to_string(),
            }),
        ))
    }
}

pub(crate) async fn admin_cloudflared_stop(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    state
        .cloudflared_state
        .ensure_manager()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })?;

    let lock = state.cloudflared_state.manager.read().await;
    if let Some(manager) = lock.as_ref() {
        let status = manager.stop().await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })?;
        Ok(Json(status))
    } else {
        Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Manager not initialized".to_string(),
            }),
        ))
    }
}
