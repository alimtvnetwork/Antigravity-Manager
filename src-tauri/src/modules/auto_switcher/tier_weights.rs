//! Candidate Account Tier Weight Normalizer.
//! Calculates normalized tier multipliers (Ultra = 4.0, Pro = 2.0, Free = 1.0) with user-configurable overrides.

use serde::{Deserialize, Serialize};

/// Recognized subscription or quota tier categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountTier {
    Ultra,
    Pro,
    Free,
    Unknown,
}

impl AccountTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ultra => "ultra",
            Self::Pro => "pro",
            Self::Free => "free",
            Self::Unknown => "unknown",
        }
    }
}

/// Configurable multiplier weights across tiers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierWeightConfig {
    pub ultra_weight: f64,
    pub pro_weight: f64,
    pub free_weight: f64,
    pub default_weight: f64,
}

impl Default for TierWeightConfig {
    fn default() -> Self {
        Self {
            ultra_weight: 4.0,
            pro_weight: 2.0,
            free_weight: 1.0,
            default_weight: 1.0,
        }
    }
}

/// Identifies the subscription tier from a tier description or role string.
pub fn resolve_account_tier(raw: &str) -> AccountTier {
    let lower = raw.trim().to_lowercase();
    if lower.contains("ultra") || lower.contains("advanced") || lower.contains("team") {
        AccountTier::Ultra
    } else if lower.contains("pro") || lower.contains("plus") || lower.contains("premium") {
        AccountTier::Pro
    } else if lower.contains("free") || lower.contains("standard") || lower.contains("basic") {
        AccountTier::Free
    } else {
        AccountTier::Unknown
    }
}

/// Returns the normalized candidate score multiplier for the given account tier.
pub fn get_tier_multiplier(tier_str: &str, config: Option<&TierWeightConfig>) -> f64 {
    let tier = resolve_account_tier(tier_str);
    let default_cfg = TierWeightConfig::default();
    let cfg = config.unwrap_or(&default_cfg);

    match tier {
        AccountTier::Ultra => cfg.ultra_weight,
        AccountTier::Pro => cfg.pro_weight,
        AccountTier::Free => cfg.free_weight,
        AccountTier::Unknown => cfg.default_weight,
    }
}
