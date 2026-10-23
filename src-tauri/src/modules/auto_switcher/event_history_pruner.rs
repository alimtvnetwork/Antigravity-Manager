//! Auto-Switch Event History Pruner.
//! Enforces bounded capacity on switch event history in memory and persistent storage,
//! pruning records older than the retention window or exceeding maximum capacity (500 items).

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Single recorded auto-switch execution event.
#[derive(Debug, Clone)]
pub struct SwitchHistoryRecord {
    pub instance_id: String,
    pub previous_account_id: String,
    pub next_account_id: String,
    pub reason: String,
    pub score: f64,
    pub timestamp: Instant,
}

/// Bounded in-memory event history with capacity eviction.
#[derive(Debug)]
pub struct EventHistoryPruner {
    records: VecDeque<SwitchHistoryRecord>,
    max_capacity: usize,
    retention_window: Duration,
}

impl Default for EventHistoryPruner {
    fn default() -> Self {
        Self {
            records: VecDeque::with_capacity(500),
            max_capacity: 500,
            retention_window: Duration::from_secs(86400 * 7), // 7 days
        }
    }
}

impl EventHistoryPruner {
    /// Creates a pruner with custom capacity and retention limits.
    pub fn new(max_capacity: usize, retention_window: Duration) -> Self {
        Self {
            records: VecDeque::with_capacity(max_capacity),
            max_capacity,
            retention_window,
        }
    }

    /// Records a new switch event, automatically evicting oldest records if over capacity.
    pub fn record_event(&mut self, record: SwitchHistoryRecord) {
        if self.records.len() >= self.max_capacity {
            self.records.pop_front();
        }
        self.records.push_back(record);
    }

    /// Prunes all events older than the configured retention window.
    pub fn prune_expired(&mut self) -> usize {
        let now = Instant::now();
        let initial_len = self.records.len();
        let retention = self.retention_window;

        self.records
            .retain(|rec| now.duration_since(rec.timestamp) <= retention);

        initial_len.saturating_sub(self.records.len())
    }

    /// Returns a slice of the most recent events up to `limit`.
    pub fn recent_events(&self, limit: usize) -> Vec<SwitchHistoryRecord> {
        let take_count = limit.min(self.records.len());
        self.records
            .iter()
            .rev()
            .take(take_count)
            .cloned()
            .collect()
    }

    /// Total number of tracked switch events.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Checks if history is empty.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}
