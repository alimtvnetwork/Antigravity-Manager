// Consecutive Switch Velocity Throttle (Imp-488)
// Enforces a hard velocity barrier to prevent erratic flip-flopping
// across accounts within a compact time window (e.g. max 3 switches in 60s).

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct ThrottleConfig {
    pub max_switches_per_window: usize,
    pub window_duration: Duration,
    pub cooloff_penalty: Duration,
}

impl Default for ThrottleConfig {
    fn default() -> Self {
        Self {
            max_switches_per_window: 3,
            window_duration: Duration::from_secs(60),
            cooloff_penalty: Duration::from_secs(45),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConsecutiveSwitchThrottle {
    config: ThrottleConfig,
    history: Arc<Mutex<HashMap<String, VecDeque<Instant>>>>,
}

impl Default for ConsecutiveSwitchThrottle {
    fn default() -> Self {
        Self::new(ThrottleConfig::default())
    }
}

impl ConsecutiveSwitchThrottle {
    pub fn new(config: ThrottleConfig) -> Self {
        Self {
            config,
            history: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Checks if a switch is permitted for the given instance/profile key.
    /// Returns Ok(()) if permitted, or Err(wait_duration) if throttled.
    pub fn check_and_record(&self, key: &str, now: Instant) -> Result<(), Duration> {
        let mut map = self.history.lock().map_err(|_| Duration::from_secs(5))?;
        let timestamps = map.entry(key.to_string()).or_default();

        // Evict expired entries outside the sliding window
        while let Some(&front) = timestamps.front() {
            if now.duration_since(front) > self.config.window_duration {
                timestamps.pop_front();
            } else {
                break;
            }
        }

        if timestamps.len() >= self.config.max_switches_per_window {
            // Throttled: calculate required wait duration until oldest event leaves window
            if let Some(&oldest) = timestamps.front() {
                let elapsed = now.duration_since(oldest);
                let wait = self.config.window_duration.saturating_sub(elapsed) + self.config.cooloff_penalty;
                return Err(wait);
            }
            return Err(self.config.cooloff_penalty);
        }

        timestamps.push_back(now);
        Ok(())
    }

    /// Resets the switch history for a specific instance.
    pub fn reset(&self, key: &str) {
        if let Ok(mut map) = self.history.lock() {
            map.remove(key);
        }
    }
}
