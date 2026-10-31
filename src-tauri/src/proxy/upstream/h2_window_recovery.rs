//! Upstream HTTP/2 Stream Window Recovery.
//! Monitors HTTP/2 flow-control window exhaustion and provides fallback to HTTP/1.1 chunked transfer.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

/// Default consecutive stall threshold triggering HTTP/1.1 downgrade.
pub const DEFAULT_H2_STALL_THRESHOLD: u32 = 3;

/// Tracker for HTTP/2 window stalls and protocol fallback.
#[derive(Debug, Clone)]
pub struct H2WindowRecoveryTracker {
    stall_counter: Arc<AtomicU32>,
    threshold: u32,
}

impl H2WindowRecoveryTracker {
    pub fn new(threshold: Option<u32>) -> Self {
        Self {
            stall_counter: Arc::new(AtomicU32::new(0)),
            threshold: threshold.unwrap_or(DEFAULT_H2_STALL_THRESHOLD),
        }
    }

    /// Records an HTTP/2 flow-control window exhaustion stall event.
    pub fn record_stall(&self) -> u32 {
        self.stall_counter.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Resets stall counter upon successful streaming completion.
    pub fn record_success(&self) {
        self.stall_counter.store(0, Ordering::SeqCst);
    }

    /// Returns whether the upstream client should downgrade to HTTP/1.1 due to persistent stalls.
    pub fn should_downgrade_to_http1(&self) -> bool {
        self.stall_counter.load(Ordering::SeqCst) >= self.threshold
    }

    /// Returns current consecutive stall count.
    pub fn stall_count(&self) -> u32 {
        self.stall_counter.load(Ordering::SeqCst)
    }
}

impl Default for H2WindowRecoveryTracker {
    fn default() -> Self {
        Self::new(None)
    }
}
