//! Instance Rapid Crash-Loop Quarantine Guard
//! Tracks process launch timestamps and immediate terminations (<5s)
//! to detect crash-loops and place offending instance profiles in temporary quarantine.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct CrashRecord {
    launch_time: Instant,
    consecutive_rapid_crashes: u32,
    quarantine_until: Option<Instant>,
}

#[derive(Debug, Clone, Default)]
pub struct CrashLoopGuard {
    records: Arc<RwLock<HashMap<String, CrashRecord>>>,
    rapid_crash_window: Duration,
    crash_threshold: u32,
    quarantine_duration: Duration,
}

impl CrashLoopGuard {
    pub fn new() -> Self {
        Self {
            records: Arc::new(RwLock::new(HashMap::new())),
            rapid_crash_window: Duration::from_secs(5),
            crash_threshold: 3,
            quarantine_duration: Duration::from_secs(120),
        }
    }

    /// Records that an instance process has launched.
    pub fn record_launch(&self, instance_id: &str) {
        if let Ok(mut guard) = self.records.write() {
            let entry = guard
                .entry(instance_id.to_string())
                .or_insert_with(|| CrashRecord {
                    launch_time: Instant::now(),
                    consecutive_rapid_crashes: 0,
                    quarantine_until: None,
                });
            entry.launch_time = Instant::now();
        }
    }

    /// Records that an instance process terminated.
    /// Returns true if the instance has just entered quarantine.
    pub fn record_termination(&self, instance_id: &str) -> bool {
        let now = Instant::now();
        let mut entered_quarantine = false;

        if let Ok(mut guard) = self.records.write() {
            if let Some(record) = guard.get_mut(instance_id) {
                let uptime = now.duration_since(record.launch_time);
                if uptime <= self.rapid_crash_window {
                    record.consecutive_rapid_crashes += 1;
                    if record.consecutive_rapid_crashes >= self.crash_threshold {
                        record.quarantine_until = Some(now + self.quarantine_duration);
                        entered_quarantine = true;
                    }
                } else {
                    // Healthy run, reset counter
                    record.consecutive_rapid_crashes = 0;
                    record.quarantine_until = None;
                }
            }
        }

        entered_quarantine
    }

    /// Checks if an instance is currently quarantined due to crash loops.
    pub fn is_quarantined(&self, instance_id: &str) -> bool {
        let now = Instant::now();
        if let Ok(guard) = self.records.read() {
            if let Some(record) = guard.get(instance_id) {
                if let Some(until) = record.quarantine_until {
                    return now < until;
                }
            }
        }
        false
    }

    /// Clears any crash records or quarantine status for an instance.
    pub fn reset(&self, instance_id: &str) {
        if let Ok(mut guard) = self.records.write() {
            guard.remove(instance_id);
        }
    }
}
