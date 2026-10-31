//! Cooldown Event & Rotation Telemetry Reporter
//!
//! Captures and buffers structured auto-switcher telemetry events (cooldown entrance,
//! cooldown exit, candidate rejection, quota throttle triggers) for auditing, UI inspection,
//! and decentralized fleet telemetry broadcasting.

use std::sync::Mutex;
use tracing::info;

/// Severity classification for auto-switcher events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TelemetrySeverity {
    Info,
    Warning,
    Critical,
}

/// Structured record representing an auto-switcher event.
#[derive(Debug, Clone)]
pub struct AutoSwitcherTelemetryEvent {
    pub timestamp_epoch_ms: u64,
    pub account_id: String,
    pub instance_id: Option<String>,
    pub action_type: String,
    pub reason: String,
    pub duration_secs: Option<u64>,
    pub severity: TelemetrySeverity,
}

/// Buffer for storing recent auto-switcher telemetry events (bounded ring buffer).
pub struct CooldownTelemetryBuffer {
    events: Mutex<Vec<AutoSwitcherTelemetryEvent>>,
    max_capacity: usize,
}

impl CooldownTelemetryBuffer {
    /// Creates a new telemetry buffer with specified event capacity (default 250).
    pub fn new(max_capacity: usize) -> Self {
        Self {
            events: Mutex::new(Vec::with_capacity(max_capacity)),
            max_capacity: max_capacity.max(10),
        }
    }

    /// Records an event into the bounded buffer.
    pub fn record_event(&self, event: AutoSwitcherTelemetryEvent) {
        let mut list = match self.events.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };

        if list.len() >= self.max_capacity {
            list.remove(0);
        }

        info!(
            account_id = %event.account_id,
            action = %event.action_type,
            reason = %event.reason,
            "Auto-switcher telemetry event captured"
        );

        list.push(event);
    }

    /// Retrieves a cloned snapshot of all currently recorded events.
    pub fn get_recent_events(&self) -> Vec<AutoSwitcherTelemetryEvent> {
        let list = match self.events.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        list.clone()
    }

    /// Clears the event buffer.
    pub fn clear(&self) {
        let mut list = match self.events.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        list.clear();
    }
}
