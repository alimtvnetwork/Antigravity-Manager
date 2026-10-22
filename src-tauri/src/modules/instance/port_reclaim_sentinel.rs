//! Abandoned Debugging Port Reclaimer
//! Monitors active instance CDP remote debugging port allocations and automatically
//! reclaims reserved ports when an instance process exits or terminates ungracefully.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Default)]
pub struct PortReclaimSentinel {
    port_owners: Arc<RwLock<HashMap<String, u16>>>, // instance_id -> cdp_port
}

impl PortReclaimSentinel {
    pub fn new() -> Self {
        Self {
            port_owners: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Records that an instance has been assigned a specific CDP debugging port.
    pub fn register_port(&self, instance_id: &str, port: u16) {
        if let Ok(mut guard) = self.port_owners.write() {
            guard.insert(instance_id.to_string(), port);
        }
    }

    /// Reclaims the debugging port for an instance when it is confirmed offline.
    /// Returns the reclaimed port number if it was registered.
    pub fn reclaim_port(&self, instance_id: &str) -> Option<u16> {
        if let Ok(mut guard) = self.port_owners.write() {
            guard.remove(instance_id)
        } else {
            None
        }
    }

    /// Checks if a port is currently marked as owned by any active instance.
    pub fn is_port_claimed(&self, port: u16) -> bool {
        if let Ok(guard) = self.port_owners.read() {
            guard.values().any(|&p| p == port)
        } else {
            false
        }
    }
}
