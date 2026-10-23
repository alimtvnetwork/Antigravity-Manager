//! Split-Database Connection Pool Metrics Collector
//! Aggregates real-time connection telemetry across decoupled SQLite databases
//! to monitor pool utilization and prevent connection starvation.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct SplitDbPoolMetrics {
    pub active_readers: AtomicUsize,
    pub active_writers: AtomicUsize,
    pub total_queries_executed: AtomicU64,
    pub total_slow_queries: AtomicU64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolMetricsSnapshot {
    pub active_readers: usize,
    pub active_writers: usize,
    pub total_queries: u64,
    pub slow_queries: u64,
}

#[derive(Debug, Clone, Default)]
pub struct ConnectionPoolMetrics {
    inner: Arc<SplitDbPoolMetrics>,
}

impl ConnectionPoolMetrics {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(SplitDbPoolMetrics::default()),
        }
    }

    pub fn acquire_reader(&self) {
        self.inner.active_readers.fetch_add(1, Ordering::Relaxed);
    }

    pub fn release_reader(&self) {
        self.inner.active_readers.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn acquire_writer(&self) {
        self.inner.active_writers.fetch_add(1, Ordering::Relaxed);
    }

    pub fn release_writer(&self) {
        self.inner.active_writers.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn record_query(&self, is_slow: bool) {
        self.inner
            .total_queries_executed
            .fetch_add(1, Ordering::Relaxed);
        if is_slow {
            self.inner
                .total_slow_queries
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn snapshot(&self) -> PoolMetricsSnapshot {
        PoolMetricsSnapshot {
            active_readers: self.inner.active_readers.load(Ordering::Relaxed),
            active_writers: self.inner.active_writers.load(Ordering::Relaxed),
            total_queries: self.inner.total_queries_executed.load(Ordering::Relaxed),
            slow_queries: self.inner.total_slow_queries.load(Ordering::Relaxed),
        }
    }
}
