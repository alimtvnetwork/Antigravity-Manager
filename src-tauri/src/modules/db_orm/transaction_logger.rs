//! Transaction Latency & Health Logger
//! Tracks transaction duration from begin to commit/rollback and emits
//! diagnostic telemetry warnings when execution latency exceeds safe thresholds.

use std::time::{Duration, Instant};
use tracing::{debug, warn};

pub struct TransactionTimer {
    tag: &'static str,
    start_time: Instant,
    warn_threshold: Duration,
}

impl TransactionTimer {
    pub fn new(tag: &'static str) -> Self {
        Self::with_threshold(tag, Duration::from_millis(50))
    }

    pub fn with_threshold(tag: &'static str, warn_threshold: Duration) -> Self {
        Self {
            tag,
            start_time: Instant::now(),
            warn_threshold,
        }
    }

    /// Records successful commit. Emits warning if elapsed time exceeded threshold.
    pub fn record_commit(self) -> Duration {
        let elapsed = self.start_time.elapsed();
        if elapsed >= self.warn_threshold {
            warn!(
                target: "db_orm::transaction",
                "Slow SQLite transaction commit [{}]: took {:?} (threshold: {:?})",
                self.tag, elapsed, self.warn_threshold
            );
        } else {
            debug!(
                target: "db_orm::transaction",
                "Transaction commit [{}]: took {:?}",
                self.tag, elapsed
            );
        }
        elapsed
    }

    /// Records transaction rollback.
    pub fn record_rollback(self, reason: &str) -> Duration {
        let elapsed = self.start_time.elapsed();
        warn!(
            target: "db_orm::transaction",
            "Transaction rollback [{}] after {:?}: reason={}",
            self.tag, elapsed, reason
        );
        elapsed
    }
}
