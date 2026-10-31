//! Stale Profile Lease Reclaim Watchdog
//!
//! Sweeps active profile leases held by IDE instances and detects orphaned leases
//! where holding processes have terminated or the lease duration exceeds safety TTLs
//! (default 15 minutes), forcefully releasing them back to the candidate pool.

use std::time::{Duration, Instant};
use tracing::{info, warn};

/// Default maximum lease duration before considered stale (15 minutes).
pub const DEFAULT_MAX_LEASE_DURATION: Duration = Duration::from_secs(15 * 60);

/// Active lease snapshot for evaluation.
#[derive(Debug, Clone)]
pub struct ActiveLeaseSnapshot {
    pub lease_id: String,
    pub account_id: String,
    pub instance_id: String,
    pub acquired_at: Instant,
    pub last_heartbeat: Instant,
    pub holder_pid: Option<u32>,
}

/// Action to take on an evaluated lease.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeaseReclaimAction {
    Retain,
    ReclaimTimeout { age_secs: u64 },
    ReclaimDeadProcess { pid: u32 },
}

/// Evaluates an active lease snapshot and determines if it should be reclaimed.
pub fn evaluate_lease_reclaim(
    lease: &ActiveLeaseSnapshot,
    now: Instant,
    max_duration: Option<Duration>,
    is_pid_alive: impl Fn(u32) -> bool,
) -> LeaseReclaimAction {
    let limit = max_duration.unwrap_or(DEFAULT_MAX_LEASE_DURATION);
    let age = now.duration_since(lease.acquired_at);

    // Check age limit
    if age > limit {
        warn!(
            lease_id = %lease.lease_id,
            account_id = %lease.account_id,
            age_secs = age.as_secs(),
            "Profile lease exceeded maximum duration, recommending reclaim"
        );
        return LeaseReclaimAction::ReclaimTimeout {
            age_secs: age.as_secs(),
        };
    }

    // Check process liveness if PID is recorded
    if let Some(pid) = lease.holder_pid {
        if !is_pid_alive(pid) {
            info!(
                lease_id = %lease.lease_id,
                pid = pid,
                "Holding process terminated, reclaiming orphaned profile lease"
            );
            return LeaseReclaimAction::ReclaimDeadProcess { pid };
        }
    }

    LeaseReclaimAction::Retain
}
