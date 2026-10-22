//! Dynamic Busy Timeout Scaler
//!
//! Dynamically scales SQLite `PRAGMA busy_timeout` between 1,000ms and 10,000ms
//! according to moving average lock acquisition wait times, allowing transactions
//! sufficient headroom during bursty writes while failing fast when idle.

use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use tracing::debug;

pub const MIN_BUSY_TIMEOUT_MS: u32 = 1000;
pub const MAX_BUSY_TIMEOUT_MS: u32 = 10000;

/// Scaler managing dynamic busy timeout calculation.
pub struct BusyTimeoutScaler {
    current_timeout_ms: AtomicU32,
}

impl Default for BusyTimeoutScaler {
    fn default() -> Self {
        Self::new(3000)
    }
}

impl BusyTimeoutScaler {
    pub fn new(initial_ms: u32) -> Self {
        Self {
            current_timeout_ms: AtomicU32::new(initial_ms.clamp(MIN_BUSY_TIMEOUT_MS, MAX_BUSY_TIMEOUT_MS)),
        }
    }

    /// Records an observed lock wait duration and updates the busy timeout.
    pub fn on_lock_wait(&self, wait_duration: Duration) -> u32 {
        let wait_ms = wait_duration.as_millis() as u32;

        let new_timeout = if wait_ms > 500 {
            // Escalate timeout if lock contention is high
            (wait_ms * 3).clamp(MIN_BUSY_TIMEOUT_MS, MAX_BUSY_TIMEOUT_MS)
        } else {
            // Gradually decay back toward minimum
            let current = self.current_timeout_ms.load(Ordering::Relaxed);
            ((current * 4 + MIN_BUSY_TIMEOUT_MS) / 5).clamp(MIN_BUSY_TIMEOUT_MS, MAX_BUSY_TIMEOUT_MS)
        };

        self.current_timeout_ms.store(new_timeout, Ordering::Relaxed);
        debug!(
            observed_wait_ms = wait_ms,
            scaled_timeout_ms = new_timeout,
            "Updated SQLite dynamic busy timeout"
        );

        new_timeout
    }

    /// Gets the current active busy timeout in milliseconds.
    pub fn current_timeout(&self) -> u32 {
        self.current_timeout_ms.load(Ordering::Relaxed)
    }
}
