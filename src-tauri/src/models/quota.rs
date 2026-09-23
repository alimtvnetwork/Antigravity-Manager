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
    /// Successful bucket observation time in milliseconds; absent in older snapshots.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_at: Option<i64>,
    /// First observed early reset, in seconds; normal cycles start seven days before reset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cycle_start: Option<i64>,
    /// Usage recorded by this instance, populated only when returning the account list.
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub cycle_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl QuotaBucket {
    /// Valid current weekly interval, with an inclusive start and exclusive reset.
    pub(crate) fn weekly_cycle_bounds(&self, now: i64) -> Option<(i64, i64)> {
        let window = format!("{} {}", self.window, self.bucket_id).to_lowercase();
        if !(window.contains("week") || window.contains("7d"))
            || !(0.0..=1.0).contains(&self.remaining_fraction)
        {
            return None;
        }
        let end = chrono::DateTime::parse_from_rfc3339(&self.reset_time)
            .ok()?
            .timestamp();
        let normal_start = end.checked_sub(7 * 24 * 60 * 60)?;
        let start = self.cycle_start.unwrap_or(normal_start);
        (normal_start <= start && start <= now && now < end).then_some((start, end))
    }

    /// Called only for a newer observation of the same bucket by the existing merge.
    pub(crate) fn retain_cycle_boundary(&mut self, previous: &Self, observed_at: i64) {
        if self.reset_time == previous.reset_time {
            self.cycle_start = previous.cycle_start;
        }
        let observed_secs = observed_at.div_euclid(1000);
        if self.weekly_cycle_bounds(observed_secs).is_some()
            && previous.weekly_cycle_bounds(observed_secs).is_some()
            && self.remaining_fraction > previous.remaining_fraction + 1e-9
        {
            self.cycle_start = Some(observed_secs);
        }
    }
}

/// A model group (e.g. Gemini Models / Claude and GPT models)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaGroup {
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
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
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_subscription_tier() {
        // Upstream real ID (authoritative field returned by loadCodeAssist)
        assert_eq!(normalize_subscription_tier("free-tier"), "FREE");
        assert_eq!(normalize_subscription_tier("g1-pro-tier"), "PRO");
        assert_eq!(
            normalize_subscription_tier("standard-tier"),
            "standard-tier"
        );
        assert_eq!(normalize_subscription_tier("g1-ultra-tier"), "ULTRA");
        assert_eq!(normalize_subscription_tier("GOOGLE_ONE_HELIUM"), "ULTRA");
        assert_eq!(normalize_subscription_tier("GDP_HELIUM"), "ULTRA");

        // Upstream real name (free text)
        assert_eq!(
            normalize_subscription_tier("Antigravity Starter Quota"),
            "FREE"
        );
        assert_eq!(normalize_subscription_tier("Google AI Pro"), "PRO");
        assert_eq!(normalize_subscription_tier("Google AI Ultra"), "ULTRA");

        // Legacy / compatibility naming
        assert_eq!(normalize_subscription_tier("Google One AI Premium"), "PRO");
        assert_eq!(normalize_subscription_tier("gemini-advanced"), "PRO");
        assert_eq!(normalize_subscription_tier("Gemini Pro"), "PRO");
        assert_eq!(normalize_subscription_tier("pro"), "PRO");
        assert_eq!(normalize_subscription_tier("gemini-ultra"), "ULTRA");
        assert_eq!(normalize_subscription_tier("ULTRA"), "ULTRA");
        assert_eq!(normalize_subscription_tier("Free"), "FREE");

        // Empty / unrecognized returns as-is
        assert_eq!(normalize_subscription_tier(""), "");
        assert_eq!(normalize_subscription_tier("   "), "");
        assert_eq!(
            normalize_subscription_tier("totally-unknown"),
            "totally-unknown"
        );
    }

    #[test]
    fn test_is_known_tier() {
        assert!(is_known_tier("FREE"));
        assert!(is_known_tier("PRO"));
        assert!(is_known_tier("ULTRA"));
        assert!(!is_known_tier(""));
        assert!(!is_known_tier("totally-unknown"));
        assert!(!is_known_tier("pro")); // Unnormalized lowercase is not a valid tier constant
    }

    #[test]
    fn test_resolve_subscription_tier_no_model_fallback() {
        // Critical regression: model list no longer participates in inference.
        // Free accounts also receive the full claude/gpt catalog, so None must resolve to FREE,
        // otherwise free accounts would be erroneously tagged as Pro.
        assert_eq!(resolve_subscription_tier(None), "FREE");

        // Upstream authoritative ID
        assert_eq!(resolve_subscription_tier(Some("free-tier")), "FREE");
        assert_eq!(resolve_subscription_tier(Some("g1-pro-tier")), "PRO");
        assert_eq!(resolve_subscription_tier(Some("g1-ultra-tier")), "ULTRA");

        // Upstream name
        assert_eq!(
            resolve_subscription_tier(Some("Antigravity Starter Quota")),
            "FREE"
        );
        assert_eq!(resolve_subscription_tier(Some("Google AI Pro")), "PRO");

        // Unrecognized -> FREE (never guess PRO)
        assert_eq!(resolve_subscription_tier(Some("totally-unknown")), "FREE");
        assert_eq!(resolve_subscription_tier(Some("")), "FREE");
    }

    #[test]
    fn test_tier_priority() {
        assert_eq!(tier_priority(Some("ULTRA")), 0);
        assert_eq!(tier_priority(Some("g1-ultra-tier")), 0);
        assert_eq!(tier_priority(Some("PRO")), 1);
        assert_eq!(tier_priority(Some("g1-pro-tier")), 1);
        assert_eq!(tier_priority(Some("FREE")), 2);
        assert_eq!(tier_priority(Some("free-tier")), 2);

        // Unknown / missing tier treated as FREE
        assert_eq!(tier_priority(None), 2);
        assert_eq!(tier_priority(Some("")), 2);
        assert_eq!(tier_priority(Some("garbage")), 2);
    }

    #[test]
    fn test_ensure_subscription_tier_normalizes_and_drops_unknown() {
        let mut quota = QuotaData::new();

        // Upstream name normalized
        quota.subscription_tier = Some("Antigravity Starter Quota".to_string());
        quota.ensure_subscription_tier();
        assert_eq!(quota.subscription_tier.as_deref(), Some("FREE"));

        quota.subscription_tier = Some("g1-pro-tier".to_string());
        quota.ensure_subscription_tier();
        assert_eq!(quota.subscription_tier.as_deref(), Some("PRO"));

        // Unrecognized -> emptied, waiting for upstream backfill (never falls back to PRO)
        quota.subscription_tier = Some("totally-unknown".to_string());
        quota.ensure_subscription_tier();
        assert_eq!(quota.subscription_tier, None);

        // None stays None
        quota.subscription_tier = None;
        quota.ensure_subscription_tier();
        assert_eq!(quota.subscription_tier, None);
    }

    #[test]
    fn test_quota_group_deserialization_with_missing_buckets() {
        let json = r#"{"display_name":"Gemini Models"}"#;
        let group: QuotaGroup =
            serde_json::from_str(json).expect("Should deserialize with missing buckets");
        assert_eq!(group.display_name, "Gemini Models");
        assert!(group.buckets.is_empty());
    }
}
