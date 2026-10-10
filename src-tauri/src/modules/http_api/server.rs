use crate::modules::{account, logger, proxy_db};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use tower_http::cors::{Any, CorsLayer};

use super::*;
use super::handlers::list_accounts;
use super::handlers::get_current_account;
use super::handlers::get_logs;

// ============================================================================
// Server
// ============================================================================

/// Start HTTP API server
pub async fn start_server(
    port: u16,
    integration: crate::modules::integration::SystemManager,
) -> Result<(), String> {
    let state = ApiState::new(integration);

    // CORS config - allow local calls
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(health))
        .route("/accounts", get(list_accounts))
        .route("/accounts/current", get(get_current_account))
        .route("/accounts/switch", post(switch_account))
        .route("/accounts/refresh", post(refresh_all_quotas))
        .route("/accounts/{id}/bind-device", post(bind_device))
        .route("/logs", get(get_logs))
        .layer(cors)
        .with_state(state);

    let addr = format!("127.0.0.1:{}", port);
    logger::log_info(&format!("[HTTP API] Starting server: http://{}", addr));

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("failed_to_bind_port: {}", e))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| format!("failed_to_run_server: {}", e))?;

    Ok(())
}

/// Start HTTP API server in background (non-blocking)
pub fn spawn_server(port: u16, integration: crate::modules::integration::SystemManager) {
    // Use tauri::async_runtime::spawn to ensure running within Tauri's runtime
    tauri::async_runtime::spawn(async move {
        if let Err(e) = start_server(port, integration).await {
            logger::log_error(&format!("[HTTP API] Failed to start server: {}", e));
        }
    });
}
