//! SSE Stream Heartbeat & Keepalive Flusher.
//! Emits SSE comment keepalives during idle periods to prevent intermediary reverse proxy timeouts.

use std::time::{Duration, Instant};

pub const DEFAULT_SSE_KEEPALIVE_INTERVAL: Duration = Duration::from_secs(15);
pub const SSE_KEEPALIVE_BYTES: &[u8] = b": keepalive\n\n";

#[derive(Debug)]
pub struct StreamFlusher {
    last_frame_time: Instant,
    keepalive_interval: Duration,
}

impl Default for StreamFlusher {
    fn default() -> Self {
        Self::new(None)
    }
}

impl StreamFlusher {
    pub fn new(interval: Option<Duration>) -> Self {
        Self {
            last_frame_time: Instant::now(),
            keepalive_interval: interval.unwrap_or(DEFAULT_SSE_KEEPALIVE_INTERVAL),
        }
    }

    /// Resets the elapsed timer when genuine payload data is sent.
    pub fn record_data_sent(&mut self) {
        self.last_frame_time = Instant::now();
    }

    /// Returns true if the idle duration since the last frame exceeds the keepalive threshold.
    pub fn should_emit_keepalive(&self) -> bool {
        self.last_frame_time.elapsed() >= self.keepalive_interval
    }

    /// Returns the static SSE comment frame bytes (`: keepalive\n\n`).
    pub fn get_keepalive_frame(&mut self) -> &'static [u8] {
        self.last_frame_time = Instant::now();
        SSE_KEEPALIVE_BYTES
    }
}
