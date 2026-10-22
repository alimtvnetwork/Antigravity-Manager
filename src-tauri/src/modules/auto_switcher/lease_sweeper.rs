//! Multi-Node Machine Lease Expiration Sweep.
//! Sweeps distributed workspace leases and marks expired leases (TTL > 3600s or heartbeat age > 300s) as reclaimable.

use serde::{Deserialize, Serialize};

/// Maximum lease TTL: 3600 seconds (1 hour).
pub const MAX_LEASE_TTL_SECS: i64 = 3600;
/// Heartbeat expiration window: 300 seconds (5 minutes).
pub const HEARTBEAT_EXPIRATION_SECS: i64 = 300;

/// Snapshot of a distributed workspace profile lease.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaseCandidate {
    pub lease_id: String,
    pub machine_id: String,
    pub account_id: String,
    pub created_at_epoch: i64,
    pub last_heartbeat_epoch: i64,
    pub is_active: bool,
}

/// Result of lease sweep identifying reclaimable entries.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LeaseSweepResult {
    pub expired_lease_ids: Vec<String>,
    pub active_lease_count: usize,
    pub swept_at_epoch: i64,
}

/// Evaluates if an individual lease has expired based on TTL or stale heartbeats.
pub fn is_lease_expired(lease: &LeaseCandidate, now_epoch: i64) -> bool {
    if !lease.is_active {
        return true;
    }

    // Check TTL from creation
    let ttl_elapsed = now_epoch - lease.created_at_epoch;
    if ttl_elapsed >= MAX_LEASE_TTL_SECS {
        return true;
    }

    // Check heartbeat freshness
    let heartbeat_age = now_epoch - lease.last_heartbeat_epoch;
    heartbeat_age >= HEARTBEAT_EXPIRATION_SECS
}

/// Sweeps an array of leases, collecting expired/orphaned leases that are safe to reclaim.
pub fn sweep_expired_leases(leases: &[LeaseCandidate], now_epoch: i64) -> LeaseSweepResult {
    let mut expired_lease_ids = Vec::new();
    let mut active_count = 0;

    for lease in leases {
        if is_lease_expired(lease, now_epoch) {
            expired_lease_ids.push(lease.lease_id.clone());
        } else {
            active_count += 1;
        }
    }

    LeaseSweepResult {
        expired_lease_ids,
        active_lease_count: active_count,
        swept_at_epoch: now_epoch,
    }
}
