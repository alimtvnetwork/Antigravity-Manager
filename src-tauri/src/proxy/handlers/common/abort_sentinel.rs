//! Client Disconnect Stream Abort Sentinel.
//! Monitors downstream HTTP client connection drops, immediately terminating the upstream generative AI stream.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::Mutex;

/// Sentinel token tracking client connection vitality and abort status.
#[derive(Debug, Clone)]
pub struct StreamAbortSentinel {
    is_aborted: Arc<AtomicBool>,
    abort_reason: Arc<Mutex<Option<String>>>,
}

impl StreamAbortSentinel {
    pub fn new() -> Self {
        Self {
            is_aborted: Arc::new(AtomicBool::new(false)),
            abort_reason: Arc::new(Mutex::new(None)),
        }
    }

    /// Triggers an immediate abort signal with the specified reason.
    pub fn trigger_abort(&self, reason: &str) {
        self.is_aborted.store(true, Ordering::Release);
        if let Ok(mut lock) = self.abort_reason.lock() {
            *lock = Some(reason.to_string());
        }
    }

    /// Evaluates whether the stream has been aborted by client disconnect or cancellation.
    pub fn is_aborted(&self) -> bool {
        self.is_aborted.load(Ordering::Acquire)
    }

    /// Returns the recorded abort reason, if aborted.
    pub fn reason(&self) -> Option<String> {
        self.abort_reason.lock().ok()?.clone()
    }
}

impl Default for StreamAbortSentinel {
    fn default() -> Self {
        Self::new()
    }
}
