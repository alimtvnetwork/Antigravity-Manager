//! Child Process Health Heartbeat Monitor.
//! Tracks heartbeat timestamps emitted by active IDE child instances,
//! flagging processes that have ceased heartbeat emissions as unresponsive or frozen.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Maximum allowed silence duration before marking an instance as unresponsive.
pub const HEARTBEAT_STALL_THRESHOLD: Duration = Duration::from_secs(45);

/// Heartbeat state record for an instance.
#[derive(Debug, Clone)]
pub struct ChildHeartbeatRecord {
    pub last_heartbeat_at: Instant,
    pub total_heartbeats: u64,
}

/// Centralized monitor tracking instance heartbeat pulses.
#[derive(Debug, Default)]
pub struct ChildHealthMonitor {
    instances: HashMap<String, ChildHeartbeatRecord>,
    stall_threshold: Duration,
}

impl ChildHealthMonitor {
    /// Creates a new monitor with default or custom stall timeout.
    pub fn new(stall_threshold: Option<Duration>) -> Self {
        Self {
            instances: HashMap::new(),
            stall_threshold: stall_threshold.unwrap_or(HEARTBEAT_STALL_THRESHOLD),
        }
    }

    /// Records receipt of a heartbeat pulse from an instance.
    pub fn record_heartbeat(&mut self, instance_id: &str) {
        let now = Instant::now();
        let entry = self.instances.entry(instance_id.to_string()).or_insert_with(|| ChildHeartbeatRecord {
            last_heartbeat_at: now,
            total_heartbeats: 0,
        });
        entry.last_heartbeat_at = now;
        entry.total_heartbeats += 1;
    }

    /// Evaluates if an instance has stalled or frozen.
    pub fn is_stalled(&self, instance_id: &str) -> bool {
        match self.instances.get(instance_id) {
            Some(record) => record.last_heartbeat_at.elapsed() > self.stall_threshold,
            None => true, // No heartbeat recorded yet
        }
    }

    /// Returns the duration since the last heartbeat was received.
    pub fn silence_duration(&self, instance_id: &str) -> Option<Duration> {
        self.instances.get(instance_id).map(|r| r.last_heartbeat_at.elapsed())
    }

    /// Deregisters an instance upon clean shutdown.
    pub fn remove_instance(&mut self, instance_id: &str) -> bool {
        self.instances.remove(instance_id).is_some()
    }
}
