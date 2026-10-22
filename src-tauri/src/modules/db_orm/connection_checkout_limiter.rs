// Connection Pool Checkout Rate Limiter (Imp-494)
// Prevents asynchronous background batch jobs from exhausting connection
// pool capacity, reserving checkout tokens for foreground UI interactions.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub const DEFAULT_MAX_BACKGROUND_CHECKOUTS: usize = 4;

#[derive(Debug, Clone)]
pub struct ConnectionCheckoutLimiter {
    max_background_checkouts: usize,
    active_background_checkouts: Arc<AtomicUsize>,
}

impl Default for ConnectionCheckoutLimiter {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_BACKGROUND_CHECKOUTS)
    }
}

impl ConnectionCheckoutLimiter {
    pub fn new(max_background: usize) -> Self {
        Self {
            max_background_checkouts: if max_background == 0 { DEFAULT_MAX_BACKGROUND_CHECKOUTS } else { max_background },
            active_background_checkouts: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Attempts to acquire a background checkout permit.
    /// Returns a guard on success, or None if the capacity is saturated.
    pub fn try_acquire_background_permit(&self) -> Option<BackgroundCheckoutGuard> {
        loop {
            let current = self.active_background_checkouts.load(Ordering::Acquire);
            if current >= self.max_background_checkouts {
                return None;
            }
            if self.active_background_checkouts
                .compare_exchange_weak(current, current + 1, Ordering::SeqCst, Ordering::Acquire)
                .is_ok()
            {
                return Some(BackgroundCheckoutGuard {
                    counter: Arc::clone(&self.active_background_checkouts),
                });
            }
        }
    }

    pub fn current_background_load(&self) -> usize {
        self.active_background_checkouts.load(Ordering::Relaxed)
    }
}

pub struct BackgroundCheckoutGuard {
    counter: Arc<AtomicUsize>,
}

impl Drop for BackgroundCheckoutGuard {
    fn drop(&mut self) {
        self.counter.fetch_sub(1, Ordering::SeqCst);
    }
}
