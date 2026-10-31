//! Rapid Account Bounce Dampener.
//! Suppresses rapid oscillation (bounce flip-flop) between two accounts having
//! near-identical scores within an aggressive cooldown dampening window (<60s).

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Dampening configuration parameters.
#[derive(Debug, Clone, Copy)]
pub struct BounceDampenerConfig {
    pub min_residence_time: Duration,
    pub bounce_penalty: f64,
}

impl Default for BounceDampenerConfig {
    fn default() -> Self {
        Self {
            min_residence_time: Duration::from_secs(60),
            bounce_penalty: 30.0,
        }
    }
}

/// Tracks recently abandoned accounts per instance to penalize immediate bounce-backs.
#[derive(Debug, Default)]
pub struct RapidBounceDampener {
    // (instance_id, account_id) -> time_abandoned
    abandoned_accounts: HashMap<(String, String), Instant>,
    config: BounceDampenerConfig,
}

impl RapidBounceDampener {
    /// Creates a new dampener with default or custom config.
    pub fn new(config: Option<BounceDampenerConfig>) -> Self {
        Self {
            abandoned_accounts: HashMap::new(),
            config: config.unwrap_or_default(),
        }
    }

    /// Records that an instance just switched away from an account.
    pub fn record_abandonment(&mut self, instance_id: &str, account_id: &str) {
        self.abandoned_accounts.insert(
            (instance_id.to_string(), account_id.to_string()),
            Instant::now(),
        );
    }

    /// Evaluates if switching back to an account incurs a bounce penalty.
    pub fn evaluate_bounce_penalty(&self, instance_id: &str, candidate_account_id: &str) -> f64 {
        let key = (instance_id.to_string(), candidate_account_id.to_string());
        if let Some(abandoned_at) = self.abandoned_accounts.get(&key) {
            let elapsed = abandoned_at.elapsed();
            if elapsed < self.config.min_residence_time {
                // Linear decay of the penalty over the residence window
                let progress = elapsed.as_secs_f64() / self.config.min_residence_time.as_secs_f64();
                return (1.0 - progress) * self.config.bounce_penalty;
            }
        }
        0.0
    }

    /// Cleans up tracking entries that have passed the minimum residence window.
    pub fn sweep_expired(&mut self) -> usize {
        let initial_len = self.abandoned_accounts.len();
        let timeout = self.config.min_residence_time;
        self.abandoned_accounts
            .retain(|_, time| time.elapsed() < timeout);
        initial_len.saturating_sub(self.abandoned_accounts.len())
    }
}
