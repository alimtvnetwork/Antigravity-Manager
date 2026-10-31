//! SQLite Lock Contention & Busy Wait Profiler
//!
//! Tracks and profiles lock contention events, busy handler invocations, and acquisition
//! latencies across split SQLite databases, providing early warnings for database bottlenecking.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;
use tracing::warn;

/// Lock contention metrics for a specific database.
#[derive(Debug, Clone, Default)]
pub struct DatabaseLockMetrics {
    pub busy_count: u64,
    pub locked_count: u64,
    pub total_wait_micros: u64,
    pub max_wait_micros: u64,
}

/// Global or module-scoped profiler recording SQLite lock contention statistics.
#[derive(Default)]
pub struct LockContentionProfiler {
    db_metrics: Mutex<HashMap<String, DatabaseLockMetrics>>,
}

impl LockContentionProfiler {
    pub fn new() -> Self {
        Self {
            db_metrics: Mutex::new(HashMap::new()),
        }
    }

    /// Records a lock contention or busy-wait event.
    pub fn record_contention(&self, db_name: &str, wait_time: Duration, is_busy: bool) {
        let wait_micros = wait_time.as_micros() as u64;

        let mut map = match self.db_metrics.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };

        let metrics = map.entry(db_name.to_string()).or_default();
        if is_busy {
            metrics.busy_count += 1;
        } else {
            metrics.locked_count += 1;
        }

        metrics.total_wait_micros += wait_micros;
        if wait_micros > metrics.max_wait_micros {
            metrics.max_wait_micros = wait_micros;
        }

        if wait_time > Duration::from_millis(100) {
            warn!(
                db = db_name,
                wait_ms = wait_time.as_millis(),
                is_busy = is_busy,
                "Significant SQLite lock contention detected"
            );
        }
    }

    /// Retrieves a cloned snapshot of current metrics per database.
    pub fn snapshot(&self) -> HashMap<String, DatabaseLockMetrics> {
        let map = match self.db_metrics.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        map.clone()
    }

    /// Resets all metrics.
    pub fn reset(&self) {
        let mut map = match self.db_metrics.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        map.clear();
    }
}
