//! Auto-Switcher Candidate Dynamic Scorer (`Imp-619`)
//!
//! Evaluates candidate accounts for auto-switching by dynamically combining
//! quota headroom percentages, token burn rate velocities, consecutive throttle
//! counts, and cooldown timers into a unified composite score.
//! Prevents collision across sibling instances and prioritizes accounts with
//! durable runway before 429 rate limit lockouts.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Maximum possible composite score for an ideal candidate.
pub const MAX_COMPOSITE_SCORE: f64 = 1000.0;

/// Scoring factors describing a candidate account state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CandidateScoringFactors {
    pub account_id: String,
    pub email: String,
    /// Remaining 4-hour quota percentage (0.0 to 100.0).
    pub remaining_quota_percent: f64,
    /// Measured consumption velocity in tokens per second.
    pub burn_rate_tokens_per_sec: f64,
    /// Projected seconds until quota exhaustion, if active.
    pub estimated_seconds_remaining: Option<u64>,
    /// Number of consecutive 429 or transient throttle errors observed.
    pub consecutive_throttles: u32,
    /// Cooldown seconds remaining before account can be safely used.
    pub cooldown_remaining_secs: u64,
    /// Whether the account is currently enabled and healthy.
    pub is_active: bool,
    /// Whether the account is currently leased by another active instance.
    pub is_in_use_by_other_instance: bool,
}

impl CandidateScoringFactors {
    /// Creates a new CandidateScoringFactors instance with full quota defaults.
    pub fn new(account_id: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            account_id: account_id.into(),
            email: email.into(),
            remaining_quota_percent: 100.0,
            burn_rate_tokens_per_sec: 0.0,
            estimated_seconds_remaining: None,
            consecutive_throttles: 0,
            cooldown_remaining_secs: 0,
            is_active: true,
            is_in_use_by_other_instance: false,
        }
    }

    /// Sets the remaining quota percentage (clamped between 0.0 and 100.0).
    pub fn with_remaining_quota_percent(mut self, percent: f64) -> Self {
        self.remaining_quota_percent = if percent.is_nan() {
            0.0
        } else {
            percent.clamp(0.0, 100.0)
        };
        self
    }

    /// Sets the measured burn rate velocity in tokens per second.
    pub fn with_burn_rate(mut self, burn_rate: f64) -> Self {
        self.burn_rate_tokens_per_sec = if burn_rate.is_nan() || burn_rate < 0.0 {
            0.0
        } else {
            burn_rate
        };
        self
    }

    /// Sets the estimated seconds remaining until quota exhaustion.
    pub fn with_estimated_seconds_remaining(mut self, secs: Option<u64>) -> Self {
        self.estimated_seconds_remaining = secs;
        self
    }

    /// Sets consecutive throttles observed.
    pub fn with_consecutive_throttles(mut self, throttles: u32) -> Self {
        self.consecutive_throttles = throttles;
        self
    }

    /// Sets remaining active cooldown in seconds.
    pub fn with_cooldown_remaining(mut self, cooldown_secs: u64) -> Self {
        self.cooldown_remaining_secs = cooldown_secs;
        self
    }

    /// Sets active/enabled state of the candidate.
    pub fn with_active(mut self, is_active: bool) -> Self {
        self.is_active = is_active;
        self
    }

    /// Sets whether candidate is currently leased by another instance.
    pub fn with_in_use_by_other_instance(mut self, in_use: bool) -> Self {
        self.is_in_use_by_other_instance = in_use;
        self
    }
}

/// Scoring violation flags identified during dynamic evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScoringViolation {
    /// Account is disabled or flagged inactive.
    AccountInactiveOrDisabled,
    /// Account is currently bound or leased by another active instance.
    InUseByOtherInstance,
    /// Candidate is actively within a cooldown recovery window.
    ActiveCooldown,
    /// Token quota balance is completely exhausted (0% remaining).
    QuotaExhausted,
    /// Projected exhaustion runway is critically short (<= 120s).
    CriticalVelocityExhaustion,
    /// Consecutive throttles exceeded safe threshold (>= 3).
    ExcessiveThrottling,
}

impl ScoringViolation {
    /// Returns true if this violation entirely disqualifies the candidate from selection.
    pub fn is_disqualifying(&self) -> bool {
        matches!(
            self,
            Self::AccountInactiveOrDisabled
                | Self::InUseByOtherInstance
                | Self::ActiveCooldown
                | Self::QuotaExhausted
        )
    }

    /// Descriptive summary of the violation.
    pub fn description(&self) -> &'static str {
        match self {
            Self::AccountInactiveOrDisabled => "Account is inactive or disabled",
            Self::InUseByOtherInstance => "Account is in use by sibling instance",
            Self::ActiveCooldown => "Account is in active cooldown",
            Self::QuotaExhausted => "Remaining quota completely exhausted (0%)",
            Self::CriticalVelocityExhaustion => "Projected exhaustion runway is under 120 seconds",
            Self::ExcessiveThrottling => "Excessive consecutive rate-limit throttles observed",
        }
    }
}

impl fmt::Display for ScoringViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// Evaluated candidate result with breakdown score metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoredCandidate {
    pub account_id: String,
    pub email: String,
    pub final_score: f64,
    pub base_score: f64,
    pub velocity_penalty: f64,
    pub throttle_penalty: f64,
    pub is_eligible: bool,
    pub exclusion_reason: Option<String>,
    pub violations: Vec<ScoringViolation>,
}

impl ScoredCandidate {
    /// Returns whether this candidate has any non-disqualifying or disqualifying violations.
    pub fn has_violations(&self) -> bool {
        !self.violations.is_empty()
    }

    /// Returns whether this candidate is immediately selectable (eligible and score > 0).
    pub fn is_selectable(&self) -> bool {
        self.is_eligible && self.final_score > 0.0
    }
}

/// Evaluates dynamic score for a single candidate account.
pub fn score_candidate_dynamically(factors: &CandidateScoringFactors) -> ScoredCandidate {
    // 1. Hard disqualification criteria
    if !factors.is_active {
        return ScoredCandidate {
            account_id: factors.account_id.clone(),
            email: factors.email.clone(),
            final_score: 0.0,
            base_score: 0.0,
            velocity_penalty: 0.0,
            throttle_penalty: 0.0,
            is_eligible: false,
            exclusion_reason: Some("Account is inactive or disabled".to_string()),
            violations: vec![ScoringViolation::AccountInactiveOrDisabled],
        };
    }

    if factors.is_in_use_by_other_instance {
        return ScoredCandidate {
            account_id: factors.account_id.clone(),
            email: factors.email.clone(),
            final_score: 0.0,
            base_score: 0.0,
            velocity_penalty: 0.0,
            throttle_penalty: 0.0,
            is_eligible: false,
            exclusion_reason: Some("Account is in use by sibling instance".to_string()),
            violations: vec![ScoringViolation::InUseByOtherInstance],
        };
    }

    if factors.cooldown_remaining_secs > 0 {
        return ScoredCandidate {
            account_id: factors.account_id.clone(),
            email: factors.email.clone(),
            final_score: 0.0,
            base_score: 0.0,
            velocity_penalty: 0.0,
            throttle_penalty: 0.0,
            is_eligible: false,
            exclusion_reason: Some(format!(
                "Account in cooldown ({}s remaining)",
                factors.cooldown_remaining_secs
            )),
            violations: vec![ScoringViolation::ActiveCooldown],
        };
    }

    if factors.remaining_quota_percent <= 0.0 {
        return ScoredCandidate {
            account_id: factors.account_id.clone(),
            email: factors.email.clone(),
            final_score: 0.0,
            base_score: 0.0,
            velocity_penalty: 0.0,
            throttle_penalty: 0.0,
            is_eligible: false,
            exclusion_reason: Some("Quota completely exhausted (0%)".to_string()),
            violations: vec![ScoringViolation::QuotaExhausted],
        };
    }

    // 2. Base score derived from remaining quota percentage (scaled up to 1000.0)
    let base_score = factors.remaining_quota_percent.clamp(0.0, 100.0) * 10.0;

    let mut violations = Vec::new();

    // 3. Velocity penalty: accounts running out of quota quickly get penalized
    let velocity_penalty = match factors.estimated_seconds_remaining {
        Some(secs) if secs <= 120 => {
            violations.push(ScoringViolation::CriticalVelocityExhaustion);
            400.0
        }
        Some(secs) if secs <= 300 => 200.0,
        Some(secs) if secs <= 600 => 50.0,
        _ => 0.0,
    };

    // 4. Consecutive throttle penalty: each recent 429 deducts 150 points
    let throttle_penalty = (factors.consecutive_throttles as f64 * 150.0).min(500.0);
    if factors.consecutive_throttles >= 3 {
        violations.push(ScoringViolation::ExcessiveThrottling);
    }

    // 5. Compute net composite score
    let raw_score = base_score - velocity_penalty - throttle_penalty;
    let final_score = raw_score.max(1.0); // Minimum 1.0 score if eligible

    ScoredCandidate {
        account_id: factors.account_id.clone(),
        email: factors.email.clone(),
        final_score,
        base_score,
        velocity_penalty,
        throttle_penalty,
        is_eligible: true,
        exclusion_reason: None,
        violations,
    }
}

/// Ranks a slice of candidate accounts, returning candidates sorted by final_score descending.
pub fn rank_candidates(candidates: &[CandidateScoringFactors]) -> Vec<ScoredCandidate> {
    let mut scored: Vec<ScoredCandidate> =
        candidates.iter().map(score_candidate_dynamically).collect();

    scored.sort_by(|a, b| {
        // First sort by eligibility (eligible first)
        b.is_eligible
            .cmp(&a.is_eligible)
            // Then sort by final_score descending
            .then_with(|| {
                b.final_score
                    .partial_cmp(&a.final_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            // Tie-breaker by account_id ascending
            .then_with(|| a.account_id.cmp(&b.account_id))
    });

    scored
}

/// Selects the single best candidate from a candidate list, or None if no candidate is selectable.
pub fn select_best_candidate(candidates: &[CandidateScoringFactors]) -> Option<ScoredCandidate> {
    rank_candidates(candidates)
        .into_iter()
        .find(|c| c.is_selectable())
}
