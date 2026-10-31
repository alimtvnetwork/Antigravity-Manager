//! Split-DB Connection Pool Starvation & Checkout Watchdog (Imp-634).
//!
//! Under high concurrency across split SQLite databases (e.g. `active_prompts`,
//! `repo_prompts.db`), tracks active connection checkout durations across threads
//! and detects thread pool starvation before query timeout exhaustion.
//!
//! Complies with the AGENTS.md mandate: "Bound every wait with a timeout".

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents an active database connection lease checked out from the pool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionLease {
    pub lease_id: String,
    pub thread_id: String,
    pub acquired_at_epoch_ms: u64,
    pub query_preview: String,
    pub database_name: String,
}

impl ConnectionLease {
    /// Constructs a new active connection lease entry.
    pub fn new(
        lease_id: impl Into<String>,
        thread_id: impl Into<String>,
        acquired_at_epoch_ms: u64,
        query_preview: impl Into<String>,
        database_name: impl Into<String>,
    ) -> Self {
        Self {
            lease_id: lease_id.into(),
            thread_id: thread_id.into(),
            acquired_at_epoch_ms,
            query_preview: query_preview.into(),
            database_name: database_name.into(),
        }
    }
}

/// Starvation severity classifications for leased connection hold durations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum StarvationSeverity {
    Nominal,
    Warning,
    CriticalStarvation,
}

impl StarvationSeverity {
    /// Returns true if the checkout duration is within safe normal limits.
    #[must_use]
    pub fn is_nominal(&self) -> bool {
        matches!(self, Self::Nominal)
    }

    /// Returns true if the checkout duration exceeds the warning threshold.
    #[must_use]
    pub fn is_warning(&self) -> bool {
        matches!(self, Self::Warning)
    }

    /// Returns true if the checkout duration has escalated to critical starvation.
    #[must_use]
    pub fn is_critical(&self) -> bool {
        matches!(self, Self::CriticalStarvation)
    }
}

/// Diagnostic alert produced when a lease hold duration exceeds configured thresholds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StarvationAlert {
    pub lease_id: String,
    pub hold_duration_ms: u64,
    pub severity: StarvationSeverity,
    pub thread_id: String,
    pub query_preview: String,
    pub database_name: String,
}

impl StarvationAlert {
    /// Returns true if the alert severity is critical.
    #[must_use]
    pub fn is_critical(&self) -> bool {
        self.severity.is_critical()
    }

    /// Returns true if the alert severity is warning.
    #[must_use]
    pub fn is_warning(&self) -> bool {
        self.severity.is_warning()
    }
}

/// Policy thresholds governing connection pool starvation detection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StarvationWatchdogPolicy {
    pub warning_threshold_ms: u64,
    pub critical_threshold_ms: u64,
    pub max_active_checkouts: usize,
}

impl StarvationWatchdogPolicy {
    pub const DEFAULT_WARNING_THRESHOLD_MS: u64 = 3000;
    pub const DEFAULT_CRITICAL_THRESHOLD_MS: u64 = 10000;
    pub const DEFAULT_MAX_ACTIVE_CHECKOUTS: usize = 16;

    /// Creates a custom policy with explicit thresholds.
    pub fn new(
        warning_threshold_ms: u64,
        critical_threshold_ms: u64,
        max_active_checkouts: usize,
    ) -> Self {
        Self {
            warning_threshold_ms,
            critical_threshold_ms,
            max_active_checkouts,
        }
    }
}

impl Default for StarvationWatchdogPolicy {
    fn default() -> Self {
        Self {
            warning_threshold_ms: Self::DEFAULT_WARNING_THRESHOLD_MS,
            critical_threshold_ms: Self::DEFAULT_CRITICAL_THRESHOLD_MS,
            max_active_checkouts: Self::DEFAULT_MAX_ACTIVE_CHECKOUTS,
        }
    }
}

/// Watchdog that tracks active SQLite connection checkouts to prevent thread pool starvation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStarvationWatchdog {
    pub active_leases: HashMap<String, ConnectionLease>,
    pub policy: StarvationWatchdogPolicy,
    pub total_leases_tracked: u64,
}

impl Default for ConnectionStarvationWatchdog {
    fn default() -> Self {
        Self::new(StarvationWatchdogPolicy::default())
    }
}

impl ConnectionStarvationWatchdog {
    /// Instantiates a new watchdog with the provided policy.
    pub fn new(policy: StarvationWatchdogPolicy) -> Self {
        Self {
            active_leases: HashMap::new(),
            policy,
            total_leases_tracked: 0,
        }
    }

    /// Registers the acquisition of a connection lease.
    pub fn acquire_lease(&mut self, lease: ConnectionLease) {
        self.active_leases.insert(lease.lease_id.clone(), lease);
        self.total_leases_tracked = self.total_leases_tracked.saturating_add(1);
    }

    /// Releases an active lease by its ID. Returns true if the lease was found and removed.
    pub fn release_lease(&mut self, lease_id: &str) -> bool {
        self.active_leases.remove(lease_id).is_some()
    }

    /// Evaluates current active checkouts against configured policy thresholds.
    /// Emits `StarvationAlert` entries for leases in `Warning` or `CriticalStarvation` state.
    pub fn check_starvation(&self, now_ms: u64) -> Vec<StarvationAlert> {
        let mut alerts = Vec::new();

        for lease in self.active_leases.values() {
            let hold_duration_ms = now_ms.saturating_sub(lease.acquired_at_epoch_ms);
            let severity = self.compute_severity(hold_duration_ms);

            if !severity.is_nominal() {
                alerts.push(StarvationAlert {
                    lease_id: lease.lease_id.clone(),
                    hold_duration_ms,
                    severity,
                    thread_id: lease.thread_id.clone(),
                    query_preview: lease.query_preview.clone(),
                    database_name: lease.database_name.clone(),
                });
            }
        }

        alerts.sort_by(|a, b| {
            b.hold_duration_ms
                .cmp(&a.hold_duration_ms)
                .then_with(|| a.lease_id.cmp(&b.lease_id))
        });

        alerts
    }

    /// Checks if the connection pool has saturated its maximum allowed active checkouts.
    #[must_use]
    pub fn is_pool_exhausted(&self) -> bool {
        self.active_leases.len() >= self.policy.max_active_checkouts
    }

    /// Returns the number of currently active connection leases.
    #[must_use]
    pub fn get_active_lease_count(&self) -> usize {
        self.active_leases.len()
    }

    /// Positive boolean indicator returning true if there is at least one active lease.
    #[must_use]
    pub fn has_active_leases(&self) -> bool {
        !self.active_leases.is_empty()
    }

    /// Positive boolean indicator checking if a specific lease is currently registered.
    #[must_use]
    pub fn is_lease_active(&self, lease_id: &str) -> bool {
        self.active_leases.contains_key(lease_id)
    }

    /// Retrieves a reference to an active lease if present.
    #[must_use]
    pub fn get_lease(&self, lease_id: &str) -> Option<&ConnectionLease> {
        self.active_leases.get(lease_id)
    }

    /// Clears all tracked active leases.
    pub fn clear(&mut self) {
        self.active_leases.clear();
    }

    /// Computes the starvation severity based on hold duration.
    fn compute_severity(&self, hold_duration_ms: u64) -> StarvationSeverity {
        if hold_duration_ms >= self.policy.critical_threshold_ms {
            StarvationSeverity::CriticalStarvation
        } else if hold_duration_ms >= self.policy.warning_threshold_ms {
            StarvationSeverity::Warning
        } else {
            StarvationSeverity::Nominal
        }
    }
}
