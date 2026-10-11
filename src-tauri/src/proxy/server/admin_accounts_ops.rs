//! Bulk account operations: delete, reorder, quota, priority, proxy toggle, warm-up.
use super::app_state::AppState;
use super::dto::ErrorResponse;
use crate::modules::{account, logger};
use crate::proxy::TokenManager;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct BulkDeleteRequest {
    #[serde(rename = "accountIds")]
    pub(crate) account_ids: Vec<String>,
}

pub(crate) async fn admin_delete_accounts(
    Json(payload): Json<BulkDeleteRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    crate::modules::account::delete_accounts(&payload.account_ids).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReorderRequest {
    pub(crate) account_ids: Vec<String>,
}

pub(crate) async fn admin_reorder_accounts(
    State(state): State<AppState>,
    Json(payload): Json<ReorderRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    crate::modules::account::reorder_accounts(&payload.account_ids).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;

    // [FIX #1166]   TokenManager
    if let Err(e) = state.token_manager.load_accounts().await {
        logger::log_error(&format!(
            "[API] Failed to reload accounts after reorder: {}",
            e
        ));
    }

    Ok(StatusCode::OK)
}

pub(crate) async fn admin_fetch_account_quota(
    Path(account_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let mut account = crate::modules::load_account(&account_id).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;

    let quota = crate::modules::account::fetch_quota_with_retry(&mut account)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    crate::modules::update_account_quota(&account_id, quota.clone()).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;

    Ok(Json(quota))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ToggleProxyRequest {
    pub(crate) enable: bool,
    pub(crate) reason: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct AccountPriorityRequest {
    #[serde(deserialize_with = "crate::models::account::deserialize_priority")]
    pub(crate) priority: u8,
}

pub(crate) async fn admin_update_account_priority(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
    Json(payload): Json<AccountPriorityRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    crate::modules::account::update_account_priority(&account_id, payload.priority).map_err(
        |e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        },
    )?;
    state
        .token_manager
        .update_account_priority(&account_id, payload.priority);
    Ok(StatusCode::OK)
}

pub(crate) async fn admin_toggle_proxy_status(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
    Json(payload): Json<ToggleProxyRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    crate::modules::account::toggle_proxy_status(
        &account_id,
        payload.enable,
        payload.reason.as_deref(),
    )
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;

    //
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(
        state.token_manager.reload_account(&account_id).await,
        "reload_account",
    );

    Ok(StatusCode::OK)
}

pub(crate) async fn admin_warm_up_all_accounts(
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let result = crate::commands::warm_up_all_accounts().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    Ok(Json(result))
}

pub(crate) async fn admin_warm_up_account(
    Path(account_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let result = crate::commands::warm_up_account(account_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })?;
    Ok(Json(result))
}
