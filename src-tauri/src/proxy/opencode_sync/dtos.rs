use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpencodeStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub is_synced: bool,
    pub has_backup: bool,
    pub current_base_url: Option<String>,
    pub files: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct CanonicalFamilyDto {
    pub canonical_id: String,
    pub display_name: String,
    pub match_ids: Vec<String>,
}

/// Plugin schema v3 account structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct PluginAccount {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) email: Option<String>,
    #[serde(rename = "refreshToken")]
    pub(crate) refresh_token: String,
    #[serde(default, rename = "projectId", skip_serializing_if = "Option::is_none")]
    pub(crate) project_id: Option<String>,
    #[serde(rename = "addedAt")]
    pub(crate) added_at: i64,
    #[serde(rename = "lastUsed")]
    pub(crate) last_used: i64,
    #[serde(
        rename = "rateLimitResetTimes",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) rate_limit_reset_times: Option<HashMap<String, i64>>,
    // Optional preserved state fields
    #[serde(rename = "managedProjectId", skip_serializing_if = "Option::is_none")]
    pub(crate) managed_project_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) enabled: Option<bool>,
    #[serde(rename = "lastSwitchReason", skip_serializing_if = "Option::is_none")]
    pub(crate) last_switch_reason: Option<String>,
    #[serde(rename = "coolingDownUntil", skip_serializing_if = "Option::is_none")]
    pub(crate) cooling_down_until: Option<i64>,
    #[serde(rename = "cooldownReason", skip_serializing_if = "Option::is_none")]
    pub(crate) cooldown_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) fingerprint: Option<Value>,
    #[serde(rename = "cachedQuota", skip_serializing_if = "Option::is_none")]
    pub(crate) cached_quota: Option<Value>,
    #[serde(
        rename = "cachedQuotaUpdatedAt",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) cached_quota_updated_at: Option<i64>,
    #[serde(rename = "fingerprintHistory", skip_serializing_if = "Option::is_none")]
    pub(crate) fingerprint_history: Option<Value>,
}

/// Plugin schema v3 accounts file structure
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct PluginAccountsFile {
    pub(crate) version: i32,
    pub(crate) accounts: Vec<PluginAccount>,
    #[serde(rename = "activeIndex")]
    pub(crate) active_index: i32,
    #[serde(rename = "activeIndexByFamily")]
    pub(crate) active_index_by_family: HashMap<String, i32>,
}
