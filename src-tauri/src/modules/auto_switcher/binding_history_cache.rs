// LRU Account-Instance Binding History Cache (Imp-489)
// Tracks historical instance-to-account bindings to maintain profile affinity
// and identify patterns of oscillation between the same accounts.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub const DEFAULT_MAX_HISTORY_ENTRIES: usize = 128;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindingRecord {
    pub instance_id: String,
    pub account_id: String,
    pub timestamp_epoch_secs: u64,
    pub reason_tag: String,
}

#[derive(Debug, Clone)]
pub struct BindingHistoryCache {
    max_capacity: usize,
    records: Arc<Mutex<VecDeque<BindingRecord>>>,
}

impl Default for BindingHistoryCache {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_HISTORY_ENTRIES)
    }
}

impl BindingHistoryCache {
    pub fn new(max_capacity: usize) -> Self {
        Self {
            max_capacity: if max_capacity == 0 { DEFAULT_MAX_HISTORY_ENTRIES } else { max_capacity },
            records: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Records a new binding event, evicting the oldest entry when capacity is exceeded.
    pub fn record_binding(&self, instance_id: &str, account_id: &str, timestamp: u64, reason: &str) {
        if let Ok(mut lock) = self.records.lock() {
            if lock.len() >= self.max_capacity {
                lock.pop_front();
            }
            lock.push_back(BindingRecord {
                instance_id: instance_id.to_string(),
                account_id: account_id.to_string(),
                timestamp_epoch_secs: timestamp,
                reason_tag: reason.to_string(),
            });
        }
    }

    /// Finds the most recent binding for a given instance ID.
    pub fn get_last_binding_for_instance(&self, instance_id: &str) -> Option<BindingRecord> {
        let lock = self.records.lock().ok()?;
        lock.iter().rev().find(|r| r.instance_id == instance_id).cloned()
    }

    /// Counts how many times an account was bound across all instances recently.
    pub fn count_recent_bindings_for_account(&self, account_id: &str, lookback_entries: usize) -> usize {
        let lock = match self.records.lock() {
            Ok(l) => l,
            Err(_) => return 0,
        };
        lock.iter()
            .rev()
            .take(lookback_entries)
            .filter(|r| r.account_id == account_id)
            .count()
    }
}
