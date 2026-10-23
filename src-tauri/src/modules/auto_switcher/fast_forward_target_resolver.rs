//! Fast-Forward Account Target Resolver.
//!
//! Implements intelligent, high-velocity account rotation for developer IDE instances
//! and daemon workers when an active account encounters quota exhaustion or requires
//! fast-forward rotation according to the 4-tier decision ladder:
//!
//! 1. **Baseline Exclusion & Weekly Quota Floor**:
//!    - Filter out disabled accounts (`is_disabled`).
//!    - Filter out current instance account (`id == current_instance_account_id`).
//!    - If `weekly_quota_percent < config.min_weekly_quota_percent`, treat as 0 credits and skip unconditionally.
//!    - Filter out accounts currently active on other instances (`is_active_on_other_instance`).
//!
//! 2. **Primary Tier (100% 4h Credit & Clean 4h Fleet Usage)**:
//!    - Candidates with `rolling_4h_quota_percent >= 99.5` (full credit with floating tolerance)
//!    - AND no usage in the last 4 hours by any source (SSH, Email, GitMap, Telegram, or local instances:
//!      `now_epoch_seconds.saturating_sub(last_used_epoch_seconds) >= fleet_usage_window_seconds || last_used_epoch_seconds == 0`).
//!    - Sorted ascending by `reset_epoch_seconds` (closest time to expire).
//!    - Ties broken by higher `weekly_quota_percent`, then higher `tier_multiplier`.
//!
//! 3. **Degraded Fallback (Highest Combined Credits)**:
//!    - If no primary candidate qualifies, score all baseline-eligible candidates:
//!      `tier_multiplier * (weekly_quota_percent + rolling_4h_quota_percent)`.
//!    - Sorted descending by score.
//!
//! 4. **No Candidates**:
//!    - Return descriptive `FfResolutionOutcome::NoEligibleCandidates`.

use serde::{Deserialize, Serialize};

/// Fast-Forward candidate account descriptor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FfCandidateAccount {
    pub id: String,
    pub email: String,
    pub weekly_quota_percent: f64,
    pub rolling_4h_quota_percent: f64,
    pub reset_epoch_seconds: u64,
    pub last_used_epoch_seconds: u64,
    pub is_active_on_other_instance: bool,
    pub tier_multiplier: f64,
    pub is_disabled: bool,
}

/// Backward compatibility alias for `FfCandidateAccount`.
pub type FfCandidate = FfCandidateAccount;

impl Default for FfCandidateAccount {
    fn default() -> Self {
        Self {
            id: String::new(),
            email: String::new(),
            weekly_quota_percent: 100.0,
            rolling_4h_quota_percent: 100.0,
            reset_epoch_seconds: 0,
            last_used_epoch_seconds: 0,
            is_active_on_other_instance: false,
            tier_multiplier: 1.0,
            is_disabled: false,
        }
    }
}

impl FfCandidateAccount {
    pub fn new(id: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            email: email.into(),
            ..Default::default()
        }
    }

    pub fn with_weekly_quota(mut self, percent: f64) -> Self {
        self.weekly_quota_percent = percent;
        self
    }

    pub fn with_rolling_4h_quota(mut self, percent: f64) -> Self {
        self.rolling_4h_quota_percent = percent;
        self
    }

    pub fn with_reset_epoch(mut self, epoch_sec: u64) -> Self {
        self.reset_epoch_seconds = epoch_sec;
        self
    }

    pub fn with_last_used_epoch(mut self, epoch_sec: u64) -> Self {
        self.last_used_epoch_seconds = epoch_sec;
        self
    }

    pub fn with_active_on_other_instance(mut self, active: bool) -> Self {
        self.is_active_on_other_instance = active;
        self
    }

    pub fn with_tier_multiplier(mut self, multiplier: f64) -> Self {
        self.tier_multiplier = multiplier;
        self
    }

    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.is_disabled = disabled;
        self
    }
}

/// Configuration parameters for Fast-Forward target account resolution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FfResolutionConfig {
    pub min_weekly_quota_percent: f64,
    pub require_full_4h_headroom: bool,
    pub fleet_usage_window_seconds: u64,
    pub now_epoch_seconds: u64,
}

impl Default for FfResolutionConfig {
    fn default() -> Self {
        Self {
            min_weekly_quota_percent: 15.0,
            require_full_4h_headroom: true,
            fleet_usage_window_seconds: 14400,
            now_epoch_seconds: 0,
        }
    }
}

impl FfResolutionConfig {
    pub fn new(now_epoch_seconds: u64) -> Self {
        Self {
            now_epoch_seconds,
            ..Default::default()
        }
    }

    pub fn with_min_weekly_quota_percent(mut self, percent: f64) -> Self {
        self.min_weekly_quota_percent = percent;
        self
    }

    pub fn with_require_full_4h_headroom(mut self, require: bool) -> Self {
        self.require_full_4h_headroom = require;
        self
    }

    pub fn with_fleet_usage_window(mut self, seconds: u64) -> Self {
        self.fleet_usage_window_seconds = seconds;
        self
    }
}

/// Selection tier classification explaining why an account was chosen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FfSelectionTier {
    PrimaryFullHeadroomClosestExpiry,
    FallbackHighestCombinedQuota,
}

/// Resolution outcome for Fast-Forward targeting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FfResolutionOutcome {
    FoundCandidate {
        account_id: String,
        email: String,
        selection_tier: FfSelectionTier,
        score: f64,
    },
    NoEligibleCandidates {
        reason: String,
    },
}

impl FfResolutionOutcome {
    pub fn is_found(&self) -> bool {
        matches!(self, Self::FoundCandidate { .. })
    }

    pub fn candidate_id(&self) -> Option<&str> {
        match self {
            Self::FoundCandidate { account_id, .. } => Some(account_id),
            Self::NoEligibleCandidates { .. } => None,
        }
    }
}

/// Resolves the optimal target account for Fast-Forward switching based on the 4-tier decision ladder.
pub fn resolve_fast_forward_target(
    candidates: &[FfCandidateAccount],
    current_instance_account_id: Option<&str>,
    config: &FfResolutionConfig,
) -> FfResolutionOutcome {
    // Step 1: Baseline Exclusion & Weekly Quota Floor
    let baseline_eligible: Vec<&FfCandidateAccount> = candidates
        .iter()
        .filter(|c| {
            if c.is_disabled {
                return false;
            }
            if let Some(curr_id) = current_instance_account_id {
                if c.id == curr_id {
                    return false;
                }
            }
            if c.is_active_on_other_instance {
                return false;
            }
            // Weekly quota floor: if weekly_quota_percent < min_weekly_quota_percent,
            // treat as 0 credits and skip unconditionally!
            if c.weekly_quota_percent < config.min_weekly_quota_percent {
                return false;
            }
            true
        })
        .collect();

    if baseline_eligible.is_empty() {
        return FfResolutionOutcome::NoEligibleCandidates {
            reason: format!(
                "No eligible accounts found: all candidates are disabled, active on other instances, currently bound, or below weekly quota floor ({:.1}%)",
                config.min_weekly_quota_percent
            ),
        };
    }

    // Step 2: Primary Tier (100% 4h Credit & Zero 4h Fleet Usage)
    if config.require_full_4h_headroom {
        let mut primary_candidates: Vec<&FfCandidateAccount> = baseline_eligible
            .iter()
            .copied()
            .filter(|c| {
                let has_full_4h_quota = c.rolling_4h_quota_percent >= 99.5;
                let is_clean_4h_fleet = c.last_used_epoch_seconds == 0
                    || config
                        .now_epoch_seconds
                        .saturating_sub(c.last_used_epoch_seconds)
                        >= config.fleet_usage_window_seconds;
                has_full_4h_quota && is_clean_4h_fleet
            })
            .collect();

        if !primary_candidates.is_empty() {
            // Sort ascending by reset_epoch_seconds (closest time to expire).
            // Break ties by higher weekly_quota_percent, then higher tier_multiplier.
            primary_candidates.sort_by(|a, b| {
                a.reset_epoch_seconds
                    .cmp(&b.reset_epoch_seconds)
                    .then_with(|| {
                        b.weekly_quota_percent
                            .partial_cmp(&a.weekly_quota_percent)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .then_with(|| {
                        b.tier_multiplier
                            .partial_cmp(&a.tier_multiplier)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .then_with(|| a.id.cmp(&b.id))
            });

            let best = primary_candidates[0];
            let score = best.tier_multiplier
                * (best.weekly_quota_percent + best.rolling_4h_quota_percent);
            return FfResolutionOutcome::FoundCandidate {
                account_id: best.id.clone(),
                email: best.email.clone(),
                selection_tier: FfSelectionTier::PrimaryFullHeadroomClosestExpiry,
                score,
            };
        }
    }

    // Step 3: Degraded Fallback (Highest Combined Credits)
    let mut fallback_candidates = baseline_eligible;
    fallback_candidates.sort_by(|a, b| {
        let score_a =
            a.tier_multiplier * (a.weekly_quota_percent + a.rolling_4h_quota_percent);
        let score_b =
            b.tier_multiplier * (b.weekly_quota_percent + b.rolling_4h_quota_percent);

        score_b
            .partial_cmp(&score_a)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                b.weekly_quota_percent
                    .partial_cmp(&a.weekly_quota_percent)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| {
                b.rolling_4h_quota_percent
                    .partial_cmp(&a.rolling_4h_quota_percent)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| a.id.cmp(&b.id))
    });

    let best = fallback_candidates[0];
    let score =
        best.tier_multiplier * (best.weekly_quota_percent + best.rolling_4h_quota_percent);
    FfResolutionOutcome::FoundCandidate {
        account_id: best.id.clone(),
        email: best.email.clone(),
        selection_tier: FfSelectionTier::FallbackHighestCombinedQuota,
        score,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weekly_quota_floor_skips_sub_15_percent() {
        let config = FfResolutionConfig::default();
        let candidates = vec![
            FfCandidateAccount::new("acc-1", "low@test.com")
                .with_weekly_quota(14.9)
                .with_rolling_4h_quota(100.0),
            FfCandidateAccount::new("acc-2", "valid@test.com")
                .with_weekly_quota(15.0)
                .with_rolling_4h_quota(100.0),
        ];

        let result = resolve_fast_forward_target(&candidates, None, &config);
        match result {
            FfResolutionOutcome::FoundCandidate { account_id, .. } => {
                assert_eq!(account_id, "acc-2");
            }
            _ => panic!("Expected FoundCandidate"),
        }
    }

    #[test]
    fn test_sub_100_percent_4h_quota_skips_primary_tier() {
        let config = FfResolutionConfig::default();
        let candidates = vec![FfCandidateAccount::new("acc-1", "user@test.com")
            .with_weekly_quota(80.0)
            .with_rolling_4h_quota(95.0)];

        let result = resolve_fast_forward_target(&candidates, None, &config);
        match result {
            FfResolutionOutcome::FoundCandidate {
                account_id,
                selection_tier,
                ..
            } => {
                assert_eq!(account_id, "acc-1");
                assert_eq!(
                    selection_tier,
                    FfSelectionTier::FallbackHighestCombinedQuota
                );
            }
            _ => panic!("Expected FallbackHighestCombinedQuota"),
        }
    }

    #[test]
    fn test_recent_fleet_usage_in_last_4h_skips_primary_tier() {
        let config = FfResolutionConfig::new(20000);
        let candidates = vec![
            // Used 2000s ago (< 14400s)
            FfCandidateAccount::new("acc-recent", "recent@test.com")
                .with_weekly_quota(80.0)
                .with_rolling_4h_quota(100.0)
                .with_last_used_epoch(18000),
            // Used 15000s ago (> 14400s)
            FfCandidateAccount::new("acc-clean", "clean@test.com")
                .with_weekly_quota(80.0)
                .with_rolling_4h_quota(100.0)
                .with_last_used_epoch(5000),
        ];

        let result = resolve_fast_forward_target(&candidates, None, &config);
        match result {
            FfResolutionOutcome::FoundCandidate {
                account_id,
                selection_tier,
                ..
            } => {
                assert_eq!(account_id, "acc-clean");
                assert_eq!(
                    selection_tier,
                    FfSelectionTier::PrimaryFullHeadroomClosestExpiry
                );
            }
            _ => panic!("Expected FoundCandidate with acc-clean"),
        }
    }

    #[test]
    fn test_active_on_other_instance_excluded() {
        let config = FfResolutionConfig::default();
        let candidates = vec![
            FfCandidateAccount::new("acc-active", "active@test.com")
                .with_active_on_other_instance(true),
            FfCandidateAccount::new("acc-idle", "idle@test.com")
                .with_active_on_other_instance(false),
        ];

        let result = resolve_fast_forward_target(&candidates, None, &config);
        match result {
            FfResolutionOutcome::FoundCandidate { account_id, .. } => {
                assert_eq!(account_id, "acc-idle");
            }
            _ => panic!("Expected acc-idle"),
        }
    }

    #[test]
    fn test_primary_tier_selects_closest_to_expire() {
        let config = FfResolutionConfig::new(10000);
        let candidates = vec![
            FfCandidateAccount::new("acc-later", "later@test.com")
                .with_reset_epoch(15000)
                .with_weekly_quota(90.0)
                .with_rolling_4h_quota(100.0),
            FfCandidateAccount::new("acc-sooner", "sooner@test.com")
                .with_reset_epoch(11000)
                .with_weekly_quota(80.0)
                .with_rolling_4h_quota(100.0),
        ];

        let result = resolve_fast_forward_target(&candidates, None, &config);
        match result {
            FfResolutionOutcome::FoundCandidate {
                account_id,
                selection_tier,
                ..
            } => {
                assert_eq!(account_id, "acc-sooner");
                assert_eq!(
                    selection_tier,
                    FfSelectionTier::PrimaryFullHeadroomClosestExpiry
                );
            }
            _ => panic!("Expected acc-sooner"),
        }
    }

    #[test]
    fn test_degraded_fallback_selects_highest_combined_credits() {
        let config = FfResolutionConfig::default();
        let candidates = vec![
            // Free tier (multiplier 1.0): score = 1.0 * (80 + 50) = 130
            FfCandidateAccount::new("acc-free", "free@test.com")
                .with_tier_multiplier(1.0)
                .with_weekly_quota(80.0)
                .with_rolling_4h_quota(50.0),
            // Ultra tier (multiplier 4.0): score = 4.0 * (30 + 40) = 280
            FfCandidateAccount::new("acc-ultra", "ultra@test.com")
                .with_tier_multiplier(4.0)
                .with_weekly_quota(30.0)
                .with_rolling_4h_quota(40.0),
        ];

        let result = resolve_fast_forward_target(&candidates, None, &config);
        match result {
            FfResolutionOutcome::FoundCandidate {
                account_id,
                selection_tier,
                score,
                ..
            } => {
                assert_eq!(account_id, "acc-ultra");
                assert_eq!(
                    selection_tier,
                    FfSelectionTier::FallbackHighestCombinedQuota
                );
                assert!((score - 280.0).abs() < 1e-6);
            }
            _ => panic!("Expected acc-ultra"),
        }
    }

    #[test]
    fn test_configurable_weekly_threshold() {
        let config = FfResolutionConfig::default().with_min_weekly_quota_percent(25.0);
        let candidates = vec![FfCandidateAccount::new("acc-1", "user@test.com")
            .with_weekly_quota(20.0)
            .with_rolling_4h_quota(100.0)];

        let result = resolve_fast_forward_target(&candidates, None, &config);
        assert!(!result.is_found());
        match result {
            FfResolutionOutcome::NoEligibleCandidates { reason } => {
                assert!(reason.contains("25.0%"));
            }
            _ => panic!("Expected NoEligibleCandidates"),
        }
    }

    #[test]
    fn test_current_instance_account_excluded() {
        let config = FfResolutionConfig::default();
        let candidates = vec![
            FfCandidateAccount::new("acc-curr", "curr@test.com"),
            FfCandidateAccount::new("acc-other", "other@test.com"),
        ];

        let result = resolve_fast_forward_target(&candidates, Some("acc-curr"), &config);
        match result {
            FfResolutionOutcome::FoundCandidate { account_id, .. } => {
                assert_eq!(account_id, "acc-other");
            }
            _ => panic!("Expected acc-other"),
        }
    }
}
