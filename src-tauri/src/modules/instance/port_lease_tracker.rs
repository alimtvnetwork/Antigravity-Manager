//! Dynamic Port Lease Allocation Tracker
//!
//! Manages and tracks allocated TCP and CDP remote debugging ports across concurrent instance
//! profiles, ensuring ports are leased without collision and returned when instances terminate.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;
use tracing::debug;

/// Record of an allocated port lease.
#[derive(Debug, Clone)]
pub struct PortLease {
    pub port: u16,
    pub instance_id: String,
    pub allocated_at: Instant,
}

/// Thread-safe tracker for port allocations.
pub struct PortLeaseTracker {
    leases: Mutex<HashMap<u16, PortLease>>,
    port_range_start: u16,
    port_range_end: u16,
}

impl Default for PortLeaseTracker {
    fn default() -> Self {
        Self::new(9222, 9299)
    }
}

impl PortLeaseTracker {
    pub fn new(range_start: u16, range_end: u16) -> Self {
        Self {
            leases: Mutex::new(HashMap::new()),
            port_range_start: range_start,
            port_range_end: range_end,
        }
    }

    /// Allocates the next available port for the specified instance.
    pub fn allocate_port(&self, instance_id: &str) -> Option<u16> {
        let mut leases = match self.leases.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };

        // If instance already has a port allocated, return it
        for (port, lease) in leases.iter() {
            if lease.instance_id == instance_id {
                return Some(*port);
            }
        }

        // Search for an unallocated port in the range
        for port in self.port_range_start..=self.port_range_end {
            if !leases.contains_key(&port) {
                let lease = PortLease {
                    port,
                    instance_id: instance_id.to_string(),
                    allocated_at: Instant::now(),
                };
                leases.insert(port, lease);
                debug!(port = port, instance_id = instance_id, "Allocated instance port lease");
                return Some(port);
            }
        }

        None
    }

    /// Releases a port previously allocated to an instance.
    pub fn release_port_for_instance(&self, instance_id: &str) -> Option<u16> {
        let mut leases = match self.leases.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };

        let mut port_to_remove = None;
        for (port, lease) in leases.iter() {
            if lease.instance_id == instance_id {
                port_to_remove = Some(*port);
                break;
            }
        }

        if let Some(port) = port_to_remove {
            leases.remove(&port);
            debug!(port = port, instance_id = instance_id, "Released instance port lease");
            Some(port)
        } else {
            None
        }
    }

    /// Checks if a port is currently leased.
    pub fn is_port_leased(&self, port: u16) -> bool {
        let leases = match self.leases.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        leases.contains_key(&port)
    }
}
