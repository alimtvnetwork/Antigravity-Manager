//! Adaptive Connection Pool Dynamic Sizer
//!
//! Adjusts maximum idle connections and connection pool limits dynamically (5 to 50)
//! per upstream host based on real-time active request concurrency to prevent connection
//! starvation under heavy load while avoiding resource bloat when traffic is low.

use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::debug;

/// Minimum idle connections to keep in the pool.
pub const MIN_IDLE_CONNECTIONS: usize = 5;

/// Maximum idle connections allowable during peak concurrency.
pub const MAX_IDLE_CONNECTIONS: usize = 50;

/// Controller managing adaptive connection pool sizing.
pub struct DynamicPoolSizer {
    active_in_flight_requests: AtomicUsize,
    current_pool_limit: AtomicUsize,
}

impl Default for DynamicPoolSizer {
    fn default() -> Self {
        Self::new()
    }
}

impl DynamicPoolSizer {
    pub fn new() -> Self {
        Self {
            active_in_flight_requests: AtomicUsize::new(0),
            current_pool_limit: AtomicUsize::new(MIN_IDLE_CONNECTIONS),
        }
    }

    /// Increments active in-flight request counter and calculates required pool size.
    pub fn on_request_started(&self) -> usize {
        let count = self
            .active_in_flight_requests
            .fetch_add(1, Ordering::SeqCst)
            + 1;
        self.recalculate(count)
    }

    /// Decrements active in-flight request counter and recalculates pool size.
    pub fn on_request_finished(&self) -> usize {
        let prev = self
            .active_in_flight_requests
            .fetch_sub(1, Ordering::SeqCst);
        let count = prev.saturating_sub(1);
        self.recalculate(count)
    }

    /// Returns the current dynamic pool limit.
    pub fn current_limit(&self) -> usize {
        self.current_pool_limit.load(Ordering::Relaxed)
    }

    fn recalculate(&self, active_count: usize) -> usize {
        // Pool size allocates roughly 1 connection per active request plus 5 headroom
        let target = (active_count + 5).clamp(MIN_IDLE_CONNECTIONS, MAX_IDLE_CONNECTIONS);
        let prev = self.current_pool_limit.swap(target, Ordering::Relaxed);

        if prev != target {
            debug!(
                active_requests = active_count,
                new_limit = target,
                "Dynamically adjusted upstream connection pool size"
            );
        }

        target
    }
}
