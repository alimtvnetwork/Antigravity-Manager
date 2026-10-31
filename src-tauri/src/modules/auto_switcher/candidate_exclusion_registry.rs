//! Candidate Exclusion Registry & Manual Bypass Cache
//!
//! Provides a thread-safe registry tracking temporarily excluded accounts (manual bypass,
//! user disabled, authentication failure, quota lockouts) with TTL expirations
//! and query filters to prevent excluded accounts from entering rotation scoring.

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};
use tracing::debug;

/// Reason code for account exclusion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExclusionReason {
    ManualBypass,
    AuthFailure,
    QuotaLocked,
    UserDisabled,
    Custom(String),
}

/// Entry stored in the exclusion registry.
#[derive(Debug, Clone)]
pub struct ExclusionEntry {
    pub account_id: String,
    pub reason: ExclusionReason,
    pub added_at: Instant,
    pub ttl: Option<Duration>,
}

impl ExclusionEntry {
    pub fn is_expired(&self, now: Instant) -> bool {
        match self.ttl {
            Some(duration) => now.duration_since(self.added_at) > duration,
            None => false, // Permanent exclusion until manually removed
        }
    }
}

/// Thread-safe in-memory candidate exclusion registry.
pub struct CandidateExclusionRegistry {
    entries: RwLock<HashMap<String, ExclusionEntry>>,
}

impl Default for CandidateExclusionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CandidateExclusionRegistry {
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
        }
    }

    /// Excludes an account with a reason and optional TTL.
    pub fn exclude(&self, account_id: String, reason: ExclusionReason, ttl: Option<Duration>) {
        let entry = ExclusionEntry {
            account_id: account_id.clone(),
            reason,
            added_at: Instant::now(),
            ttl,
        };

        if let Ok(mut map) = self.entries.write() {
            map.insert(account_id, entry);
        }
    }

    /// Removes an account from exclusion manually.
    pub fn clear_exclusion(&self, account_id: &str) -> bool {
        if let Ok(mut map) = self.entries.write() {
            map.remove(account_id).is_some()
        } else {
            false
        }
    }

    /// Checks if an account is currently excluded (ignoring expired entries).
    pub fn is_excluded(&self, account_id: &str) -> bool {
        let now = Instant::now();
        if let Ok(map) = self.entries.read() {
            if let Some(entry) = map.get(account_id) {
                !entry.is_expired(now)
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Sweeps all expired entries from memory.
    pub fn sweep_expired(&self) -> usize {
        let now = Instant::now();
        let mut swept = 0;
        if let Ok(mut map) = self.entries.write() {
            let before = map.len();
            map.retain(|_, entry| !entry.is_expired(now));
            swept = before - map.len();
        }
        if swept > 0 {
            debug!(swept = swept, "Swept expired candidate exclusions");
        }
        swept
    }
}
