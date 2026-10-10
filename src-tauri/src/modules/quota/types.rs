use serde::{Deserialize, Serialize};

use super::*;

pub(crate) const QUOTA_API_ENDPOINTS: [&str; 3] = [
    "https://daily-cloudcode-pa.googleapis.com/v1internal:fetchAvailableModels",
    "https://daily-cloudcode-pa.sandbox.googleapis.com/v1internal:fetchAvailableModels",
    "https://cloudcode-pa.googleapis.com/v1internal:fetchAvailableModels",
];

// Quota Summary API endpoints (weekly + 5h grouped quota, fallback order: Daily → Sandbox → Prod)
// 顺序理由同上：优先与官方客户端一致的 Daily 端点。
pub(crate) const QUOTA_SUMMARY_ENDPOINTS: [&str; 3] = [
    "https://daily-cloudcode-pa.googleapis.com/v1internal:retrieveUserQuotaSummary",
    "https://daily-cloudcode-pa.sandbox.googleapis.com/v1internal:retrieveUserQuotaSummary",
    "https://cloudcode-pa.googleapis.com/v1internal:retrieveUserQuotaSummary",
];

/// Critical retry threshold: considered near recovery when quota reaches 95%
pub(crate) const NEAR_READY_THRESHOLD: i32 = 95;

pub(crate) const MAX_RETRIES: u32 = 3;

pub(crate) const RETRY_DELAY_SECS: u64 = 30;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct QuotaResponse {
    pub(crate) models: std::collections::HashMap<String, ModelInfo>,
    #[serde(rename = "deprecatedModelIds")]
    pub(crate) deprecated_model_ids: Option<std::collections::HashMap<String, DeprecatedModelInfo>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct DeprecatedModelInfo {
    #[serde(rename = "newModelId")]
    pub(crate) new_model_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ModelInfo {
    #[serde(rename = "quotaInfo")]
    pub(crate) quota_info: Option<QuotaInfo>,
    #[serde(rename = "displayName")]
    pub(crate) display_name: Option<String>,
    #[serde(rename = "supportsImages")]
    pub(crate) supports_images: Option<bool>,
    #[serde(rename = "supportsThinking")]
    pub(crate) supports_thinking: Option<bool>,
    #[serde(rename = "thinkingBudget")]
    pub(crate) thinking_budget: Option<i32>,
    pub(crate) recommended: Option<bool>,
    #[serde(rename = "maxTokens")]
    pub(crate) max_tokens: Option<i32>,
    #[serde(rename = "maxOutputTokens")]
    pub(crate) max_output_tokens: Option<i32>,
    #[serde(rename = "supportedMimeTypes")]
    pub(crate) supported_mime_types: Option<std::collections::HashMap<String, bool>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct QuotaInfo {
    #[serde(rename = "remainingFraction")]
    pub(crate) remaining_fraction: Option<f64>,
    #[serde(rename = "resetTime")]
    pub(crate) reset_time: Option<String>,
}

// ---- retrieveUserQuotaSummary response deserialization structures ----

#[derive(Debug, Deserialize)]
pub(crate) struct QuotaSummaryResponse {
    pub(crate) groups: Vec<QuotaSummaryGroup>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct QuotaSummaryGroup {
    #[serde(rename = "displayName")]
    pub(crate) display_name: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) buckets: Vec<QuotaSummaryBucket>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct QuotaSummaryBucket {
    #[serde(rename = "bucketId")]
    pub(crate) bucket_id: Option<String>,
    pub(crate) window: Option<String>,
    #[serde(rename = "remainingFraction")]
    pub(crate) remaining_fraction: Option<f64>,
    #[serde(rename = "resetTime")]
    pub(crate) reset_time: Option<String>,
    #[serde(rename = "displayName")]
    pub(crate) display_name: Option<String>,
    pub(crate) description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LoadProjectResponse {
    #[serde(rename = "cloudaicompanionProject")]
    pub(crate) project_id: Option<String>,
    #[serde(rename = "currentTier")]
    pub(crate) current_tier: Option<Tier>,
    #[serde(rename = "paidTier")]
    pub(crate) paid_tier: Option<Tier>,
    #[serde(rename = "allowedTiers")]
    pub(crate) allowed_tiers: Option<Vec<Tier>>,
    #[serde(rename = "ineligibleTiers")]
    pub(crate) ineligible_tiers: Option<Vec<IneligibleTier>>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct IneligibleTier {
    #[allow(dead_code)]
    #[serde(rename = "reasonCode")]
    pub(crate) reason_code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Tier {
    #[serde(rename = "isDefault", alias = "is_default")]
    pub(crate) is_default: Option<bool>,
    pub(crate) id: Option<String>,
    #[serde(rename = "quotaTier", alias = "quota_tier")]
    pub(crate) quota_tier: Option<String>,
    pub(crate) name: Option<String>,
    pub(crate) slug: Option<String>,
}

pub(crate) fn tier_text(tier: &Tier) -> Option<String> {
    [
        tier.name.clone(),
        tier.id.clone(),
        tier.slug.clone(),
        tier.quota_tier.clone(),
    ]
    .into_iter()
    .find(|value| value.as_ref().is_some_and(|text| !text.trim().is_empty()))
    .flatten()
}

pub(crate) fn finalize_subscription_tier(raw: Option<String>, email: &str) -> Option<String> {
    let raw = raw?;
    let normalized = crate::models::quota::normalize_subscription_tier(&raw);
    if crate::models::quota::is_known_tier(&normalized) {
        crate::modules::logger::log_info(&format!(
            "📊 [{}] Subscription identified successfully: {} (raw: {})",
            email, normalized, raw
        ));
        Some(normalized)
    } else {
        crate::modules::logger::log_warn(&format!(
            "📊 [{}] Unrecognized subscription tier id '{}'; not mapped to PRO",
            email, raw
        ));
        Some(raw)
    }
}

/// Get shared HTTP Client (15s timeout) for pure info fetching (No JA3)
async fn create_standard_client(account_id: Option<&str>) -> rquest::Client {
    if let Some(pool) = crate::proxy::proxy_pool::get_global_proxy_pool() {
        pool.get_effective_standard_client(account_id, 15).await
    } else {
        crate::utils::http::get_standard_client()
    }
}

/// Get shared HTTP Client (60s timeout) for pure info fetching (No JA3)
#[allow(dead_code)] // Reserved for warmup and background tasks
async fn create_long_standard_client(account_id: Option<&str>) -> rquest::Client {
    if let Some(pool) = crate::proxy::proxy_pool::get_global_proxy_pool() {
        pool.get_effective_standard_client(account_id, 60).await
    } else {
        crate::utils::http::get_long_standard_client()
    }
}

// 项目 / 档位解析端点（fallback order: Daily → Sandbox → Prod）
// [FIX Issue #3525] 与官方 Antigravity language_server 的出站端点保持一致，理由见上文 QUOTA_API_ENDPOINTS。
pub(crate) const CLOUD_CODE_LOAD_PROJECT_ENDPOINTS: [&str; 3] = [
    "https://daily-cloudcode-pa.googleapis.com/v1internal:loadCodeAssist",
    "https://daily-cloudcode-pa.sandbox.googleapis.com/v1internal:loadCodeAssist",
    "https://cloudcode-pa.googleapis.com/v1internal:loadCodeAssist",
];
