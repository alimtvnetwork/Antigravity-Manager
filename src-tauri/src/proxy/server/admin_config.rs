//! Proxy configuration and log-viewer admin handlers.
use super::app_state::AppState;
use super::dto::ErrorResponse;
use crate::models::AppConfig;
use crate::modules::{config, proxy_db};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)] // Reserved log interface struct
struct LogsRequest {
    #[serde(default)]
    limit: usize,
    #[serde(default)]
    offset: usize,
    #[serde(default)]
    filter: String,
    #[serde(default)]
    errors_only: bool,
}

#[allow(dead_code)] // Reserved log endpoint
pub(crate) async fn admin_get_logs(
    Query(params): Query<LogsRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let limit = if params.limit == 0 { 50 } else { params.limit };
    let total =
        proxy_db::get_logs_count_filtered(&params.filter, params.errors_only).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })?;
    let logs =
        proxy_db::get_logs_filtered(&params.filter, params.errors_only, limit, params.offset)
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse { error: e }),
                )
            })?;

    Ok(Json(serde_json::json!({
        "total": total,
        "logs": logs,
    })))
}

pub(crate) async fn admin_get_config(
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let cfg = config::load_app_config().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    Ok(Json(cfg))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaveConfigWrapper {
    config: AppConfig,
}

pub(crate) async fn admin_save_config(
    State(state): State<AppState>,
    Json(payload): Json<SaveConfigWrapper>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let new_config = payload.config;
    // 1. Persistence
    config::save_app_config(&new_config).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;

    // 2. Hot-reload memory state
    // Reuse internal component update methods
    // ：AppState   Arc<RwLock>

    // AxumServer  ，
    // AppState  。
    // ，  AppState  。

    //
    {
        let mut mapping = state.custom_mapping.write().await;
        *mapping = new_config.clone().proxy.custom_mapping;
    }

    //
    {
        let mut proxy = state.upstream_proxy.write().await;
        *proxy = new_config.clone().proxy.upstream_proxy;
    }

    //
    {
        let mut security = state.security.write().await;
        *security = crate::proxy::ProxySecurityConfig::from_proxy_config(&new_config.proxy);
    }

    // z.ai
    {
        let mut zai = state.zai.write().await;
        *zai = new_config.clone().proxy.zai;
    }

    //
    {
        let mut exp = state.experimental.write().await;
        *exp = new_config.clone().proxy.experimental;
    }

    // （Web/Docker  ）
    {
        let mut pool = state.proxy_pool_state.write().await;
        *pool = new_config.clone().proxy.proxy_pool;
    }

    Ok(StatusCode::OK)
}
