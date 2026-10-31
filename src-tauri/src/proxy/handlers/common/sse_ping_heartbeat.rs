//! SSE comment ping heartbeat generator.
//! Injects standard SSE comment frames (`: keepalive <timestamp>\n\n`)
//! to keep client and intermediate proxy TCP sockets alive during long
//! reasoning or thinking phases without breaking SSE protocol semantics.

use std::time::{Duration, Instant};

/// Configuration for SSE ping heartbeats.
#[derive(Debug, Clone)]
pub struct SsePingConfig {
    /// Interval after which a keepalive comment should be emitted if no payload arrived.
    pub ping_interval: Duration,
    /// Prefix label for the comment frame.
    pub label: String,
}

impl Default for SsePingConfig {
    fn default() -> Self {
        Self {
            ping_interval: Duration::from_secs(15),
            label: "keepalive".to_string(),
        }
    }
}

/// State tracker for emitting SSE comment heartbeats.
#[derive(Debug)]
pub struct SsePingHeartbeat {
    config: SsePingConfig,
    last_event_time: Instant,
}

impl SsePingHeartbeat {
    /// Creates a new ping heartbeat tracker.
    pub fn new(config: Option<SsePingConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
            last_event_time: Instant::now(),
        }
    }

    /// Records that genuine payload data was dispatched, resetting the ping timer.
    pub fn record_activity(&mut self) {
        self.last_event_time = Instant::now();
    }

    /// Checks if a keepalive ping frame is due.
    /// If due, resets the timer and returns the formatted SSE comment frame.
    pub fn poll_keepalive(&mut self) -> Option<String> {
        let now = Instant::now();
        if now.duration_since(self.last_event_time) >= self.config.ping_interval {
            self.last_event_time = now;
            let unix_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0);
            Some(format!(": {} {}\n\n", self.config.label, unix_ms))
        } else {
            None
        }
    }

    /// Returns time remaining until the next keepalive ping.
    pub fn time_until_next_ping(&self) -> Duration {
        let elapsed = self.last_event_time.elapsed();
        if elapsed >= self.config.ping_interval {
            Duration::ZERO
        } else {
            self.config.ping_interval - elapsed
        }
    }
}
