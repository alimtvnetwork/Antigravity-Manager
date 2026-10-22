//! Stream early abort sentinel.
//! Detects client-side TCP disconnects, request cancellations, and timeout interrupts
//! during active SSE token generation, immediately propagating cancellation upstream.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone)]
pub struct StreamEarlyAbortSentinel {
    is_aborted: Arc<AtomicBool>,
    token: CancellationToken,
}

impl StreamEarlyAbortSentinel {
    pub fn new() -> Self {
        Self {
            is_aborted: Arc::new(AtomicBool::new(false)),
            token: CancellationToken::new(),
        }
    }

    /// Triggers an immediate abort signal.
    pub fn trigger_abort(&self) {
        self.is_aborted.store(true, Ordering::SeqCst);
        self.token.cancel();
    }

    /// Checks if the stream has been aborted by client disconnect or deadline.
    pub fn is_aborted(&self) -> bool {
        self.is_aborted.load(Ordering::SeqCst) || self.token.is_cancelled()
    }

    /// Returns a child cancellation token for passing to Tokio async futures.
    pub fn cancellation_token(&self) -> CancellationToken {
        self.token.clone()
    }
}

impl Default for StreamEarlyAbortSentinel {
    fn default() -> Self {
        Self::new()
    }
}
