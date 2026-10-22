//! Instance Runtime Crash Frequency Counter
//!
//! Tracks abnormal termination events per instance profile within a sliding 10-minute window,
//! identifying crash loops and calculating required backoff delays before permitting relaunch.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// Sliding window duration for counting crash events (10 minutes).
pub const CRASH_WINDOW_DURATION: Duration = Duration::from_secs(600);

/// Maximum crashes allowed in the sliding window before relaunch backoff is enforced.
pub const MAX_ACCEPTABLE_CRASHES: usize = 3;

/// Tracker for instance crashes.
pub struct RuntimeCrashCounter {
    crashes: Mutex<HashMap<String, Vec<Instant>>>,
}

impl Default for RuntimeCrashCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeCrashCounter {
    pub fn new() -> Self {
        Self {
            crashes: Mutex::new(HashMap::new()),
        }
    }

    /// Records an abnormal crash event for an instance. Returns the current crash count in the window.
    pub fn record_crash(&self, instance_id: &str) -> usize {
        let now = Instant::now();
        let mut map = match self.crashes.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };

        let list = map.entry(instance_id.to_string()).or_default();
        list.retain(|t| now.duration_since(*t) < CRASH_WINDOW_DURATION);
        list.push(now);

        let count = list.len();
        if count >= MAX_ACCEPTABLE_CRASHES {
            warn!(
                instance = instance_id,
                crash_count = count,
                "Instance has exceeded maximum crash threshold in sliding window"
            );
        } else {
            debug!(
                instance = instance_id,
                crash_count = count,
                "Recorded instance crash event"
            );
        }

        count
    }

    /// Calculates recommended relaunch backoff duration based on recent crash frequency.
    pub fn get_backoff_delay(&self, instance_id: &str) -> Duration {
        let now = Instant::now();
        let map = match self.crashes.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };

        if let Some(list) = map.get(instance_id) {
            let recent_count = list.iter().filter(|t| now.duration_since(**t) < CRASH_WINDOW_DURATION).count();
            match recent_count {
                0 | 1 => Duration::ZERO,
                2 => Duration::from_secs(2),
                3 => Duration::from_secs(5),
                _ => Duration::from_secs(15),
            }
        } else {
            Duration::ZERO
        }
    }

    /// Clears crash history for an instance upon clean run or manual reset.
    pub fn clear(&self, instance_id: &str) {
        let mut map = match self.crashes.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        map.remove(instance_id);
    }
}
