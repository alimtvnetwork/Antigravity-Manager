//! Device binding, device profiles, and device-version admin handlers.
use super::app_state::AppState;
use super::dto::ErrorResponse;
use crate::modules::account;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct BindDeviceRequest {
    #[serde(default = "default_bind_mode")]
    mode: String,
}

fn default_bind_mode() -> String {
    "generate".to_string()
}

pub(crate) async fn admin_bind_device(
    Path(account_id): Path<String>,
    Json(payload): Json<BindDeviceRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let result = account::bind_device_profile(&account_id, &payload.mode).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Device fingerprint bound successfully",
        "device_profile": result,
    })))
}

// --- Supplementary Account Handlers ---

pub(crate) async fn admin_get_device_profiles(
    State(_state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let profiles = account::get_device_profiles(&account_id).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    Ok(Json(profiles))
}

pub(crate) async fn admin_list_device_versions(
    State(_state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let profiles = account::get_device_profiles(&account_id).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    Ok(Json(profiles))
}

pub(crate) async fn admin_preview_generate_profile(
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let profile = crate::modules::device::generate_profile();
    Ok(Json(profile))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BindDeviceProfileWrapper {
    #[serde(default)]
    account_id: String,
    #[serde(alias = "profile")]
    profile_wrapper: DeviceProfileApiWrapper,
}

// API   DeviceProfile  ，  camelCase
#[derive(Deserialize)]
struct DeviceProfileApiWrapper {
    #[serde(alias = "machineId")]
    machine_id: String,
    #[serde(alias = "macMachineId")]
    mac_machine_id: String,
    #[serde(alias = "devDeviceId")]
    dev_device_id: String,
    #[serde(alias = "sqmId")]
    sqm_id: String,
}

impl From<DeviceProfileApiWrapper> for crate::models::account::DeviceProfile {
    fn from(wrapper: DeviceProfileApiWrapper) -> Self {
        Self {
            machine_id: wrapper.machine_id,
            mac_machine_id: wrapper.mac_machine_id,
            dev_device_id: wrapper.dev_device_id,
            sqm_id: wrapper.sqm_id,
        }
    }
}

pub(crate) async fn admin_bind_device_profile_with_profile(
    State(_state): State<AppState>,
    Path(account_id): Path<String>,
    Json(payload): Json<BindDeviceProfileWrapper>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    // payload   account_id（ ），
    let target_account_id = if !payload.account_id.is_empty() {
        &payload.account_id
    } else {
        &account_id
    };

    let profile: crate::models::account::DeviceProfile = payload.profile_wrapper.into();

    let result = account::bind_device_profile_with_profile(target_account_id, profile, None)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: e }),
            )
        })?;
    Ok(Json(result))
}

pub(crate) async fn admin_restore_original_device(
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let msg = account::restore_original_device().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    Ok(Json(msg))
}

pub(crate) async fn admin_restore_device_version(
    State(_state): State<AppState>,
    Path((account_id, version_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let profile = account::restore_device_version(&account_id, &version_id).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    Ok(Json(profile))
}

pub(crate) async fn admin_delete_device_version(
    State(_state): State<AppState>,
    Path((account_id, version_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    account::delete_device_version(&account_id, &version_id).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    Ok(StatusCode::NO_CONTENT)
}
