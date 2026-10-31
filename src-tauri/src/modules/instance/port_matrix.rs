//! Multi-Profile Port Collision Matrix.
//! In-memory allocation matrix preventing CDP port collisions across concurrent instance profiles.

use std::collections::HashMap;
use std::sync::Mutex;

/// Manages active CDP port reservations per instance.
pub struct PortMatrix {
    assignments: Mutex<HashMap<String, u16>>,
}

impl PortMatrix {
    pub fn new() -> Self {
        Self {
            assignments: Mutex::new(HashMap::new()),
        }
    }

    /// Claims a port for an instance. Returns false if the port is already allocated to another instance.
    pub fn claim_port(&self, instance_id: &str, port: u16) -> bool {
        if let Ok(mut map) = self.assignments.lock() {
            // Check if another instance holds this port
            for (id, &p) in map.iter() {
                if p == port && id != instance_id {
                    return false;
                }
            }
            map.insert(instance_id.to_string(), port);
            true
        } else {
            false
        }
    }

    /// Releases any port allocated to the given instance.
    pub fn release_port(&self, instance_id: &str) {
        if let Ok(mut map) = self.assignments.lock() {
            map.remove(instance_id);
        }
    }

    /// Returns the assigned port for an instance, if any.
    pub fn get_port(&self, instance_id: &str) -> Option<u16> {
        self.assignments.lock().ok()?.get(instance_id).copied()
    }

    /// Checks if a port is currently held by any instance.
    pub fn is_port_claimed(&self, port: u16) -> bool {
        if let Ok(map) = self.assignments.lock() {
            map.values().any(|&p| p == port)
        } else {
            false
        }
    }
}

impl Default for PortMatrix {
    fn default() -> Self {
        Self::new()
    }
}
