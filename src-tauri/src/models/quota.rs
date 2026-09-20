use serde::{Deserialize, Serialize};

/// Individual quota bucket (corresponding to a bucket in retrieveUserQuotaSummary)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaBucket {
    /// Bucket ID, e.g. "gemini-weekly" / "gemini-5h" / "3p-weekly" / "3p-5h"
    pub bucket_id: String,
    /// Window type: "weekly" / "5h"
    pub window: String,
    /// Remaining fraction 0.0-1.0
    pub remaining_fraction: f64,
    /// Reset time (RFC3339)
    pub reset_time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// A model group (e.g. Gemini Models / Claude and GPT models)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaGroup {
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub buckets: Vec<QuotaBucket>,
}

/// Model quota information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelQuota {
    pub name: String,
    pub percentage: i32, // Remaining percentage 0-100
    pub reset_time: String,

    // -- Dynamic parameter parsing and persistence --
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_images: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_thinking: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_budget: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommended: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_mime_types: Option<std::collections::HashMap<String, bool>>,
}

/// Quota data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaData {
    pub models: Vec<ModelQuota>,
    pub last_updated: i64,
    #[serde(default)]
    pub is_forbidden: bool,
    /// Forbidden reason (403 details)
    #[serde(default)]
    pub forbidden_reason: Option<String>,
    /// Subscription tier (FREE/PRO/ULTRA)
    #[serde(default)]
    pub subscription_tier: Option<String>,
    /// Model retirement redirection rules (old_model_id -> new_model_id)
    #[serde(default)]
    pub model_forwarding_rules: std::collections::HashMap<String, String>,
    /// Quota summary by model group (weekly + 5h dual windows) from retrieveUserQuotaSummary
    #[serde(default)]
    pub quota_groups: Option<Vec<QuotaGroup>>,
}

impl QuotaData {
    pub fn new() -> Self {
        Self {
            models: Vec::new(),
            last_updated: chrono::Utc::now().timestamp(),
            is_forbidden: false,
            forbidden_reason: None,
            subscription_tier: None,
            model_forwarding_rules: std::collections::HashMap::new(),
            quota_groups: None,
        }
    }

    pub fn add_model(&mut self, model: ModelQuota) {
        self.models.push(model);
    }

    pub fn ensure_subscription_tier(&mut self) {
        self.subscription_tier = match self.subscription_tier.as_deref() {
            Some(tier) => {
                let normalized = normalize_subscription_tier(tier);
                if is_known_tier(&normalized) {
                    Some(normalized)
                } else {
                    None
                }
            }
            None => None,
        };
    }
}

/// Check if tier matches known values (ULTRA, PRO, FREE)
pub fn is_known_tier(tier: &str) -> bool {
    matches!(tier, "ULTRA" | "PRO" | "FREE")
}

/// Normalize subscription tier into "ULTRA" | "PRO" | "FREE"
pub fn normalize_subscription_tier(tier: &str) -> String {
    let lower = tier.trim().to_lowercase();
    if lower.is_empty() {
        return String::new();
    }

    if lower.contains("ultra") || lower.contains("helium") {
        return "ULTRA".to_string();
    }

    if lower.contains("free") || lower.contains("starter") {
        return "FREE".to_string();
    }

    if lower.contains("pro") || lower.contains("premium") || lower.contains("advanced") {
        return "PRO".to_string();
    }

    tier.to_string()
}

/// Resolve subscription tier, defaulting to FREE if unrecognized
pub fn resolve_subscription_tier(raw_tier: Option<&str>) -> String {
    if let Some(tier) = raw_tier {
        let normalized = normalize_subscription_tier(tier);
        if is_known_tier(&normalized) {
            return normalized;
        }
    }
    "FREE".to_string()
}

/// Tier priority for scheduler polling (ULTRA=0, PRO=1, FREE=2)
pub fn tier_priority(tier: Option<&str>) -> u8 {
    match normalize_subscription_tier(tier.unwrap_or("")).as_str() {
        "ULTRA" => 0,
        "PRO" => 1,
        _ => 2,
    }
}

impl Default for QuotaData {
    fn default() -> Self {
        Self::new()
    }
}
