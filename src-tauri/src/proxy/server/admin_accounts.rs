//! Account CRUD, switching, rotation, and auto-switcher admin handlers.
use super::app_state::AppState;
use super::dto::quota_group_to_dto;
use super::dto::to_account_response;
use super::dto::AccountListResponse;
use super::dto::AccountResponse;
use super::dto::ErrorResponse;
use super::dto::ModelQuota;
use super::dto::QuotaResponse;
use crate::modules::{account, logger};
use crate::proxy::TokenManager;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::{Deserialize, Serialize};

// [Cleanup] Legacy model definitions migrated upstream

pub(crate) async fn admin_list_accounts(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let accounts = state.account_service.list_accounts().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;

    let current_id = state.account_service.get_current_id().ok().flatten();

    let account_responses: Vec<AccountResponse> = accounts
        .into_iter()
        .map(|acc| {
            let is_current = current_id.as_ref().map(|id| id == &acc.id).unwrap_or(false);
            let quota = acc.quota.map(|q| QuotaResponse {
                models: q
                    .models
                    .into_iter()
                    .map(|m| ModelQuota {
                        name: m.name,
                        percentage: m.percentage,
                        reset_time: m.reset_time,
                    })
                    .collect(),
                last_updated: q.last_updated,
                subscription_tier: q.subscription_tier,
                is_forbidden: q.is_forbidden,
                quota_groups: q
                    .quota_groups
                    .map(|groups| groups.iter().map(quota_group_to_dto).collect()),
            });

            AccountResponse {
                id: acc.id,
                email: acc.email,
                name: acc.name,
                priority: acc.priority,
                is_current,
                disabled: acc.disabled,
                disabled_reason: acc.disabled_reason,
                disabled_at: acc.disabled_at,
                proxy_disabled: acc.proxy_disabled,
                proxy_disabled_reason: acc.proxy_disabled_reason,
                proxy_disabled_at: acc.proxy_disabled_at,
                protected_models: acc.protected_models.into_iter().collect(),
                live_limited_models: acc.live_limited_models,
                validation_blocked: acc.validation_blocked,
                validation_blocked_until: acc.validation_blocked_until,
                validation_blocked_reason: acc.validation_blocked_reason,
                quota,
                device_bound: acc.device_profile.is_some(),
                last_used: acc.last_used,
            }
        })
        .collect();

    Ok(Json(AccountListResponse {
        current_account_id: current_id,
        accounts: account_responses,
    }))
}

/// Export accounts with refresh tokens (for backup/migration)
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExportAccountsRequest {
    account_ids: Vec<String>,
}

pub(crate) async fn admin_export_accounts(
    State(_state): State<AppState>,
    Json(payload): Json<ExportAccountsRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let response = account::export_accounts_by_ids(&payload.account_ids).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;

    Ok(Json(response))
}

pub(crate) async fn admin_get_current_account(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let current_id = state.account_service.get_current_id().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;

    let response = if let Some(id) = current_id {
        let acc = account::load_account(&id).ok();
        acc.map(|acc| {
            let quota = acc.quota.map(|q| QuotaResponse {
                models: q
                    .models
                    .into_iter()
                    .map(|m| ModelQuota {
                        name: m.name,
                        percentage: m.percentage,
                        reset_time: m.reset_time,
                    })
                    .collect(),
                last_updated: q.last_updated,
                subscription_tier: q.subscription_tier,
                is_forbidden: q.is_forbidden,
                quota_groups: q
                    .quota_groups
                    .map(|groups| groups.iter().map(quota_group_to_dto).collect()),
            });

            AccountResponse {
                id: acc.id,
                email: acc.email,
                name: acc.name,
                priority: acc.priority,
                is_current: true,
                disabled: acc.disabled,
                disabled_reason: acc.disabled_reason,
                disabled_at: acc.disabled_at,
                proxy_disabled: acc.proxy_disabled,
                proxy_disabled_reason: acc.proxy_disabled_reason,
                proxy_disabled_at: acc.proxy_disabled_at,
                protected_models: acc.protected_models.into_iter().collect(),
                live_limited_models: acc.live_limited_models,
                validation_blocked: acc.validation_blocked,
                validation_blocked_until: acc.validation_blocked_until,
                validation_blocked_reason: acc.validation_blocked_reason,
                quota,
                device_bound: acc.device_profile.is_some(),
                last_used: acc.last_used,
            }
        })
    } else {
        None
    };

    Ok(Json(response))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddAccountRequest {
    refresh_token: String,
}

pub(crate) async fn admin_add_account(
    State(state): State<AppState>,
    Json(payload): Json<AddAccountRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let account = state
        .account_service
        .add_account(&payload.refresh_token)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })?;

    // [FIX #1166] Reload TokenManager immediately after account changes
    if let Err(e) = state.token_manager.load_accounts().await {
        logger::log_error(&format!(
            "[API] Failed to reload accounts after adding: {}",
            e
        ));
    }

    let current_id = state.account_service.get_current_id().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    Ok(Json(to_account_response(&account, &current_id)))
}

pub(crate) async fn admin_delete_account(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    state
        .account_service
        .delete_account(&account_id)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })?;

    // [FIX #1166] Reload TokenManager immediately after account changes
    if let Err(e) = state.token_manager.load_accounts().await {
        logger::log_error(&format!(
            "[API] Failed to reload accounts after deletion: {}",
            e
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SwitchRequest {
    pub(crate) account_id: String,
    #[serde(default)]
    pub(crate) target_ide: Option<String>,
}

pub(crate) async fn admin_switch_account(
    State(state): State<AppState>,
    Json(payload): Json<SwitchRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    {
        let switching = state.switching.read().await;
        if *switching {
            return Err((
                StatusCode::CONFLICT,
                Json(ErrorResponse {
                    error: "Another switch operation is already in progress".to_string(),
                }),
            ));
        }
    }

    {
        let mut switching = state.switching.write().await;
        *switching = true;
    }

    let account_id = payload.account_id.clone();
    logger::log_info(&format!(
        "[API] Starting account switch: {} (target_ide: {:?})",
        account_id, payload.target_ide
    ));

    let result = state
        .account_service
        .switch_account(&account_id, payload.target_ide.as_deref())
        .await;

    {
        let mut switching = state.switching.write().await;
        *switching = false;
    }

    match result {
        Ok(()) => {
            logger::log_info(&format!("[API] Account switch successful: {}", account_id));

            // [FIX #1166] Synchronize memory state immediately after account switch
            state.token_manager.clear_all_sessions();
            if let Err(e) = state.token_manager.load_accounts().await {
                logger::log_error(&format!(
                    "[API] Failed to reload accounts after switch: {}",
                    e
                ));
            }

            Ok(StatusCode::OK)
        }
        Err(e) => {
            logger::log_error(&format!("[API] Account switch failed: {}", e));
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            ))
        }
    }
}

#[derive(Serialize)]
struct RotateAccountResponse {
    is_success: bool,
    message: String,
    status: crate::modules::auto_switcher::AutoSwitcherStatus,
}

pub(crate) async fn admin_rotate_account(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    {
        let switching = state.switching.read().await;
        if *switching {
            return Err((
                StatusCode::CONFLICT,
                Json(ErrorResponse {
                    error: "Another switch or rotation operation is already in progress"
                        .to_string(),
                }),
            ));
        }
    }

    {
        let mut switching = state.switching.write().await;
        *switching = true;
    }

    let result = crate::modules::auto_switcher::trigger_manual_rotation().await;

    {
        let mut switching = state.switching.write().await;
        *switching = false;
    }

    match result {
        Ok(msg) => {
            logger::log_info(&format!("[API] Account rotation successful: {}", msg));
            state.token_manager.clear_all_sessions();
            if let Err(e) = state.token_manager.load_accounts().await {
                logger::log_error(&format!(
                    "[API] Failed to reload accounts after rotation: {}",
                    e
                ));
            }

            let switcher_status = crate::modules::auto_switcher::get_status();
            Ok(Json(RotateAccountResponse {
                is_success: true,
                message: msg,
                status: switcher_status,
            }))
        }
        Err(e) => {
            logger::log_error(&format!("[API] Account rotation failed: {}", e));
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            ))
        }
    }
}

pub(crate) async fn admin_get_auto_switcher_status(
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let status = crate::modules::auto_switcher::get_status();
    Ok(Json(status))
}
