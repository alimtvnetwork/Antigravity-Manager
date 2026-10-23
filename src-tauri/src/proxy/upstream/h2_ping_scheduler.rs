//! HTTP/2 Keepalive Ping Scheduler (Imp-546)
//! Schedules periodic HTTP/2 PING frames and tracks round-trip acknowledgment (PONG)
//! to prevent middlebox drops during long-running reasoning model streams.

use serde::{Deserialize, Serialize};

pub const DEFAULT_H2_PING_INTERVAL_SECS: u64 = 30;
pub const DEFAULT_H2_PING_TIMEOUT_SECS: u64 = 10;
pub const DEFAULT_MAX_UNACKNOWLEDGED_PINGS: u32 = 3;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct H2PingScheduler {
    pub ping_interval_secs: u64,
    pub ping_timeout_secs: u64,
    pub max_unacknowledged: u32,
    pub last_ping_sec: i64,
    pub last_pong_sec: i64,
    pub unacknowledged_pings: u32,
    pub last_rtt_ms: Option<u64>,
}

impl Default for H2PingScheduler {
    fn default() -> Self {
        Self::new(None, None, None)
    }
}

impl H2PingScheduler {
    pub fn new(interval: Option<u64>, timeout: Option<u64>, max_unack: Option<u32>) -> Self {
        Self {
            ping_interval_secs: interval.unwrap_or(DEFAULT_H2_PING_INTERVAL_SECS),
            ping_timeout_secs: timeout.unwrap_or(DEFAULT_H2_PING_TIMEOUT_SECS),
            max_unacknowledged: max_unack.unwrap_or(DEFAULT_MAX_UNACKNOWLEDGED_PINGS),
            last_ping_sec: 0,
            last_pong_sec: 0,
            unacknowledged_pings: 0,
            last_rtt_ms: None,
        }
    }

    /// Evaluates if an HTTP/2 PING frame should be dispatched.
    pub fn should_send_ping(&self, now_sec: i64) -> bool {
        if self.last_ping_sec == 0 {
            true
        } else {
            now_sec.saturating_sub(self.last_ping_sec) >= self.ping_interval_secs as i64
        }
    }

    /// Records dispatch of a PING frame at epoch seconds.
    pub fn record_ping_sent(&mut self, now_sec: i64) {
        self.last_ping_sec = now_sec;
        self.unacknowledged_pings = self.unacknowledged_pings.saturating_add(1);
    }

    /// Records arrival of an acknowledged PONG frame and round-trip duration.
    pub fn record_pong_received(&mut self, now_sec: i64, rtt_ms: Option<u64>) {
        self.last_pong_sec = now_sec;
        self.unacknowledged_pings = 0;
        self.last_rtt_ms = rtt_ms;
    }

    /// Checks if connection should be considered dead due to unacknowledged PINGs or timeout.
    pub fn is_connection_stale(&self, now_sec: i64) -> bool {
        if self.unacknowledged_pings >= self.max_unacknowledged {
            return true;
        }
        if self.unacknowledged_pings > 0 && self.last_ping_sec > 0 {
            let elapsed = now_sec.saturating_sub(self.last_ping_sec);
            if elapsed >= self.ping_timeout_secs as i64 {
                return true;
            }
        }
        false
    }

    /// Resets ping accounting state.
    pub fn reset(&mut self) {
        self.last_ping_sec = 0;
        self.last_pong_sec = 0;
        self.unacknowledged_pings = 0;
        self.last_rtt_ms = None;
    }
}
