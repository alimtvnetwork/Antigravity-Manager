use crate::modules::{account, logger, proxy_db};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::*;

/// Server State
#[derive(Clone)]
pub struct ApiState {
    /// Whether there is a switch operation currently in progress
    pub switching: Arc<RwLock<bool>>,
    pub integration: crate::modules::integration::SystemManager,
}

impl ApiState {
    pub fn new(integration: crate::modules::integration::SystemManager) -> Self {
        Self {
            switching: Arc::new(RwLock::new(false)),
            integration,
        }
    }
}

// ============================================================================
// Response Types
// ============================================================================

#[derive(Serialize)]
pub(crate) struct HealthResponse {
    pub(crate) status: String,
    pub(crate) version: String,
}

#[derive(Serialize)]
pub(crate) struct AccountResponse {
    pub(crate) id: String,
    pub(crate) email: String,
    pub(crate) name: Option<String>,
    pub(crate) is_current: bool,
    pub(crate) disabled: bool,
    pub(crate) live_limited_models: HashMap<String, crate::models::account::LiveLimitStatus>,
    pub(crate) quota: Option<QuotaResponse>,
    pub(crate) device_bound: bool,
    pub(crate) last_used: i64,
}

#[derive(Serialize)]
pub(crate) struct QuotaResponse {
    pub(crate) models: Vec<ModelQuota>,
    pub(crate) updated_at: Option<i64>,
    pub(crate) subscription_tier: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct ModelQuota {
    pub(crate) name: String,
    pub(crate) percentage: i32,
    pub(crate) reset_time: String,
}

#[derive(Serialize)]
pub(crate) struct AccountListResponse {
    pub(crate) accounts: Vec<AccountResponse>,
    pub(crate) current_account_id: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct CurrentAccountResponse {
    pub(crate) account: Option<AccountResponse>,
}

#[derive(Serialize)]
pub(crate) struct SwitchResponse {
    pub(crate) success: bool,
    pub(crate) message: String,
}

#[derive(Serialize)]
pub(crate) struct RefreshResponse {
    pub(crate) success: bool,
    pub(crate) message: String,
    pub(crate) refreshed_count: usize,
}

#[derive(Serialize)]
pub(crate) struct BindDeviceResponse {
    pub(crate) success: bool,
    pub(crate) message: String,
    pub(crate) device_profile: Option<DeviceProfileResponse>,
}

#[derive(Serialize)]
pub(crate) struct DeviceProfileResponse {
    pub(crate) machine_id: String,
    pub(crate) mac_machine_id: String,
    pub(crate) dev_device_id: String,
    pub(crate) sqm_id: String,
}

#[derive(Serialize)]
pub(crate) struct ErrorResponse {
    pub(crate) error: String,
}

#[derive(Serialize)]
pub(crate) struct LogsResponse {
    pub(crate) total: u64,
    pub(crate) logs: Vec<crate::proxy::monitor::ProxyRequestLog>,
}

// ============================================================================
// Request Types
// ============================================================================

#[derive(Deserialize)]
pub(crate) struct SwitchRequest {
    pub(crate) account_id: String,
}

#[derive(Deserialize)]
pub(crate) struct BindDeviceRequest {
    #[serde(default = "default_bind_mode")]
    pub(crate) mode: String,
}

pub(crate) fn default_bind_mode() -> String {
    "generate".to_string()
}

#[derive(Deserialize)]
pub(crate) struct LogsRequest {
    #[serde(default)]
    pub(crate) limit: usize,
    #[serde(default)]
    pub(crate) offset: usize,
    #[serde(default)]
    pub(crate) filter: String,
    #[serde(default)]
    pub(crate) errors_only: bool,
}
