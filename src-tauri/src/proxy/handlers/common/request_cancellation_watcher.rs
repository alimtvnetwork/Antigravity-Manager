//! Client Request Cancellation Watcher (Imp-542)
//! Monitors client connection drops and early aborts via atomic cancellation flags,
//! signaling streaming loops to stop pulling upstream tokens and close network sockets.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct RequestCancellationWatcher {
    is_cancelled: Arc<AtomicBool>,
}

impl Default for RequestCancellationWatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl RequestCancellationWatcher {
    /// Creates a new uncancelled request watcher.
    pub fn new() -> Self {
        Self {
            is_cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Triggers cancellation, notifying any streaming tasks to terminate.
    pub fn cancel(&self) {
        self.is_cancelled.store(true, Ordering::SeqCst);
    }

    /// Checks if the request has been cancelled by client or timeout.
    pub fn is_cancelled(&self) -> bool {
        self.is_cancelled.load(Ordering::SeqCst)
    }

    /// Returns a clone of the underlying atomic flag for cross-thread sharing.
    pub fn flag(&self) -> Arc<AtomicBool> {
        self.is_cancelled.clone()
    }
}
