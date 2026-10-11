//! Proxy service control, proxy pool, rate limits, and model-mapping handlers.
use super::app_state::AppState;
use super::dto::ErrorResponse;
use crate::modules::{config, logger};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::Deserialize;
use serde_json::json;

// [FIX Web Mode] Get proxy pool config
pub(crate) async fn admin_get_proxy_pool_config(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let config = state.proxy_pool_state.read().await;
    Ok(Json(config.clone()))
}

// [FIX Web Mode] Get all account proxy bindings
pub(crate) async fn admin_get_all_account_bindings(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let bindings = state.proxy_pool_manager.get_all_bindings_snapshot();
    Ok(Json(bindings))
}

// [FIX Web Mode] Bind account to proxy
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BindAccountProxyRequest {
    pub(crate) account_id: String,
    pub(crate) proxy_id: String,
}

pub(crate) async fn admin_bind_account_proxy(
    State(state): State<AppState>,
    Json(payload): Json<BindAccountProxyRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    state
        .proxy_pool_manager
        .bind_account_to_proxy(payload.account_id, payload.proxy_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })?;
    Ok(StatusCode::OK)
}

// [FIX Web Mode] Unbind account from proxy
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UnbindAccountProxyRequest {
    pub(crate) account_id: String,
}

pub(crate) async fn admin_unbind_account_proxy(
    State(state): State<AppState>,
    Json(payload): Json<UnbindAccountProxyRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    state
        .proxy_pool_manager
        .unbind_account_proxy(payload.account_id)
        .await;
    Ok(StatusCode::OK)
}

// [FIX Web Mode] Get account proxy binding
pub(crate) async fn admin_get_account_proxy_binding(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let binding = state.proxy_pool_manager.get_account_binding(&account_id);
    Ok(Json(binding))
}

// [FIX Web Mode] Trigger proxy pool health check
pub(crate) async fn admin_trigger_proxy_health_check(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    state.proxy_pool_manager.health_check().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;

    // （ ）
    let config = state.proxy_pool_state.read().await;
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Health check completed",
        "proxies": config.proxies,
    })))
}

pub(crate) async fn admin_get_proxy_status(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    // Headless/Axum  ，AxumServer  ，  running
    let active_accounts = state.token_manager.len();

    let is_running = { *state.is_running.read().await };
    Ok(Json(serde_json::json!({
        "running": is_running,
        "port": state.port,
        "base_url": format!("http://127.0.0.1:{}", state.port),
        "active_accounts": active_accounts,
    })))
}

pub(crate) async fn admin_start_proxy_service(State(state): State<AppState>) -> impl IntoResponse {
    // 1. Persistence  (  #1166)
    if let Ok(mut config) = crate::modules::config::load_app_config() {
        config.proxy.auto_start = true;
        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
        crate::error::record_ignored(
            crate::modules::config::save_app_config(&config),
            "save_app_config",
        );
    }

    // 2.   ( )
    if let Err(e) = state.token_manager.load_accounts().await {
        logger::log_error(&format!(
            "[API] Failed to enable service and load accounts: {}",
            e
        ));
    }

    let mut running = state.is_running.write().await;
    *running = true;
    logger::log_info("[API] Proxy service enabled (persisted)");
    StatusCode::OK
}

pub(crate) async fn admin_set_proxy_capture_health_logs(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let enabled = payload
        .get("enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if state.monitor.is_capture_health_logs() != enabled {
        state.monitor.set_capture_health_logs(enabled);
        logger::log_info(&format!(
            "[API] Health check capture logging state set to: {}",
            enabled
        ));
    }

    StatusCode::OK
}

pub(crate) async fn admin_stop_proxy_service(State(state): State<AppState>) -> impl IntoResponse {
    // 1. Persistence  (  #1166)
    if let Ok(mut config) = crate::modules::config::load_app_config() {
        config.proxy.auto_start = false;
        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
        crate::error::record_ignored(
            crate::modules::config::save_app_config(&config),
            "save_app_config",
        );
    }

    let mut running = state.is_running.write().await;
    *running = false;
    logger::log_info("[API] Proxy service disabled (persisted)");
    StatusCode::OK
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UpdateMappingWrapper {
    pub(crate) config: crate::proxy::config::ProxyConfig,
}

pub(crate) async fn admin_update_model_mapping(
    State(state): State<AppState>,
    Json(payload): Json<UpdateMappingWrapper>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let config = payload.config;

    // 1.   ( )
    {
        let mut mapping = state.custom_mapping.write().await;
        *mapping = config.custom_mapping.clone();
    }

    // 2.   (  #1149)
    // ，  mapping，
    let mut app_config = crate::modules::config::load_app_config().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;

    app_config.proxy.custom_mapping = config.custom_mapping;

    crate::modules::config::save_app_config(&app_config).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;

    logger::log_info("[API] Model mappings hot-reloaded and saved via API");
    Ok(StatusCode::OK)
}

pub(crate) async fn admin_generate_api_key() -> impl IntoResponse {
    let new_key = format!("sk-{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
    Json(new_key)
}

pub(crate) async fn admin_clear_proxy_session_bindings(
    State(state): State<AppState>,
) -> impl IntoResponse {
    state.token_manager.clear_all_sessions();
    logger::log_info("[API] Cleared all session bindings");
    StatusCode::OK
}

pub(crate) async fn admin_clear_all_rate_limits(
    State(state): State<AppState>,
) -> impl IntoResponse {
    state.token_manager.clear_all_rate_limits();
    logger::log_info("[API] Cleared all rate limit records");
    StatusCode::OK
}

pub(crate) async fn admin_clear_rate_limit(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> impl IntoResponse {
    let cleared = state.token_manager.clear_rate_limit(&account_id);
    if cleared {
        logger::log_info(&format!(
            "[API] Cleared rate limit records for account {}",
            account_id
        ));
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

pub(crate) async fn admin_get_preferred_account(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let pref = state.token_manager.get_preferred_account().await;
    Json(pref)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SetPreferredAccountRequest {
    pub(crate) account_id: Option<String>,
}

pub(crate) async fn admin_set_preferred_account(
    State(state): State<AppState>,
    Json(payload): Json<SetPreferredAccountRequest>,
) -> impl IntoResponse {
    state
        .token_manager
        .set_preferred_account(payload.account_id)
        .await;
    StatusCode::OK
}
