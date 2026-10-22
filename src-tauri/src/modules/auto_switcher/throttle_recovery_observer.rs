//! Account Throttle Recovery Observer.
//! Tracks accounts subjected to HTTP 429 rate limit backoffs and applies smooth half-life
//! penalty decay, gradually reintroducing them as valid candidates rather than step-function flips.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Throttle state for an individual account.
#[derive(Debug, Clone)]
pub struct AccountThrottleState {
    pub throttled_at: Instant,
    pub base_penalty: f64,
    pub half_life: Duration,
    pub consecutive_throttles: u32,
}

/// Centralized observer tracking rate limits and computing decaying score penalties.
#[derive(Debug, Default)]
pub struct ThrottleRecoveryObserver {
    throttled_accounts: HashMap<String, AccountThrottleState>,
    default_half_life: Duration,
}

impl ThrottleRecoveryObserver {
    /// Creates a new observer with a default half-life duration (e.g. 180 seconds).
    pub fn new(default_half_life: Option<Duration>) -> Self {
        Self {
            throttled_accounts: HashMap::new(),
            default_half_life: default_half_life.unwrap_or(Duration::from_secs(180)),
        }
    }

    /// Records a throttle event (e.g. 429 response) for an account.
    pub fn record_throttle(&mut self, account_id: &str, penalty: Option<f64>) {
        let now = Instant::now();
        let penalty_val = penalty.unwrap_or(50.0);

        let entry = self
            .throttled_accounts
            .entry(account_id.to_string())
            .or_insert_with(|| AccountThrottleState {
                throttled_at: now,
                base_penalty: penalty_val,
                half_life: self.default_half_life,
                consecutive_throttles: 0,
            });

        entry.throttled_at = now;
        entry.consecutive_throttles += 1;
        // Escalate penalty proportionally to consecutive failures
        entry.base_penalty = (penalty_val * (1.5_f64).powi(entry.consecutive_throttles.min(4) as i32)).min(100.0);
    }

    /// Computes the current effective decaying penalty (0.0 to 100.0) for an account.
    pub fn current_penalty(&self, account_id: &str) -> f64 {
        let state = match self.throttled_accounts.get(account_id) {
            Some(s) => s,
            None => return 0.0,
        };

        let elapsed = Instant::now().duration_since(state.throttled_at);
        let half_life_secs = state.half_life.as_secs_f64().max(1.0);
        let elapsed_secs = elapsed.as_secs_f64();

        // Standard radioactive decay: P(t) = P0 * 2^(-t / T_half)
        let decay_factor = 2.0_f64.powf(-elapsed_secs / half_life_secs);
        let effective = state.base_penalty * decay_factor;

        if effective < 0.5 {
            0.0 // Fully recovered
        } else {
            effective
        }
    }

    /// Cleans up accounts that have completely recovered from penalties.
    pub fn sweep_recovered(&mut self) -> usize {
        let initial_count = self.throttled_accounts.len();
        self.throttled_accounts.retain(|_, state| {
            let elapsed = Instant::now().duration_since(state.throttled_at);
            let half_life_secs = state.half_life.as_secs_f64().max(1.0);
            let decay_factor = 2.0_f64.powf(-elapsed.as_secs_f64() / half_life_secs);
            (state.base_penalty * decay_factor) >= 0.5
        });
        initial_count.saturating_sub(self.throttled_accounts.len())
    }
}
