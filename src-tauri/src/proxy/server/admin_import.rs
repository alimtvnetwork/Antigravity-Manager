//! Account import (v1 / DB / custom DB) and CLI-sync admin handlers.
use super::app_state::AppState;
use super::dto::to_account_response;
use super::dto::AccountResponse;
use super::dto::ErrorResponse;
use crate::modules::{account, logger, migration};
use crate::proxy::TokenManager;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::Deserialize;

pub(crate) async fn admin_open_folder(
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    // Note: In Web mode, this may not actually open a local folder unless the backend handles it.
    // For ABV_Refactor, the backend should use opener to open it on the server (the desktop).
    crate::commands::open_data_folder().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    Ok(StatusCode::OK)
}

// --- Import Handlers ---

pub(crate) async fn admin_import_v1_accounts(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let accounts = migration::import_from_v1().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;

    // [FIX #1166]
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(state.token_manager.load_accounts().await, "load_accounts");

    let current_id = state.account_service.get_current_id().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    let responses: Vec<AccountResponse> = accounts
        .iter()
        .map(|a| to_account_response(a, &current_id))
        .collect();
    Ok(Json(responses))
}

pub(crate) async fn admin_import_from_db(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let account = migration::import_from_db(None).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;

    // [FIX #820]
    state.token_manager.clear_all_sessions();

    // [FIX #1166]
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(state.token_manager.load_accounts().await, "load_accounts");

    let current_id = state.account_service.get_current_id().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    Ok(Json(to_account_response(&account, &current_id)))
}

#[derive(Deserialize)]
struct CustomDbRequest {
    path: String,
}

pub(crate) async fn admin_import_custom_db(
    State(state): State<AppState>,
    Json(payload): Json<CustomDbRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    // [SECURITY]
    if payload.path.contains("..") {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Illegal path: directory traversal prohibited".to_string(),
            }),
        ));
    }

    let account = migration::import_from_custom_db_path(payload.path)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })?;

    // [FIX #820]
    state.token_manager.clear_all_sessions();

    // [FIX #1166]
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(state.token_manager.load_accounts().await, "load_accounts");

    let current_id = state.account_service.get_current_id().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    Ok(Json(to_account_response(&account, &current_id)))
}

pub(crate) async fn admin_sync_account_from_db(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let index = account::load_account_index().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    let current_target = index.current_target_ide.as_deref();
    if current_target == Some("agy") {
        return Ok(Json(None));
    }

    // sync_account_from_db command
    let db_refresh_token = match migration::get_refresh_token_from_db(current_target) {
        Ok(token) => token,
        Err(_e) => {
            return Ok(Json(None));
        }
    };
    let curr_account = account::get_current_account().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;

    if let Some(acc) = curr_account {
        if acc.token.refresh_token == db_refresh_token {
            return Ok(Json(None));
        }
    }

    let account = match migration::import_from_db(current_target).await {
        Ok(acc) => acc,
        Err(e) => {
            crate::modules::logger::log_info(&format!(
                "HTTP API 自动同步跳过 (本地账号导入失败): {}",
                e
            ));
            return Ok(Json(None));
        }
    };

    let account_id = account.id.clone();
    account::set_current_account_id_with_target(&account_id, current_target).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;

    // [FIX #820]
    state.token_manager.clear_all_sessions();

    // [FIX #1166]   TokenManager
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(state.token_manager.load_accounts().await, "load_accounts");

    let current_id = state.account_service.get_current_id().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    Ok(Json(Some(to_account_response(&account, &current_id))))
}

// --- CLI Sync Handlers ---

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CliSyncStatusRequest {
    app_type: crate::proxy::cli_sync::CliApp,
    proxy_url: String,
}

pub(crate) async fn admin_get_cli_sync_status(
    Json(payload): Json<CliSyncStatusRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    crate::proxy::cli_sync::get_cli_sync_status(payload.app_type, payload.proxy_url)
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
struct CliSyncRequest {
    app_type: crate::proxy::cli_sync::CliApp,
    proxy_url: String,
    api_key: String,
    pub model: Option<String>,
}

pub(crate) async fn admin_execute_cli_sync(
    Json(payload): Json<CliSyncRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    crate::proxy::cli_sync::execute_cli_sync(
        payload.app_type,
        payload.proxy_url,
        payload.api_key,
        payload.model,
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
struct CliRestoreRequest {
    app_type: crate::proxy::cli_sync::CliApp,
}

pub(crate) async fn admin_execute_cli_restore(
    Json(payload): Json<CliRestoreRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    crate::proxy::cli_sync::execute_cli_restore(payload.app_type)
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
struct CliConfigContentRequest {
    app_type: crate::proxy::cli_sync::CliApp,
    file_name: Option<String>,
}

pub(crate) async fn admin_get_cli_config_content(
    Json(payload): Json<CliConfigContentRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    crate::proxy::cli_sync::get_cli_config_content(payload.app_type, payload.file_name)
        .await
        .map(Json)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })
}
