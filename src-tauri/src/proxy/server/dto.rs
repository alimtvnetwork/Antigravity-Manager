//! Admin API data-transfer objects and model-mapping helpers.
use super::app_state::AppState;
use axum::extract::FromRef;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// Implement FromRef for AppState so middleware can extract security state
impl axum::extract::FromRef<AppState> for Arc<RwLock<crate::proxy::ProxySecurityConfig>> {
    fn from_ref(state: &AppState) -> Self {
        state.security.clone()
    }
}

#[derive(Serialize)]
pub(crate) struct ErrorResponse {
    pub(crate) error: String,
}

#[derive(Serialize)]
pub(crate) struct AccountResponse {
    pub(crate) id: String,
    pub(crate) email: String,
    pub(crate) name: Option<String>,
    pub(crate) priority: u8,
    pub(crate) is_current: bool,
    pub(crate) disabled: bool,
    pub(crate) disabled_reason: Option<String>,
    pub(crate) disabled_at: Option<i64>,
    pub(crate) proxy_disabled: bool,
    pub(crate) proxy_disabled_reason: Option<String>,
    pub(crate) proxy_disabled_at: Option<i64>,
    pub(crate) protected_models: Vec<String>,
    pub(crate) live_limited_models: HashMap<String, crate::models::account::LiveLimitStatus>,
    /// [NEW] 403 validation blocked state
    pub(crate) validation_blocked: bool,
    pub(crate) validation_blocked_until: Option<i64>,
    pub(crate) validation_blocked_reason: Option<String>,
    pub(crate) quota: Option<QuotaResponse>,
    pub(crate) device_bound: bool,
    pub(crate) last_used: i64,
}

#[derive(Serialize)]
pub(crate) struct QuotaResponse {
    pub(crate) models: Vec<ModelQuota>,
    pub(crate) last_updated: i64,
    pub(crate) subscription_tier: Option<String>,
    pub(crate) is_forbidden: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) quota_groups: Option<Vec<QuotaGroupDto>>,
}

#[derive(Serialize)]
pub(crate) struct ModelQuota {
    pub(crate) name: String,
    pub(crate) percentage: i32,
    pub(crate) reset_time: String,
}

#[derive(Serialize)]
pub(crate) struct QuotaGroupDto {
    pub(crate) display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) description: Option<String>,
    pub(crate) buckets: Vec<QuotaBucketDto>,
}

#[derive(Serialize)]
pub(crate) struct QuotaBucketDto {
    pub(crate) bucket_id: String,
    pub(crate) window: String,
    pub(crate) remaining_fraction: f64,
    pub(crate) reset_time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) cycle_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) description: Option<String>,
}

/// Map a model-side QuotaGroup to the API DTO.
pub(crate) fn quota_group_to_dto(g: &crate::models::quota::QuotaGroup) -> QuotaGroupDto {
    QuotaGroupDto {
        display_name: g.display_name.clone(),
        description: g.description.clone(),
        buckets: g
            .buckets
            .iter()
            .map(|b| QuotaBucketDto {
                bucket_id: b.bucket_id.clone(),
                window: b.window.clone(),
                remaining_fraction: b.remaining_fraction,
                reset_time: b.reset_time.clone(),
                cycle_tokens: b.cycle_tokens,
                display_name: b.display_name.clone(),
                description: b.description.clone(),
            })
            .collect(),
    }
}

#[derive(Serialize)]
pub(crate) struct AccountListResponse {
    pub(crate) accounts: Vec<AccountResponse>,
    pub(crate) current_account_id: Option<String>,
}

pub(crate) fn to_account_response(
    account: &crate::models::account::Account,
    current_id: &Option<String>,
) -> AccountResponse {
    AccountResponse {
        id: account.id.clone(),
        email: account.email.clone(),
        name: account.name.clone(),
        priority: account.priority,
        is_current: current_id.as_ref() == Some(&account.id),
        disabled: account.disabled,
        disabled_reason: account.disabled_reason.clone(),
        disabled_at: account.disabled_at,
        proxy_disabled: account.proxy_disabled,
        proxy_disabled_reason: account.proxy_disabled_reason.clone(),
        proxy_disabled_at: account.proxy_disabled_at,
        protected_models: account.protected_models.iter().cloned().collect(),
        live_limited_models: account.live_limited_models.clone(),
        quota: account.quota.as_ref().map(|q| QuotaResponse {
            models: q
                .models
                .iter()
                .map(|m| ModelQuota {
                    name: m.name.clone(),
                    percentage: m.percentage,
                    reset_time: m.reset_time.clone(),
                })
                .collect(),
            last_updated: q.last_updated,
            subscription_tier: q.subscription_tier.clone(),
            is_forbidden: q.is_forbidden,
            quota_groups: q
                .quota_groups
                .as_ref()
                .map(|groups| groups.iter().map(quota_group_to_dto).collect()),
        }),
        device_bound: account.device_profile.is_some(),
        last_used: account.last_used,
        validation_blocked: account.validation_blocked,
        validation_blocked_until: account.validation_blocked_until,
        validation_blocked_reason: account.validation_blocked_reason.clone(),
    }
}
