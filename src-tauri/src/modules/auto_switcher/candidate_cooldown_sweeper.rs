//! Candidate rotation cooldown sweeper.
//! Inspects active cooldown entries, purges expired rate limits, and returns
//! accounts that are newly unblocked and eligible for rotation scoring.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CooldownSweepResult {
    pub total_tracked: usize,
    pub unblocked_count: usize,
    pub unblocked_account_ids: Vec<String>,
}

#[derive(Debug)]
struct CooldownEntry {
    account_id: String,
    expires_at: Instant,
}

pub struct CandidateCooldownSweeper {
    cooldowns: Mutex<HashMap<String, CooldownEntry>>,
}

impl CandidateCooldownSweeper {
    pub fn new() -> Self {
        Self {
            cooldowns: Mutex::new(HashMap::new()),
        }
    }

    /// Puts an account into cooldown until the specified expiration instant.
    pub fn set_cooldown(&self, account_id: impl Into<String>, expires_at: Instant) {
        let mut guard = match self.cooldowns.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        let id = account_id.into();
        guard.insert(
            id.clone(),
            CooldownEntry {
                account_id: id,
                expires_at,
            },
        );
    }

    /// Sweeps expired entries and returns accounts that are now unblocked.
    pub fn sweep_expired(&self) -> CooldownSweepResult {
        let mut guard = match self.cooldowns.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };

        let now = Instant::now();
        let total = guard.len();
        let mut unblocked = Vec::new();

        guard.retain(|id, entry| {
            if now >= entry.expires_at {
                unblocked.push(id.clone());
                false // Evict
            } else {
                true // Keep
            }
        });

        let count = unblocked.len();
        CooldownSweepResult {
            total_tracked: total,
            unblocked_count: count,
            unblocked_account_ids: unblocked,
        }
    }

    /// Checks if a specific account is currently in cooldown.
    pub fn is_in_cooldown(&self, account_id: &str) -> bool {
        let guard = match self.cooldowns.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };

        if let Some(entry) = guard.get(account_id) {
            Instant::now() < entry.expires_at
        } else {
            false
        }
    }
}

impl Default for CandidateCooldownSweeper {
    fn default() -> Self {
        Self::new()
    }
}
