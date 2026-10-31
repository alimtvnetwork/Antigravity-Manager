//! Multi-Tier Account Failover Ladder
//! Categorizes candidate accounts into structured reliability tiers
//! (Primary -> Secondary -> Sandbox -> EmergencyQuota) when top-ranked accounts are exhausted.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum FailoverTier {
    Primary = 1,
    Secondary = 2,
    Sandbox = 3,
    EmergencyQuota = 4,
}

#[derive(Debug, Clone)]
pub struct FailoverLadder;

impl FailoverLadder {
    /// Classifies an account candidate into the appropriate failover tier
    /// based on 4-hour quota, weekly quota, and cooldown status.
    pub fn classify_tier(
        four_hour_quota: f64,
        weekly_quota: f64,
        is_cooldown: bool,
        is_sandbox: bool,
    ) -> Option<FailoverTier> {
        if is_cooldown {
            return None;
        }

        if is_sandbox {
            return Some(FailoverTier::Sandbox);
        }

        // Tier 1 (Primary): 100% 4-hour quota headroom
        if four_hour_quota >= 99.5 {
            return Some(FailoverTier::Primary);
        }

        // Tier 2 (Secondary): Substantial 4-hour headroom (>20%) and healthy weekly quota (>30%)
        if four_hour_quota >= 20.0 && weekly_quota >= 30.0 {
            return Some(FailoverTier::Secondary);
        }

        // Tier 4 (EmergencyQuota): Any positive quota headroom (>1%)
        if four_hour_quota > 1.0 {
            return Some(FailoverTier::EmergencyQuota);
        }

        None
    }

    /// Selects the best available tier from a collection of candidate classifications.
    pub fn lowest_tier_available(tiers: &[FailoverTier]) -> Option<FailoverTier> {
        tiers.iter().min().copied()
    }
}
