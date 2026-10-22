//! Account tier priority weight normalizer.
//! Maps account subscription tiers (Ultra, Pro, Free, Sandbox) to calibrated
//! scoring multiplier factors to enforce subscription-based priority ordering.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubscriptionTier {
    Ultra,
    Pro,
    Standard,
    Free,
    Sandbox,
}

impl SubscriptionTier {
    pub fn from_str_loose(tier: &str) -> Self {
        let t = tier.trim().to_ascii_lowercase();
        if t.contains("ultra") || t.contains("enterprise") {
            SubscriptionTier::Ultra
        } else if t.contains("pro") || t.contains("plus") {
            SubscriptionTier::Pro
        } else if t.contains("sandbox") || t.contains("test") {
            SubscriptionTier::Sandbox
        } else if t.contains("free") {
            SubscriptionTier::Free
        } else {
            SubscriptionTier::Standard
        }
    }

    /// Priority multiplier applied to composite candidate score.
    pub fn score_multiplier(&self) -> f64 {
        match self {
            SubscriptionTier::Ultra => 1.30,
            SubscriptionTier::Pro => 1.15,
            SubscriptionTier::Standard => 1.00,
            SubscriptionTier::Free => 0.85,
            SubscriptionTier::Sandbox => 0.50,
        }
    }
}

pub struct TierPriorityNormalizer;

impl TierPriorityNormalizer {
    /// Returns the normalized scoring multiplier for an account tier string.
    pub fn get_multiplier(tier_name: &str) -> f64 {
        SubscriptionTier::from_str_loose(tier_name).score_multiplier()
    }
}
