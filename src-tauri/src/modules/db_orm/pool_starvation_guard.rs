//! SQLite connection pool starvation guard.
//! Monitors leased connection handles across split databases to detect and
//! log leaked connections that remain checked out past the timeout threshold.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarvationAlert {
    pub db_name: String,
    pub checkout_tag: String,
    pub duration_ms: u64,
}

#[derive(Debug)]
struct ActiveLease {
    db_name: String,
    tag: String,
    acquired_at: Instant,
}

pub struct PoolStarvationGuard {
    warn_threshold: Duration,
    leases: Mutex<HashMap<u64, ActiveLease>>,
    next_id: Mutex<u64>,
}

impl PoolStarvationGuard {
    pub fn new(warn_threshold: Option<Duration>) -> Self {
        Self {
            warn_threshold: warn_threshold.unwrap_or(Duration::from_secs(5)),
            leases: Mutex::new(HashMap::new()),
            next_id: Mutex::new(1),
        }
    }

    /// Registers a newly checked-out connection lease. Returns a lease ticket ID.
    pub fn record_checkout(&self, db_name: &str, tag: &str) -> u64 {
        let id = {
            let mut id_guard = match self.next_id.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            let cur = *id_guard;
            *id_guard = cur.wrapping_add(1);
            cur
        };

        let mut lease_guard = match self.leases.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };

        lease_guard.insert(
            id,
            ActiveLease {
                db_name: db_name.to_string(),
                tag: tag.to_string(),
                acquired_at: Instant::now(),
            },
        );

        id
    }

    /// Records that the connection lease has been safely returned to the pool.
    pub fn record_checkin(&self, ticket_id: u64) {
        let mut guard = match self.leases.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        guard.remove(&ticket_id);
    }

    /// Audits current active leases and returns alerts for any exceeding the threshold.
    pub fn audit_starvations(&self) -> Vec<StarvationAlert> {
        let guard = match self.leases.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };

        let now = Instant::now();
        let mut alerts = Vec::new();

        for lease in guard.values() {
            let elapsed = now.duration_since(lease.acquired_at);
            if elapsed > self.warn_threshold {
                alerts.push(StarvationAlert {
                    db_name: lease.db_name.clone(),
                    checkout_tag: lease.tag.clone(),
                    duration_ms: elapsed.as_millis() as u64,
                });
            }
        }

        alerts
    }
}

impl Default for PoolStarvationGuard {
    fn default() -> Self {
        Self::new(None)
    }
}
