//! In-Flight SSE Stream Connection Counter.
//! Monitors real-time concurrent active streaming connections and peak throughput.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};

static ACTIVE_STREAMS: AtomicUsize = AtomicUsize::new(0);
static TOTAL_STREAMS: AtomicUsize = AtomicUsize::new(0);
static PEAK_STREAMS: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamConnectionMetrics {
    pub active_streams: usize,
    pub total_streams_served: usize,
    pub peak_concurrent_streams: usize,
}

/// RAII Guard that automatically decrements active stream count on drop.
pub struct StreamGuard;

impl Drop for StreamGuard {
    fn drop(&mut self) {
        ACTIVE_STREAMS.fetch_sub(1, Ordering::Relaxed);
    }
}

/// Registers the commencement of a new streaming response.
pub fn register_stream_start() -> StreamGuard {
    let current = ACTIVE_STREAMS.fetch_add(1, Ordering::Relaxed) + 1;
    TOTAL_STREAMS.fetch_add(1, Ordering::Relaxed);

    // Update peak concurrency if current exceeds it
    let mut peak = PEAK_STREAMS.load(Ordering::Relaxed);
    while current > peak {
        match PEAK_STREAMS.compare_exchange_weak(peak, current, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => break,
            Err(actual) => peak = actual,
        }
    }

    StreamGuard
}

/// Returns snapshot metrics of active and historical streaming connections.
pub fn get_stream_metrics() -> StreamConnectionMetrics {
    StreamConnectionMetrics {
        active_streams: ACTIVE_STREAMS.load(Ordering::Relaxed),
        total_streams_served: TOTAL_STREAMS.load(Ordering::Relaxed),
        peak_concurrent_streams: PEAK_STREAMS.load(Ordering::Relaxed),
    }
}
