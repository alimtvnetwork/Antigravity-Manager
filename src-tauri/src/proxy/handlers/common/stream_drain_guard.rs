//! Upstream Stream Drain Cancellation Guard
//!
//! Handles cleanup when a downstream client terminates early or aborts a streaming request.
//! Consumes residual HTTP body bytes within a tight deadline (default 250ms / 64KB)
//! allowing the underlying HTTP/2 connection to be returned cleanly to the connection pool.

use std::time::Duration;
use tracing::debug;

pub const DEFAULT_MAX_DRAIN_BYTES: usize = 64 * 1024;
pub const DEFAULT_MAX_DRAIN_DURATION: Duration = Duration::from_millis(250);

/// Action decided for an aborted stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrainDecision {
    /// Safe to drain residual bytes and recycle connection.
    DrainAndRecycle,
    /// Stream is too large or slow; discard connection immediately.
    DropConnection,
}

/// Evaluates whether an aborted stream should be drained or dropped.
pub fn evaluate_drain_action(
    remaining_bytes_estimate: Option<usize>,
    elapsed_time: Duration,
) -> DrainDecision {
    if elapsed_time > DEFAULT_MAX_DRAIN_DURATION {
        return DrainDecision::DropConnection;
    }

    match remaining_bytes_estimate {
        Some(bytes) if bytes > DEFAULT_MAX_DRAIN_BYTES => {
            debug!(
                bytes = bytes,
                "Aborted stream too large to drain, dropping connection"
            );
            DrainDecision::DropConnection
        }
        _ => {
            debug!("Draining residual bytes to recycle upstream connection");
            DrainDecision::DrainAndRecycle
        }
    }
}
