//! Transaction commit latency observer.
//! Measures elapsed wall-clock time during SQLite transaction `COMMIT` operations
//! to identify fsync bottlenecks and WAL disk write contention.

use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitLatencyStats {
    pub total_commits_tracked: usize,
    pub avg_commit_duration_ms: f64,
    pub max_commit_duration_ms: f64,
    pub slow_commits_gt_50ms: usize,
}

pub struct CommitLatencyObserver {
    slow_threshold: Duration,
    total_count: Mutex<usize>,
    total_duration_ms: Mutex<f64>,
    max_duration_ms: Mutex<f64>,
    slow_count: Mutex<usize>,
}

impl CommitLatencyObserver {
    pub fn new(slow_threshold: Option<Duration>) -> Self {
        Self {
            slow_threshold: slow_threshold.unwrap_or(Duration::from_millis(50)),
            total_count: Mutex::new(0),
            total_duration_ms: Mutex::new(0.0),
            max_duration_ms: Mutex::new(0.0),
            slow_count: Mutex::new(0),
        }
    }

    /// Records the execution duration of a transaction commit.
    pub fn record_commit_duration(&self, duration: Duration) {
        let ms = duration.as_secs_f64() * 1000.0;

        if let Ok(mut count) = self.total_count.lock() {
            *count += 1;
        }

        if let Ok(mut total) = self.total_duration_ms.lock() {
            *total += ms;
        }

        if let Ok(mut max) = self.max_duration_ms.lock() {
            if ms > *max {
                *max = ms;
            }
        }

        if duration > self.slow_threshold {
            if let Ok(mut slow) = self.slow_count.lock() {
                *slow += 1;
            }
        }
    }

    /// Returns aggregated commit performance metrics.
    pub fn get_stats(&self) -> CommitLatencyStats {
        let count = self.total_count.lock().map(|g| *g).unwrap_or(0);
        let total_ms = self.total_duration_ms.lock().map(|g| *g).unwrap_or(0.0);
        let max_ms = self.max_duration_ms.lock().map(|g| *g).unwrap_or(0.0);
        let slow = self.slow_count.lock().map(|g| *g).unwrap_or(0);

        let avg = if count > 0 {
            total_ms / (count as f64)
        } else {
            0.0
        };

        CommitLatencyStats {
            total_commits_tracked: count,
            avg_commit_duration_ms: avg,
            max_commit_duration_ms: max_ms,
            slow_commits_gt_50ms: slow,
        }
    }
}

impl Default for CommitLatencyObserver {
    fn default() -> Self {
        Self::new(None)
    }
}
