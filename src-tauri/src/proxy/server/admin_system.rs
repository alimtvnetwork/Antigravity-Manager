//! System settings: updates, auto-launch, HTTP API, Antigravity paths, debug console.
use super::dto::ErrorResponse;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde_json::json;

pub(crate) async fn admin_should_check_updates(
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let settings = crate::modules::update_checker::load_update_settings().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    let should = crate::modules::update_checker::should_check_for_updates(&settings);
    Ok(Json(should))
}

pub(crate) async fn admin_get_antigravity_path(
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let path = crate::commands::get_antigravity_path(Some(true))
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;
    Ok(Json(path))
}

pub(crate) async fn admin_get_antigravity_args(
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let args = crate::commands::get_antigravity_args().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    Ok(Json(args))
}

pub(crate) async fn admin_clear_antigravity_cache(
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let res = crate::commands::clear_antigravity_cache()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })?;
    Ok(Json(res))
}

pub(crate) async fn admin_get_antigravity_cache_paths(
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let res = crate::commands::get_antigravity_cache_paths()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })?;
    Ok(Json(res))
}

pub(crate) async fn admin_clear_log_cache(
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    crate::commands::clear_log_cache().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    Ok(StatusCode::OK)
}

// Token Stats Handlers

pub(crate) async fn admin_get_update_settings() -> impl IntoResponse {
    //
    match crate::modules::update_checker::load_update_settings() {
        Ok(s) => Json(serde_json::to_value(s).unwrap_or_default()),
        Err(_) => Json(serde_json::json!({
            "auto_check": true,
            "last_check_time": 0,
            "check_interval_hours": 24
        })),
    }
}

pub(crate) async fn admin_check_for_updates(
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let info = crate::modules::update_checker::check_for_updates()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })?;
    Ok(Json(info))
}

pub(crate) async fn admin_update_last_check_time(
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    crate::modules::update_checker::update_last_check_time().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    Ok(StatusCode::OK)
}

pub(crate) async fn admin_save_update_settings(
    Json(settings): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Ok(s) =
        serde_json::from_value::<crate::modules::update_checker::UpdateSettings>(settings)
    {
        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
        crate::error::record_ignored(
            crate::modules::update_checker::save_update_settings(&s),
            "save_update_settings",
        );
        StatusCode::OK
    } else {
        StatusCode::BAD_REQUEST
    }
}

pub(crate) async fn admin_is_auto_launch_enabled() -> impl IntoResponse {
    // Note: Autostart requires tauri::AppHandle, which is not available in Axum State easily.
    // For now, return false in Web mode.
    Json(false)
}

pub(crate) async fn admin_toggle_auto_launch(
    Json(_payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    // Note: Autostart requires tauri::AppHandle.
    StatusCode::NOT_IMPLEMENTED
}

pub(crate) async fn admin_get_http_api_settings() -> impl IntoResponse {
    Json(serde_json::json!({ "enabled": true, "port": 8045 }))
}

// [ ]

pub(crate) async fn admin_save_http_api_settings(
    Json(payload): Json<crate::modules::http_api::HttpApiSettings>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    crate::modules::http_api::save_settings(&payload).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    Ok(StatusCode::OK)
}

// --- Debug Console Handlers ---

pub(crate) async fn admin_enable_debug_console() -> impl IntoResponse {
    crate::modules::log_bridge::enable_log_bridge();
    StatusCode::OK
}

pub(crate) async fn admin_disable_debug_console() -> impl IntoResponse {
    crate::modules::log_bridge::disable_log_bridge();
    StatusCode::OK
}

pub(crate) async fn admin_is_debug_console_enabled() -> impl IntoResponse {
    Json(crate::modules::log_bridge::is_log_bridge_enabled())
}

pub(crate) async fn admin_get_debug_console_logs() -> impl IntoResponse {
    let logs = crate::modules::log_bridge::get_buffered_logs();
    Json(logs)
}

pub(crate) async fn admin_clear_debug_console_logs() -> impl IntoResponse {
    crate::modules::log_bridge::clear_log_buffer();
    StatusCode::OK
}
