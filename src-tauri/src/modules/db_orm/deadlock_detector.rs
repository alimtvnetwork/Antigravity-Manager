//! SQLite Concurrency Deadlock Detector.
//! Monitors database busy handler retries and lock wait durations across split databases,
//! alerting when contentious write locks approach the 5,000ms busy timeout ceiling.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Threshold metrics for lock contention monitoring.
#[derive(Debug, Clone, Copy)]
pub struct DeadlockDetectorConfig {
    /// Lock duration that triggers an immediate warning.
    pub lock_contention_warning_threshold: Duration,
    /// Number of busy retry attempts considered dangerous.
    pub busy_retry_danger_threshold: u32,
}

impl Default for DeadlockDetectorConfig {
    fn default() -> Self {
        Self {
            lock_contention_warning_threshold: Duration::from_millis(3000), // 3s of 5s timeout
            busy_retry_danger_threshold: 15,
        }
    }
}

/// Global atomic telemetry for database lock contention.
#[derive(Debug, Default)]
pub struct ConcurrencyContentionStats {
    pub total_busy_events: AtomicU64,
    pub lock_escalation_warnings: AtomicU64,
}

/// Active lock contention tracker for a single transaction or query session.
#[derive(Debug)]
pub struct LockSessionTracker {
    db_name: String,
    started_at: Instant,
    busy_retry_count: u32,
    config: DeadlockDetectorConfig,
}

impl LockSessionTracker {
    /// Starts tracking a database transaction or lock acquisition attempt.
    pub fn start(db_name: &str, config: Option<DeadlockDetectorConfig>) -> Self {
        Self {
            db_name: db_name.to_string(),
            started_at: Instant::now(),
            busy_retry_count: 0,
            config: config.unwrap_or_default(),
        }
    }

    /// Records a busy handler retry callback invocation.
    pub fn record_busy_retry(&mut self) -> bool {
        self.busy_retry_count += 1;
        self.busy_retry_count >= self.config.busy_retry_danger_threshold
    }

    /// Evaluates if current transaction lock wait duration exceeds warning levels.
    pub fn check_contention(&self) -> Option<Duration> {
        let elapsed = self.started_at.elapsed();
        if elapsed >= self.config.lock_contention_warning_threshold {
            Some(elapsed)
        } else {
            None
        }
    }

    /// Returns the database name being accessed.
    pub fn db_name(&self) -> &str {
        &self.db_name
    }

    /// Total busy retries observed in this session.
    pub fn busy_retries(&self) -> u32 {
        self.busy_retry_count
    }
}
