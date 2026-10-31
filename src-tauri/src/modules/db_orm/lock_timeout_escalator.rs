//! Split-DB Lock Contention Timeout Escalator (`Imp-629`).
//!
//! Under high concurrency across split SQLite databases (`active_prompts`,
//! `repo_prompts.db`), dynamic timeout escalation prevents deadlock and busy storms.
//! Implements AGENTS.md requirements:
//! - "Bound every wait with a timeout"
//! - "No Silent Errors: every fallible operation must route its failure into the error module"

use crate::error::AppError;
use serde::{Deserialize, Serialize};

/// Determines whether an SQLite error code represents a busy or locked contention state.
///
/// Recognizes primary SQLITE_BUSY (5) and SQLITE_LOCKED (6), as well as extended error codes
/// sharing these base codes (e.g. SQLITE_BUSY_RECOVERY = 261, SQLITE_BUSY_SNAPSHOT = 517).
pub fn is_busy_error_code(sqlite_code: i32) -> bool {
    let primary = sqlite_code & 0xFF;
    primary == 5 || primary == 6
}

/// Policy defining backoff growth, timeouts, retry limits, and jitter bounds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LockEscalationPolicy {
    /// Base initial backoff timeout in milliseconds (default: 250ms).
    pub base_timeout_ms: u64,
    /// Maximum backoff ceiling in milliseconds (default: 5,000ms).
    pub max_timeout_ms: u64,
    /// Maximum allowed busy attempts before declaring deadlock (default: 5).
    pub max_attempts: u32,
    /// Multiplier applied exponentially per attempt (default: 2.0).
    pub backoff_multiplier: f64,
    /// Fractional jitter ratio in range [0.0, 1.0] (default: 0.25).
    pub jitter_ratio: f64,
}

impl Default for LockEscalationPolicy {
    fn default() -> Self {
        Self {
            base_timeout_ms: 250,
            max_timeout_ms: 5_000,
            max_attempts: 5,
            backoff_multiplier: 2.0,
            jitter_ratio: 0.25,
        }
    }
}

impl LockEscalationPolicy {
    /// Creates a custom lock escalation policy with sanitized bounds.
    pub fn new(
        base_timeout_ms: u64,
        max_timeout_ms: u64,
        max_attempts: u32,
        backoff_multiplier: f64,
        jitter_ratio: f64,
    ) -> Self {
        Self {
            base_timeout_ms: base_timeout_ms.max(1),
            max_timeout_ms: max_timeout_ms.max(base_timeout_ms),
            max_attempts,
            backoff_multiplier: if backoff_multiplier > 0.0 {
                backoff_multiplier
            } else {
                1.0
            },
            jitter_ratio: jitter_ratio.clamp(0.0, 1.0),
        }
    }

    /// Computes the unjittered nominal timeout for a zero-based attempt index.
    pub fn calculate_nominal_timeout(&self, attempt_index: u32) -> u64 {
        let mult = self.backoff_multiplier.powi(attempt_index as i32);
        let raw = (self.base_timeout_ms as f64 * mult).round() as u64;
        raw.min(self.max_timeout_ms)
    }

    /// Computes the structured escalation step descriptor for an attempt index.
    pub fn calculate_step(&self, attempt_index: u32) -> LockEscalationStep {
        let calculated_timeout_ms = self.calculate_nominal_timeout(attempt_index);
        let is_terminal_attempt = attempt_index + 1 >= self.max_attempts;

        LockEscalationStep {
            attempt_index,
            calculated_timeout_ms,
            is_terminal_attempt,
        }
    }
}

/// Describes the parameters and terminal status of an escalation step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockEscalationStep {
    /// Zero-based attempt index (0, 1, 2, ...).
    pub attempt_index: u32,
    /// Calculated timeout in milliseconds for this step.
    pub calculated_timeout_ms: u64,
    /// Whether this is the final permitted attempt before exhaustion.
    pub is_terminal_attempt: bool,
}

/// Decision outcome emitted by `LockTimeoutEscalator::next_timeout`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LockEscalationDecision {
    /// Instructs caller to sleep or apply PRAGMA busy_timeout before retrying.
    RetryWithTimeout {
        /// Recommended timeout in milliseconds.
        timeout_ms: u64,
        /// Current 1-indexed attempt number.
        attempt: u32,
    },
    /// Informs caller that max attempts were exhausted, indicating deadlock.
    LockExhaustedDeadlock {
        /// Total cumulative milliseconds spent waiting across all attempts.
        total_elapsed_ms: u64,
        /// Number of busy retry attempts executed.
        attempts_made: u32,
        /// Human-readable descriptive error message.
        error_message: String,
    },
}

impl LockEscalationDecision {
    /// Returns true if the decision permits a subsequent retry.
    pub fn is_retry(&self) -> bool {
        matches!(self, Self::RetryWithTimeout { .. })
    }

    /// Returns true if retry attempts have been completely exhausted.
    pub fn is_exhausted(&self) -> bool {
        matches!(self, Self::LockExhaustedDeadlock { .. })
    }

    /// Converts an exhausted deadlock decision into an AppError.
    pub fn to_app_error(&self) -> Option<AppError> {
        match self {
            Self::RetryWithTimeout { .. } => None,
            Self::LockExhaustedDeadlock { error_message, .. } => {
                Some(AppError::Config(error_message.clone()))
            }
        }
    }
}

/// Escalator state machine managing exponential backoff and jitter for SQLite lock contention.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LockTimeoutEscalator {
    /// Escalation policy controlling backoff progression.
    pub policy: LockEscalationPolicy,
    /// Number of busy retry attempts executed so far.
    pub attempt_count: u32,
    /// Cumulative milliseconds waited across all attempts.
    pub total_elapsed_ms: u64,
    /// Whether the escalator has reached its maximum attempts limit.
    pub is_exhausted: bool,
    /// Pseudo-random LCG state for deterministic jitter generation.
    rng_seed: u64,
}

impl Default for LockTimeoutEscalator {
    fn default() -> Self {
        Self::new(LockEscalationPolicy::default())
    }
}

impl LockTimeoutEscalator {
    /// Constructs a new escalator with default entropy.
    pub fn new(policy: LockEscalationPolicy) -> Self {
        Self::with_seed(policy, 0xa5a5_5a5a_3c3c_c3c3)
    }

    /// Constructs a new escalator with an explicit pseudo-random seed.
    pub fn with_seed(policy: LockEscalationPolicy, seed: u64) -> Self {
        Self {
            policy,
            attempt_count: 0,
            total_elapsed_ms: 0,
            is_exhausted: false,
            rng_seed: seed,
        }
    }

    /// Resets all internal attempt counters and exhaustion flags.
    pub fn reset(&mut self) {
        self.attempt_count = 0;
        self.total_elapsed_ms = 0;
        self.is_exhausted = false;
    }

    /// Computes the next deterministic LCG float in range [-1.0, 1.0].
    fn next_lcg_factor(&mut self) -> f64 {
        self.rng_seed = self
            .rng_seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let high = (self.rng_seed >> 32) as u32;
        ((high as f64) / (u32::MAX as f64)) * 2.0 - 1.0
    }

    /// Calculates a single escalation step descriptor for an arbitrary attempt index.
    pub fn calculate_step(&self, attempt_index: u32) -> LockEscalationStep {
        self.policy.calculate_step(attempt_index)
    }

    /// Evaluates lock contention status and returns the next escalation decision.
    pub fn next_timeout(&mut self, is_busy: bool) -> LockEscalationDecision {
        if !is_busy {
            self.reset();
            return LockEscalationDecision::RetryWithTimeout {
                timeout_ms: self.policy.base_timeout_ms,
                attempt: 0,
            };
        }

        if self.attempt_count >= self.policy.max_attempts {
            self.is_exhausted = true;
            let error_message = format!(
                "SQLite lock contention exhausted after {} attempts (total wait: {}ms)",
                self.attempt_count, self.total_elapsed_ms
            );
            return LockEscalationDecision::LockExhaustedDeadlock {
                total_elapsed_ms: self.total_elapsed_ms,
                attempts_made: self.attempt_count,
                error_message,
            };
        }

        let nominal_timeout = self.policy.calculate_nominal_timeout(self.attempt_count);
        let jittered_timeout = self.apply_jitter(nominal_timeout);

        self.attempt_count += 1;
        self.total_elapsed_ms = self.total_elapsed_ms.saturating_add(jittered_timeout);

        LockEscalationDecision::RetryWithTimeout {
            timeout_ms: jittered_timeout,
            attempt: self.attempt_count,
        }
    }

    /// Applies bounded pseudo-random jitter within [-jitter_ratio, +jitter_ratio].
    fn apply_jitter(&mut self, nominal: u64) -> u64 {
        let ratio = self.policy.jitter_ratio.clamp(0.0, 1.0);
        if ratio == 0.0 {
            return nominal.min(self.policy.max_timeout_ms);
        }

        let factor = self.next_lcg_factor();
        let jitter_offset = (nominal as f64) * ratio * factor;
        let candidate = (nominal as f64) + jitter_offset;
        let clamped = candidate.round().max(0.0) as u64;

        clamped.min(self.policy.max_timeout_ms)
    }
}
