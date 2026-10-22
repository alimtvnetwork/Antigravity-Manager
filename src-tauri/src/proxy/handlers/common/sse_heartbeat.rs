//! Stream SSE Heartbeat Generator.
//! Emits periodic non-disruptive SSE keepalive comments (`: ping - keepalive\n\n`) during extended model thinking cycles.

use std::time::{Duration, Instant};

/// Standard non-disruptive SSE comment frame.
pub const SSE_HEARTBEAT_COMMENT: &[u8] = b": ping - keepalive\n\n";

/// Default keepalive interval during thinking phases: 15 seconds.
pub const DEFAULT_SSE_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(15);

/// Generates raw bytes for an SSE keepalive heartbeat frame.
pub fn generate_sse_heartbeat() -> &'static [u8] {
    SSE_HEARTBEAT_COMMENT
}

/// Evaluates whether an SSE keepalive frame should be emitted based on elapsed time.
pub fn should_emit_heartbeat(last_activity: Instant, interval: Option<Duration>) -> bool {
    let threshold = interval.unwrap_or(DEFAULT_SSE_HEARTBEAT_INTERVAL);
    last_activity.elapsed() >= threshold
}

/// Tracks heartbeat timing for an active streaming turn.
#[derive(Debug, Clone, Copy)]
pub struct SseHeartbeatTracker {
    last_frame: Instant,
    interval: Duration,
}

impl SseHeartbeatTracker {
    pub fn new(interval: Option<Duration>) -> Self {
        Self {
            last_frame: Instant::now(),
            interval: interval.unwrap_or(DEFAULT_SSE_HEARTBEAT_INTERVAL),
        }
    }

    /// Records that real payload or a heartbeat was just emitted.
    pub fn touch(&mut self) {
        self.last_frame = Instant::now();
    }

    /// Returns Some(bytes) if interval has elapsed and updates internal timer, else None.
    pub fn check_heartbeat(&mut self) -> Option<&'static [u8]> {
        if self.last_frame.elapsed() >= self.interval {
            self.touch();
            Some(SSE_HEARTBEAT_COMMENT)
        } else {
            None
        }
    }
}
