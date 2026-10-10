use crate::modules::{account, logger, proxy_db};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use super::*;

// ============================================================================
// Handlers
// ============================================================================

/// GET /health - Health check
async fn health() -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// GET /accounts - Get all accounts
async fn list_accounts() -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let accounts = account::list_accounts().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;

    let current_id = account::get_current_account_id().ok().flatten();

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
                updated_at: Some(q.last_updated),
                subscription_tier: q.subscription_tier,
            });

            AccountResponse {
                id: acc.id,
                email: acc.email,
                name: acc.name,
                is_current,
                disabled: acc.disabled,
                live_limited_models: acc.live_limited_models,
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

/// GET /accounts/current - Get current account
async fn get_current_account() -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let current = account::get_current_account().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;

    let response = current.map(|acc| {
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
            updated_at: Some(q.last_updated),
            subscription_tier: q.subscription_tier,
        });

        AccountResponse {
            id: acc.id,
            email: acc.email,
            name: acc.name,
            is_current: true,
            disabled: acc.disabled,
            live_limited_models: acc.live_limited_models,
            quota,
            device_bound: acc.device_profile.is_some(),
            last_used: acc.last_used,
        }
    });

    Ok(Json(CurrentAccountResponse { account: response }))
}

/// POST /accounts/switch - Switch account
async fn switch_account(
    State(state): State<ApiState>,
    Json(payload): Json<SwitchRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    // Check if another switch operation is already in progress
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

    // Mark switch started
    {
        let mut switching = state.switching.write().await;
        *switching = true;
    }

    let account_id = payload.account_id.clone();
    let state_clone = state.clone();

    // Execute switch asynchronously (non-blocking response)
    tokio::spawn(async move {
        logger::log_info(&format!(
            "[HTTP API] Starting account switch: {}",
            account_id
        ));

        match account::switch_account(&account_id, None, &state_clone.integration).await {
            Ok(()) => {
                logger::log_info(&format!(
                    "[HTTP API] Account switch successful: {}",
                    account_id
                ));
            }
            Err(e) => {
                logger::log_error(&format!("[HTTP API] Account switch failed: {}", e));
            }
        }

        // Mark switch ended
        let mut switching = state_clone.switching.write().await;
        *switching = false;
    });

    // Immediately return 202 Accepted
    Ok((
        StatusCode::ACCEPTED,
        Json(SwitchResponse {
            success: true,
            message: format!("Account switch task started: {}", payload.account_id),
        }),
    ))
}

/// POST /accounts/refresh - Refresh all quotas
async fn refresh_all_quotas() -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    logger::log_info("[HTTP API] Starting refresh of all account quotas");

    // Execute refresh asynchronously
    tokio::spawn(async {
        match account::refresh_all_quotas_logic().await {
            Ok(stats) => {
                logger::log_info(&format!(
                    "[HTTP API] Quota refresh completed, successful {}/{} accounts",
                    stats.success, stats.total
                ));
            }
            Err(e) => {
                logger::log_error(&format!("[HTTP API] Quota refresh failed: {}", e));
            }
        }
    });

    Ok((
        StatusCode::ACCEPTED,
        Json(RefreshResponse {
            success: true,
            message: "Quota refresh task started".to_string(),
            refreshed_count: 0,
        }),
    ))
}

/// POST /accounts/:id/bind-device - Bind device fingerprint
async fn bind_device(
    Path(account_id): Path<String>,
    Json(payload): Json<BindDeviceRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    logger::log_info(&format!(
        "[HTTP API] Binding device fingerprint: account={}, mode={}",
        account_id, payload.mode
    ));

    let result = account::bind_device_profile(&account_id, &payload.mode).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;

    Ok(Json(BindDeviceResponse {
        success: true,
        message: "Device fingerprint bound successfully".to_string(),
        device_profile: Some(DeviceProfileResponse {
            machine_id: result.machine_id,
            mac_machine_id: result.mac_machine_id,
            dev_device_id: result.dev_device_id,
            sqm_id: result.sqm_id,
        }),
    }))
}

/// GET /logs - Get proxy logs
async fn get_logs(
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

    Ok(Json(LogsResponse { total, logs }))
}
