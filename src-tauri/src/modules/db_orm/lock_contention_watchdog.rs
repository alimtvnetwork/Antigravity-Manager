//! Split-SQLite Connection Lock Contention Watchdog (Imp-548)
//! Tracks active database write lock acquisition durations across SQLite pools.
//! Identifies lingering locks exceeding safe thresholds and triggers diagnostic warnings
//! before SQLite `busy_timeout` escalates to SQLITE_BUSY deadlocks.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const DEFAULT_MAX_ACCEPTABLE_LOCK_MS: u64 = 3000;
pub const DEFAULT_CRITICAL_LOCK_TIMEOUT_MS: u64 = 8000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LockContentionStatus {
    Normal,
    Contended,
    CriticalTimeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveLockEntry {
    pub lock_id: String,
    pub db_name: String,
    pub acquired_at_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockContentionWatchdog {
    pub max_acceptable_ms: u64,
    pub critical_timeout_ms: u64,
    pub active_locks: HashMap<String, ActiveLockEntry>,
    pub peak_hold_ms: u64,
}

impl Default for LockContentionWatchdog {
    fn default() -> Self {
        Self::new(None, None)
    }
}

impl LockContentionWatchdog {
    pub fn new(max_acceptable_ms: Option<u64>, critical_timeout_ms: Option<u64>) -> Self {
        Self {
            max_acceptable_ms: max_acceptable_ms.unwrap_or(DEFAULT_MAX_ACCEPTABLE_LOCK_MS),
            critical_timeout_ms: critical_timeout_ms.unwrap_or(DEFAULT_CRITICAL_LOCK_TIMEOUT_MS),
            active_locks: HashMap::new(),
            peak_hold_ms: 0,
        }
    }

    /// Registers the acquisition of a database lock.
    pub fn acquire_lock(
        &mut self,
        lock_id: impl Into<String>,
        db_name: impl Into<String>,
        now_ms: u64,
    ) {
        let id = lock_id.into();
        self.active_locks.insert(
            id.clone(),
            ActiveLockEntry {
                lock_id: id,
                db_name: db_name.into(),
                acquired_at_ms: now_ms,
            },
        );
    }

    /// Releases an active lock and records its duration and peak hold time.
    pub fn release_lock(&mut self, lock_id: &str, now_ms: u64) -> Option<u64> {
        if let Some(entry) = self.active_locks.remove(lock_id) {
            let duration = now_ms.saturating_sub(entry.acquired_at_ms);
            if duration > self.peak_hold_ms {
                self.peak_hold_ms = duration;
            }
            Some(duration)
        } else {
            None
        }
    }

    /// Evaluates current lock table and returns highest contention status and offending lock IDs.
    pub fn evaluate_contention(&self, now_ms: u64) -> (LockContentionStatus, Vec<String>) {
        let mut status = LockContentionStatus::Normal;
        let mut offenders = Vec::new();

        for (id, entry) in &self.active_locks {
            let elapsed = now_ms.saturating_sub(entry.acquired_at_ms);
            if elapsed >= self.critical_timeout_ms {
                status = LockContentionStatus::CriticalTimeout;
                offenders.push(id.clone());
            } else if elapsed >= self.max_acceptable_ms {
                if status != LockContentionStatus::CriticalTimeout {
                    status = LockContentionStatus::Contended;
                }
                offenders.push(id.clone());
            }
        }

        (status, offenders)
    }

    /// Number of locks currently held.
    pub fn active_lock_count(&self) -> usize {
        self.active_locks.len()
    }

    /// Resets all tracked locks.
    pub fn reset(&mut self) {
        self.active_locks.clear();
        self.peak_hold_ms = 0;
    }
}
