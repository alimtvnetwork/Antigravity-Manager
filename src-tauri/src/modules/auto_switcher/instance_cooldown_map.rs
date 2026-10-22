//! Per-Instance Account Cooldown Interval Map
//! Maintains isolated, thread-safe in-memory cooldown expiration timers
//! per instance profile, preventing cross-profile throttle contamination.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Default)]
pub struct InstanceCooldownMap {
    entries: Arc<RwLock<HashMap<(String, String), Instant>>>,
}

impl InstanceCooldownMap {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Checks if a specific account is in cooldown for a given instance.
    pub fn is_in_cooldown(&self, instance_id: &str, account_id: &str) -> bool {
        let key = (instance_id.to_string(), account_id.to_string());
        let guard = match self.entries.read() {
            Ok(g) => g,
            Err(_) => return false,
        };

        if let Some(&expires_at) = guard.get(&key) {
            Instant::now() < expires_at
        } else {
            false
        }
    }

    /// Sets a cooldown period for an account on a specific instance.
    pub fn set_cooldown(&self, instance_id: &str, account_id: &str, duration: Duration) {
        let key = (instance_id.to_string(), account_id.to_string());
        let expires_at = Instant::now() + duration;
        if let Ok(mut guard) = self.entries.write() {
            guard.insert(key, expires_at);
        }
    }

    /// Clears all cooldowns associated with a particular instance profile.
    pub fn clear_instance(&self, instance_id: &str) {
        if let Ok(mut guard) = self.entries.write() {
            guard.retain(|(inst, _), _| inst != instance_id);
        }
    }

    /// Sweeps all expired cooldown entries across all instances.
    pub fn sweep_expired(&self) {
        let now = Instant::now();
        if let Ok(mut guard) = self.entries.write() {
            guard.retain(|_, &mut expires_at| expires_at > now);
        }
    }
}
