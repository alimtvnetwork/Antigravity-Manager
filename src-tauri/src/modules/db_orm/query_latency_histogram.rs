//! Query Latency Histogram Collector
//!
//! Bucketizes SQLite query execution durations into fixed latency tiers
//! (<1ms, 1-5ms, 5-20ms, >20ms) for high-performance zero-allocation runtime profiling.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// Fixed latency bucket counter.
#[derive(Default)]
pub struct QueryLatencyHistogram {
    pub under_1ms: AtomicU64,
    pub between_1_and_5ms: AtomicU64,
    pub between_5_and_20ms: AtomicU64,
    pub over_20ms: AtomicU64,
    pub total_queries: AtomicU64,
}

/// Snapshot of histogram counts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistogramSnapshot {
    pub under_1ms: u64,
    pub between_1_and_5ms: u64,
    pub between_5_and_20ms: u64,
    pub over_20ms: u64,
    pub total_queries: u64,
}

impl QueryLatencyHistogram {
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a query duration into the appropriate histogram bucket.
    pub fn record_query(&self, duration: Duration) {
        self.total_queries.fetch_add(1, Ordering::Relaxed);
        let millis = duration.as_millis();

        if millis < 1 {
            self.under_1ms.fetch_add(1, Ordering::Relaxed);
        } else if millis < 5 {
            self.between_1_and_5ms.fetch_add(1, Ordering::Relaxed);
        } else if millis < 20 {
            self.between_5_and_20ms.fetch_add(1, Ordering::Relaxed);
        } else {
            self.over_20ms.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Captures a snapshot of current counts.
    pub fn snapshot(&self) -> HistogramSnapshot {
        HistogramSnapshot {
            under_1ms: self.under_1ms.load(Ordering::Relaxed),
            between_1_and_5ms: self.between_1_and_5ms.load(Ordering::Relaxed),
            between_5_and_20ms: self.between_5_and_20ms.load(Ordering::Relaxed),
            over_20ms: self.over_20ms.load(Ordering::Relaxed),
            total_queries: self.total_queries.load(Ordering::Relaxed),
        }
    }

    /// Resets all counters to zero.
    pub fn reset(&self) {
        self.under_1ms.store(0, Ordering::Relaxed);
        self.between_1_and_5ms.store(0, Ordering::Relaxed);
        self.between_5_and_20ms.store(0, Ordering::Relaxed);
        self.over_20ms.store(0, Ordering::Relaxed);
        self.total_queries.store(0, Ordering::Relaxed);
    }
}
