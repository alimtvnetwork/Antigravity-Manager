//! Machine lease pre-release coordinator.
//! Coordinates graceful distributed lease hand-off before process exit,
//! system sleep, or profile switching to prevent stale cluster locks.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Mutex;
use std::time::Instant;

/// Status of a tracked machine lease during pre-release coordination.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeaseReleaseState {
    Active,
    Relinquishing,
    Released,
}

#[derive(Debug)]
struct ManagedLease {
    lease_id: String,
    instance_id: String,
    state: LeaseReleaseState,
    acquired_at: Instant,
}

pub struct LeasePrereleaseCoordinator {
    leases: Mutex<Vec<ManagedLease>>,
}

impl LeasePrereleaseCoordinator {
    pub fn new() -> Self {
        Self {
            leases: Mutex::new(Vec::new()),
        }
    }

    /// Registers a newly acquired lease under local management.
    pub fn register_lease(&self, lease_id: impl Into<String>, instance_id: impl Into<String>) {
        let mut guard = match self.leases.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };

        guard.push(ManagedLease {
            lease_id: lease_id.into(),
            instance_id: instance_id.into(),
            state: LeaseReleaseState::Active,
            acquired_at: Instant::now(),
        });
    }

    /// Marks all active leases as relinquishing prior to graceful shutdown.
    /// Returns the list of lease IDs that need to be released upstream.
    pub fn prepare_shutdown_release(&self) -> Vec<String> {
        let mut guard = match self.leases.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };

        let mut to_release = Vec::new();
        for item in guard.iter_mut() {
            if item.state == LeaseReleaseState::Active {
                item.state = LeaseReleaseState::Relinquishing;
                to_release.push(item.lease_id.clone());
            }
        }
        to_release
    }

    /// Confirms that a lease was successfully acknowledged as released upstream.
    pub fn confirm_released(&self, lease_id: &str) {
        let mut guard = match self.leases.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };

        if let Some(pos) = guard.iter().position(|l| l.lease_id == lease_id) {
            guard[pos].state = LeaseReleaseState::Released;
        }
    }

    /// Cleans up confirmed released leases.
    pub fn prune_released(&self) {
        let mut guard = match self.leases.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        guard.retain(|l| l.state != LeaseReleaseState::Released);
    }
}

impl Default for LeasePrereleaseCoordinator {
    fn default() -> Self {
        Self::new()
    }
}
