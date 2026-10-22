// Consecutive Switch Cooldown Penalty Ladder (Imp-518)
// Enforces a progressive cooldown penalty that multiplies wait duration
// whenever an instance triggers consecutive account switches in close succession.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct CooldownLadderConfig {
    pub base_penalty: Duration,
    pub max_penalty: Duration,
    pub penalty_multiplier: f64,
    pub reset_idle_duration: Duration,
}

impl Default for CooldownLadderConfig {
    fn default() -> Self {
        Self {
            base_penalty: Duration::from_secs(15),
            max_penalty: Duration::from_secs(300), // 5 min
            penalty_multiplier: 1.75,
            reset_idle_duration: Duration::from_secs(180),
        }
    }
}

#[derive(Debug, Clone)]
struct InstanceCooldownState {
    consecutive_count: u32,
    last_switch_at: Instant,
}

#[derive(Debug, Clone)]
pub struct ConsecutiveSwitchCooldown {
    config: CooldownLadderConfig,
    state: Arc<Mutex<HashMap<String, InstanceCooldownState>>>,
}

impl Default for ConsecutiveSwitchCooldown {
    fn default() -> Self {
        Self::new(CooldownLadderConfig::default())
    }
}

impl ConsecutiveSwitchCooldown {
    pub fn new(config: CooldownLadderConfig) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Records a switch event and returns the recommended cooldown penalty duration.
    pub fn record_and_get_penalty(&self, instance_id: &str, now: Instant) -> Duration {
        let mut map = match self.state.lock() {
            Ok(m) => m,
            Err(_) => return self.config.base_penalty,
        };

        let entry = map.entry(instance_id.to_string()).or_insert_with(|| InstanceCooldownState {
            consecutive_count: 0,
            last_switch_at: now,
        });

        if now.duration_since(entry.last_switch_at) > self.config.reset_idle_duration {
            entry.consecutive_count = 1;
        } else {
            entry.consecutive_count = entry.consecutive_count.saturating_add(1);
        }
        entry.last_switch_at = now;

        let multiplier = self.config.penalty_multiplier.powi((entry.consecutive_count.saturating_sub(1)) as i32);
        let penalty_millis = (self.config.base_penalty.as_millis() as f64 * multiplier) as u64;
        let clamped = Duration::from_millis(penalty_millis).min(self.config.max_penalty);

        clamped
    }

    /// Resets the consecutive switch counter for an instance.
    pub fn reset_instance(&self, instance_id: &str) {
        if let Ok(mut map) = self.state.lock() {
            map.remove(instance_id);
        }
    }
}
