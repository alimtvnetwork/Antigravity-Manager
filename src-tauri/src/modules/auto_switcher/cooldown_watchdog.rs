//! Account Cooldown Expiration Watchdog
//! Tracks registered cooldown timers and computes exact wake durations
//! so the auto-switcher can re-evaluate candidate availability the moment an account cools down.

use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Default)]
pub struct CooldownWatchdog {
    cooldowns: HashMap<String, Instant>,
}

impl CooldownWatchdog {
    pub fn new() -> Self {
        Self {
            cooldowns: HashMap::new(),
        }
    }

    /// Registers or updates a cooldown expiration timestamp for an account.
    pub fn register_cooldown(&mut self, account_id: impl Into<String>, duration: Duration) {
        self.cooldowns.insert(account_id.into(), Instant::now() + duration);
    }

    /// Computes duration until the earliest cooldown expires.
    /// Returns None if no accounts are currently cooling down.
    pub fn next_wake_duration(&self) -> Option<Duration> {
        let now = Instant::now();
        self.cooldowns
            .values()
            .filter(|&&expires_at| expires_at > now)
            .min()
            .map(|&earliest| earliest - now)
    }

    /// Clears expired entries and returns account IDs that have completed cooldown.
    pub fn sweep_completed(&mut self) -> Vec<String> {
        let now = Instant::now();
        let mut completed = Vec::new();

        self.cooldowns.retain(|id, &mut expires_at| {
            if expires_at <= now {
                completed.push(id.clone());
                false
            } else {
                true
            }
        });

        completed
    }
}
