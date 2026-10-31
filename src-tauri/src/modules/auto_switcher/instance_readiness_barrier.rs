// Sibling Instance Switch Readiness Barrier (Imp-522)
// Coordinates mutual exclusivity across running instances during account switching,
// preventing concurrent credential mutations or socket collisions between sibling profiles.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct InstanceReadinessBarrier {
    active_switches: Arc<Mutex<HashSet<String>>>,
    switch_timeout: Duration,
}

impl Default for InstanceReadinessBarrier {
    fn default() -> Self {
        Self::new(Duration::from_secs(45))
    }
}

impl InstanceReadinessBarrier {
    pub fn new(switch_timeout: Duration) -> Self {
        Self {
            active_switches: Arc::new(Mutex::new(HashSet::new())),
            switch_timeout,
        }
    }

    /// Tries to enter a switch lock for the given instance ID.
    /// Returns an RAII guard on success, or None if already in progress.
    pub fn try_acquire_switch(&self, instance_id: &str) -> Option<SwitchLockGuard> {
        let mut lock = self.active_switches.lock().ok()?;
        if lock.contains(instance_id) {
            return None;
        }

        lock.insert(instance_id.to_string());
        Some(SwitchLockGuard {
            instance_id: instance_id.to_string(),
            registry: Arc::clone(&self.active_switches),
        })
    }

    pub fn is_instance_switching(&self, instance_id: &str) -> bool {
        self.active_switches
            .lock()
            .map(|set| set.contains(instance_id))
            .unwrap_or(false)
    }

    pub fn timeout(&self) -> Duration {
        self.switch_timeout
    }
}

pub struct SwitchLockGuard {
    instance_id: String,
    registry: Arc<Mutex<HashSet<String>>>,
}

impl Drop for SwitchLockGuard {
    fn drop(&mut self) {
        if let Ok(mut lock) = self.registry.lock() {
            lock.remove(&self.instance_id);
        }
    }
}
