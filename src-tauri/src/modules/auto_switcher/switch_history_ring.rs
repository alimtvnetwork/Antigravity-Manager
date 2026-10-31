//! Account Switch History Ring Buffer.
//! In-memory circular ring buffer recording recent switch events with millisecond timestamps and transition metadata.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Mutex;

pub const DEFAULT_SWITCH_RING_CAPACITY: usize = 50;

/// Snapshot of an account rotation event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchHistoryRecord {
    pub timestamp_ms: i64,
    pub instance_id: String,
    pub from_account_id: Option<String>,
    pub to_account_id: String,
    pub reason: String,
}

/// Circular ring buffer of switch history.
pub struct SwitchHistoryRing {
    capacity: usize,
    records: Mutex<VecDeque<SwitchHistoryRecord>>,
}

impl SwitchHistoryRing {
    pub fn new(capacity: Option<usize>) -> Self {
        let cap = capacity.unwrap_or(DEFAULT_SWITCH_RING_CAPACITY).max(5);
        Self {
            capacity: cap,
            records: Mutex::new(VecDeque::with_capacity(cap)),
        }
    }

    /// Records a new switch transition, evicting the oldest record if buffer is full.
    pub fn push(&self, record: SwitchHistoryRecord) {
        if let Ok(mut lock) = self.records.lock() {
            if lock.len() >= self.capacity {
                lock.pop_front();
            }
            lock.push_back(record);
        }
    }

    /// Returns a copy of recent switch history in reverse chronological order.
    pub fn get_recent(&self, limit: usize) -> Vec<SwitchHistoryRecord> {
        if let Ok(lock) = self.records.lock() {
            lock.iter().rev().take(limit).cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Returns total count of records currently retained.
    pub fn len(&self) -> usize {
        self.records.lock().map(|l| l.len()).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for SwitchHistoryRing {
    fn default() -> Self {
        Self::new(None)
    }
}
