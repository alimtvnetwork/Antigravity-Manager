// 5-Hour Rolling Quota Boundary Synchronizer (Imp-487)
// Calculates alignment with rolling 5-hour quota reset windows to avoid
// rotating away from an account that is about to replenish its quota.

use serde::{Deserialize, Serialize};

pub const DEFAULT_ROLLING_WINDOW_SECS: u64 = 5 * 3600; // 5 hours
pub const DEFAULT_IMPENDING_RECOVERY_SECS: u64 = 180; // 3 minutes

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaBoundaryStatus {
    pub current_epoch_secs: u64,
    pub next_reset_epoch_secs: u64,
    pub secs_until_reset: u64,
    pub is_impending_recovery: bool,
    pub should_withhold_rotation: bool,
}

#[derive(Debug, Clone)]
pub struct QuotaBoundarySynchronizer {
    window_secs: u64,
    impending_recovery_secs: u64,
}

impl Default for QuotaBoundarySynchronizer {
    fn default() -> Self {
        Self::new(DEFAULT_ROLLING_WINDOW_SECS, DEFAULT_IMPENDING_RECOVERY_SECS)
    }
}

impl QuotaBoundarySynchronizer {
    pub fn new(window_secs: u64, impending_recovery_secs: u64) -> Self {
        Self {
            window_secs: if window_secs == 0 {
                DEFAULT_ROLLING_WINDOW_SECS
            } else {
                window_secs
            },
            impending_recovery_secs,
        }
    }

    /// Evaluates whether an account with a given reset anchor or last activity epoch
    /// is close to its quota replenishment boundary.
    pub fn evaluate_boundary(
        &self,
        current_epoch_secs: u64,
        last_reset_epoch_secs: u64,
    ) -> QuotaBoundaryStatus {
        let elapsed = current_epoch_secs.saturating_sub(last_reset_epoch_secs);
        let remainder = elapsed % self.window_secs;
        let secs_until_reset = if remainder == 0 {
            self.window_secs
        } else {
            self.window_secs.saturating_sub(remainder)
        };

        let next_reset_epoch_secs = current_epoch_secs.saturating_add(secs_until_reset);
        let is_impending_recovery = secs_until_reset <= self.impending_recovery_secs;
        let should_withhold_rotation = is_impending_recovery;

        QuotaBoundaryStatus {
            current_epoch_secs,
            next_reset_epoch_secs,
            secs_until_reset,
            is_impending_recovery,
            should_withhold_rotation,
        }
    }

    /// Returns recommended backoff in seconds if rotation should be withheld.
    pub fn recommended_postpone_secs(&self, status: &QuotaBoundaryStatus) -> Option<u64> {
        if status.should_withhold_rotation {
            Some(status.secs_until_reset.saturating_add(5))
        } else {
            None
        }
    }
}
