//! Crash Loop Exponential Backoff Guard.
//! Tracks successive process termination events occurring within short runtimes (<10s),
//! enforcing escalating relaunch backoff delays to prevent high-frequency crash loops.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Crash loop tracking entry for an instance.
#[derive(Debug, Clone)]
pub struct InstanceCrashRecord {
    pub consecutive_crashes: u32,
    pub last_launched_at: Instant,
    pub last_crashed_at: Instant,
}

/// Guard preventing crash loops by calculating required backoff delays.
#[derive(Debug, Default)]
pub struct CrashLoopBackoffGuard {
    records: HashMap<String, InstanceCrashRecord>,
    short_lifespan_threshold: Duration,
    max_backoff: Duration,
}

impl CrashLoopBackoffGuard {
    /// Creates a guard with default or custom threshold durations.
    pub fn new(short_lifespan: Option<Duration>, max_backoff: Option<Duration>) -> Self {
        Self {
            records: HashMap::new(),
            short_lifespan_threshold: short_lifespan.unwrap_or(Duration::from_secs(10)),
            max_backoff: max_backoff.unwrap_or(Duration::from_secs(30)),
        }
    }

    /// Records when an instance was launched.
    pub fn record_launch(&mut self, instance_id: &str) {
        let now = Instant::now();
        let entry = self
            .records
            .entry(instance_id.to_string())
            .or_insert_with(|| InstanceCrashRecord {
                consecutive_crashes: 0,
                last_launched_at: now,
                last_crashed_at: now,
            });
        entry.last_launched_at = now;
    }

    /// Records process termination, incrementing crash count if lifespan was short.
    pub fn record_termination(&mut self, instance_id: &str) -> u32 {
        let now = Instant::now();
        let entry = self
            .records
            .entry(instance_id.to_string())
            .or_insert_with(|| InstanceCrashRecord {
                consecutive_crashes: 0,
                last_launched_at: now,
                last_crashed_at: now,
            });

        let lifespan = now.duration_since(entry.last_launched_at);
        if lifespan < self.short_lifespan_threshold {
            entry.consecutive_crashes += 1;
        } else {
            // Healthy run resets consecutive crash counter
            entry.consecutive_crashes = 0;
        }

        entry.last_crashed_at = now;
        entry.consecutive_crashes
    }

    /// Calculates required backoff delay (2s, 4s, 8s, 16s...) before next launch.
    pub fn required_backoff_delay(&self, instance_id: &str) -> Duration {
        let entry = match self.records.get(instance_id) {
            Some(e) => e,
            None => return Duration::ZERO,
        };

        if entry.consecutive_crashes == 0 {
            return Duration::ZERO;
        }

        let backoff_secs = 2_u64.pow(entry.consecutive_crashes.min(5));
        let computed = Duration::from_secs(backoff_secs);
        computed.min(self.max_backoff)
    }

    /// Checks whether an instance is currently allowed to launch without waiting.
    pub fn can_launch_now(&self, instance_id: &str) -> bool {
        let entry = match self.records.get(instance_id) {
            Some(e) => e,
            None => return true,
        };

        let delay = self.required_backoff_delay(instance_id);
        entry.last_crashed_at.elapsed() >= delay
    }
}
