// Remote Debugging Port Lease Guard (Imp-533)
// Reserves, leases, and reclaims CDP remote debugging ports within the 9222–9299 range,
// ensuring no two concurrent instances share a port.

use std::collections::HashMap;
use std::net::{SocketAddr, TcpListener};
use std::sync::{Arc, Mutex};

pub const CDP_PORT_START: u16 = 9222;
pub const CDP_PORT_END: u16 = 9299;

#[derive(Debug, Clone)]
pub struct RemotePortGuard {
    leases: Arc<Mutex<HashMap<String, u16>>>,
}

impl Default for RemotePortGuard {
    fn default() -> Self {
        Self {
            leases: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl RemotePortGuard {
    /// Attempts to allocate an unused port in range for the given instance.
    pub fn allocate_port(&self, instance_id: &str) -> Option<u16> {
        let mut map = self.leases.lock().ok()?;

        if let Some(&existing) = map.get(instance_id) {
            return Some(existing);
        }

        let allocated_ports: Vec<u16> = map.values().copied().collect();

        for candidate in CDP_PORT_START..=CDP_PORT_END {
            if !allocated_ports.contains(&candidate) && Self::can_bind_port(candidate) {
                map.insert(instance_id.to_string(), candidate);
                return Some(candidate);
            }
        }

        None
    }

    /// Releases the port assigned to an instance.
    pub fn release_port(&self, instance_id: &str) {
        if let Ok(mut map) = self.leases.lock() {
            map.remove(instance_id);
        }
    }

    fn can_bind_port(port: u16) -> bool {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        TcpListener::bind(addr).is_ok()
    }
}
