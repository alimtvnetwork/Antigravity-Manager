//! Client Disconnect Stream Abort Guard.
//! Propagates downstream client TCP connection drops to immediately abort upstream streaming requests.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct StreamAbortHandle {
    cancelled: Arc<AtomicBool>,
}

#[derive(Debug, Clone)]
pub struct StreamAbortReceiver {
    cancelled: Arc<AtomicBool>,
}

impl StreamAbortHandle {
    /// Creates a paired abort handle and receiver.
    pub fn new() -> (Self, StreamAbortReceiver) {
        let flag = Arc::new(AtomicBool::new(false));
        (
            Self {
                cancelled: Arc::clone(&flag),
            },
            StreamAbortReceiver {
                cancelled: Arc::clone(&flag),
            },
        )
    }

    /// Triggers immediate cancellation of upstream work.
    pub fn abort(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
}

impl StreamAbortReceiver {
    /// Returns true if downstream consumer dropped or requested abort.
    pub fn is_aborted(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }
}
