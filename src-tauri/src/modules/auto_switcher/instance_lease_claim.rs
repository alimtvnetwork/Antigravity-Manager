//! Multi-Instance Lease Conflict Claim Coordinator
//!
//! Coordinates atomic lease claims on account profiles across multi-instance IDE sandboxes,
//! enforcing single-owner invariants to guarantee that two separate instances never bind to
//! the same account credentials simultaneously.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;
use tracing::{debug, warn};

/// Status of an attempted account lease claim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeaseClaimResult {
    ClaimGranted,
    AlreadyClaimedBySelf,
    ConflictOccupiedByOther { holder_instance_id: String },
}

/// In-memory tracker for active account lease claims.
pub struct InstanceLeaseClaimCoordinator {
    account_to_instance: Mutex<HashMap<String, (String, Instant)>>,
}

impl Default for InstanceLeaseClaimCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl InstanceLeaseClaimCoordinator {
    pub fn new() -> Self {
        Self {
            account_to_instance: Mutex::new(HashMap::new()),
        }
    }

    /// Attempts to claim an account for an instance.
    pub fn try_claim(&self, account_id: &str, instance_id: &str) -> LeaseClaimResult {
        let mut map = match self.account_to_instance.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };

        if let Some((holder, _)) = map.get(account_id) {
            if holder == instance_id {
                return LeaseClaimResult::AlreadyClaimedBySelf;
            } else {
                warn!(
                    account = account_id,
                    requester = instance_id,
                    current_holder = holder,
                    "Account lease claim conflict: already held by another instance"
                );
                return LeaseClaimResult::ConflictOccupiedByOther {
                    holder_instance_id: holder.clone(),
                };
            }
        }

        map.insert(account_id.to_string(), (instance_id.to_string(), Instant::now()));
        debug!(account = account_id, instance = instance_id, "Granted account lease claim");
        LeaseClaimResult::ClaimGranted
    }

    /// Releases a claim when an instance switches away or terminates.
    pub fn release_claim(&self, account_id: &str, instance_id: &str) -> bool {
        let mut map = match self.account_to_instance.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };

        if let Some((holder, _)) = map.get(account_id) {
            if holder == instance_id {
                map.remove(account_id);
                debug!(account = account_id, instance = instance_id, "Released account lease claim");
                return true;
            }
        }

        false
    }

    /// Releases all claims held by a specific instance.
    pub fn release_all_for_instance(&self, instance_id: &str) -> usize {
        let mut map = match self.account_to_instance.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };

        let before = map.len();
        map.retain(|_, (holder, _)| holder != instance_id);
        let released = before - map.len();

        if released > 0 {
            debug!(instance = instance_id, count = released, "Released all instance leases");
        }

        released
    }
}
