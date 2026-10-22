//! Consecutive failure cooldown multiplier.
//! Applies progressive geometric backoff (1x, 2x, 4x, 8x, max 16x) to cooldown
//! durations when an account encounters repeated switch or rate-limit errors.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

pub const DEFAULT_BASE_COOLDOWN_SECS: u64 = 60;
pub const MAX_COOLDOWN_MULTIPLIER: u32 = 16;

#[derive(Debug, Clone)]
struct AccountFailureState {
    consecutive_failures: u32,
}

pub struct ConsecutiveFailureMultiplier {
    base_cooldown: Duration,
    max_multiplier: u32,
    states: Mutex<HashMap<String, AccountFailureState>>,
}

impl ConsecutiveFailureMultiplier {
    pub fn new(base_cooldown: Option<Duration>, max_multiplier: Option<u32>) -> Self {
        Self {
            base_cooldown: base_cooldown.unwrap_or(Duration::from_secs(DEFAULT_BASE_COOLDOWN_SECS)),
            max_multiplier: max_multiplier.unwrap_or(MAX_COOLDOWN_MULTIPLIER),
            states: Mutex::new(HashMap::new()),
        }
    }

    /// Records a failure for an account and returns the resulting cooldown duration.
    pub fn record_failure(&self, account_id: &str) -> Duration {
        let mut guard = match self.states.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };

        let entry = guard.entry(account_id.to_string()).or_insert(AccountFailureState {
            consecutive_failures: 0,
        });

        entry.consecutive_failures = entry.consecutive_failures.saturating_add(1);

        // Multiplier: 2^(failures - 1)
        let power = entry.consecutive_failures.saturating_sub(1).min(4);
        let multiplier = (1u32 << power).min(self.max_multiplier);

        self.base_cooldown * multiplier
    }

    /// Resets failure count upon successful switch or healthy health check.
    pub fn record_success(&self, account_id: &str) {
        let mut guard = match self.states.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        guard.remove(account_id);
    }

    /// Returns the current failure count for the given account.
    pub fn failure_count(&self, account_id: &str) -> u32 {
        let guard = match self.states.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        guard.get(account_id).map(|s| s.consecutive_failures).unwrap_or(0)
    }
}

impl Default for ConsecutiveFailureMultiplier {
    fn default() -> Self {
        Self::new(None, None)
    }
}
