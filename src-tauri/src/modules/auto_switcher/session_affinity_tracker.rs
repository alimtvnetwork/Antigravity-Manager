//! Account session affinity tracker.
//! Preserves conversation-to-account stickiness across multi-turn interactions
//! to minimize unnecessary context thrashing and preserve token cache affinity.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct AffinityEntry {
    account_id: String,
    last_turn_at: Instant,
    turn_count: u32,
}

pub struct SessionAffinityTracker {
    ttl: Duration,
    entries: Mutex<HashMap<String, AffinityEntry>>,
}

impl SessionAffinityTracker {
    pub fn new(ttl: Option<Duration>) -> Self {
        Self {
            ttl: ttl.unwrap_or(Duration::from_secs(600)), // 10 minutes default
            entries: Mutex::new(HashMap::new()),
        }
    }

    /// Records an active turn between a conversation and an account.
    pub fn record_turn(&self, conversation_id: impl Into<String>, account_id: impl Into<String>) {
        let mut guard = match self.entries.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };

        let conv_key = conversation_id.into();
        let acc_val = account_id.into();

        if let Some(entry) = guard.get_mut(&conv_key) {
            if entry.account_id == acc_val {
                entry.last_turn_at = Instant::now();
                entry.turn_count += 1;
                return;
            }
        }

        guard.insert(
            conv_key,
            AffinityEntry {
                account_id: acc_val,
                last_turn_at: Instant::now(),
                turn_count: 1,
            },
        );
    }

    /// Retrieves the preferred account ID for this conversation if affinity is active.
    pub fn get_preferred_account(&self, conversation_id: &str) -> Option<String> {
        let guard = match self.entries.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };

        if let Some(entry) = guard.get(conversation_id) {
            if entry.last_turn_at.elapsed() <= self.ttl {
                return Some(entry.account_id.clone());
            }
        }
        None
    }

    /// Prunes expired conversation affinity records.
    pub fn prune_stale(&self) {
        let mut guard = match self.entries.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        guard.retain(|_, entry| entry.last_turn_at.elapsed() <= self.ttl);
    }
}

impl Default for SessionAffinityTracker {
    fn default() -> Self {
        Self::new(None)
    }
}
