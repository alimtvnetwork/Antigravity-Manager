//! SSE Heartbeat Generator & Keepalive Tick Evaluator (`Imp-616`)
//!
//! Emits periodic non-disruptive SSE keepalive comment frames (`:\n\n` or `: ping\n\n`)
//! to prevent client and intermediate proxy TCP socket timeouts during extended LLM
//! deliberation or deep thinking cycles without polluting client data streams.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Configuration for periodic SSE keepalive heartbeat emissions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeartbeatConfig {
    /// Keepalive heartbeat interval in milliseconds (default 15,000ms / 15s).
    pub interval_ms: u64,
    /// SSE comment payload (must follow SSE comment format starting with `:` and terminating with `\n\n`).
    pub ping_payload: String,
    /// Minimum allowed interval in milliseconds to prevent socket saturation (default 5,000ms).
    pub min_interval_ms: u64,
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self {
            interval_ms: 15_000,
            ping_payload: ": ping\n\n".to_string(),
            min_interval_ms: 5_000,
        }
    }
}

impl HeartbeatConfig {
    /// Creates a new `HeartbeatConfig` with the specified interval and payload.
    pub fn new(interval_ms: u64, ping_payload: impl Into<String>) -> Self {
        Self {
            interval_ms,
            ping_payload: ping_payload.into(),
            min_interval_ms: 5_000,
        }
    }

    /// Sets a custom minimum interval threshold in milliseconds.
    pub fn with_min_interval(mut self, min_interval_ms: u64) -> Self {
        self.min_interval_ms = min_interval_ms;
        self
    }

    /// Creates a lightweight minimal comment configuration (`:\n\n`).
    pub fn minimal_comment() -> Self {
        Self {
            interval_ms: 15_000,
            ping_payload: ":\n\n".to_string(),
            min_interval_ms: 5_000,
        }
    }

    /// Returns the effective interval clamped by `min_interval_ms`.
    pub fn effective_interval_ms(&self) -> u64 {
        self.interval_ms.max(self.min_interval_ms)
    }
}

/// Dynamic state tracker for an active streaming turn.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct HeartbeatGeneratorState {
    /// Millisecond epoch timestamp when genuine LLM payload or event data was last observed.
    pub last_event_epoch_ms: u64,
    /// Millisecond epoch timestamp when an SSE keepalive ping was last emitted.
    pub last_ping_epoch_ms: u64,
    /// Cumulative count of keepalive ping frames emitted during this stream.
    pub total_pings_sent: u64,
}

impl HeartbeatGeneratorState {
    /// Initializes state with an initial start timestamp.
    pub fn new(initial_epoch_ms: u64) -> Self {
        Self {
            last_event_epoch_ms: initial_epoch_ms,
            last_ping_epoch_ms: 0,
            total_pings_sent: 0,
        }
    }

    /// Records genuine payload or token arrival, resetting the keepalive countdown.
    pub fn record_activity(&mut self, now_epoch_ms: u64) {
        self.last_event_epoch_ms = now_epoch_ms;
    }

    /// Returns the most recent activity timestamp (maximum of genuine event and last ping).
    pub fn last_activity_epoch_ms(&self) -> u64 {
        self.last_event_epoch_ms.max(self.last_ping_epoch_ms)
    }

    /// Calculates milliseconds remaining until the next keepalive ping tick is due.
    pub fn time_until_next_tick_ms(&self, config: &HeartbeatConfig, now_epoch_ms: u64) -> u64 {
        let effective_interval = config.effective_interval_ms();
        let last_activity = self.last_activity_epoch_ms();
        if now_epoch_ms < last_activity {
            return effective_interval;
        }
        let elapsed = now_epoch_ms - last_activity;
        effective_interval.saturating_sub(elapsed)
    }

    /// Resets the generator state to a new initial timestamp.
    pub fn reset(&mut self, now_epoch_ms: u64) {
        self.last_event_epoch_ms = now_epoch_ms;
        self.last_ping_epoch_ms = 0;
        self.total_pings_sent = 0;
    }
}

/// Action to be taken on each heartbeat timer evaluation tick.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeartbeatFrameAction {
    /// An SSE keepalive ping comment frame should be dispatched to the client.
    EmitPing(String),
    /// No action required; the stream is within its quiet window or active.
    Idle,
}

impl HeartbeatFrameAction {
    /// Returns true if this action emits a ping frame.
    pub fn is_emit(&self) -> bool {
        matches!(self, Self::EmitPing(_))
    }

    /// Returns true if this action is idle.
    pub fn is_idle(&self) -> bool {
        matches!(self, Self::Idle)
    }

    /// Returns a reference to the payload if action is `EmitPing`.
    pub fn payload(&self) -> Option<&str> {
        match self {
            Self::EmitPing(payload) => Some(payload.as_str()),
            Self::Idle => None,
        }
    }
}

impl fmt::Display for HeartbeatFrameAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmitPing(p) => write!(f, "EmitPing({:?})", p.trim()),
            Self::Idle => write!(f, "Idle"),
        }
    }
}

/// Evaluates a timer tick against the generator state and configuration.
///
/// If elapsed time since last activity (genuine payload event or preceding ping)
/// equals or exceeds the effective interval (`config.interval_ms.max(config.min_interval_ms)`),
/// this function returns `HeartbeatFrameAction::EmitPing(config.ping_payload)` and updates
/// `state.last_ping_epoch_ms` and `state.total_pings_sent`.
///
/// Clock skew where `now_epoch_ms < last_activity` is safely handled by returning `Idle`.
pub fn evaluate_heartbeat_tick(
    config: &HeartbeatConfig,
    state: &mut HeartbeatGeneratorState,
    now_epoch_ms: u64,
) -> HeartbeatFrameAction {
    let effective_interval = config.effective_interval_ms();
    let last_activity = state.last_activity_epoch_ms();

    // Defensive guard against clock skew / backwards time jump
    if now_epoch_ms < last_activity {
        return HeartbeatFrameAction::Idle;
    }

    let elapsed = now_epoch_ms - last_activity;
    if elapsed >= effective_interval {
        state.last_ping_epoch_ms = now_epoch_ms;
        state.total_pings_sent = state.total_pings_sent.saturating_add(1);
        HeartbeatFrameAction::EmitPing(config.ping_payload.clone())
    } else {
        HeartbeatFrameAction::Idle
    }
}

/// Formats a raw comment string into standard SSE comment wire format (`: <comment>\n\n`).
pub fn format_sse_comment(comment: &str) -> String {
    let trimmed = comment.trim();
    if trimmed.is_empty() {
        ":\n\n".to_string()
    } else {
        format!(": {}\n\n", trimmed)
    }
}

/// Validates whether a frame string qualifies as a well-formed SSE comment frame.
pub fn is_valid_sse_comment_frame(frame: &str) -> bool {
    let trimmed = frame.trim_start();
    trimmed.starts_with(':') && frame.ends_with("\n\n")
}
