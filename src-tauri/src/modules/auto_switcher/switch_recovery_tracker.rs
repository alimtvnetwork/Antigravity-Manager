//! Post-Switch Request Health Tracker
//!
//! Tracks post-switch prompt execution outcomes across the initial requests (default 3)
//! dispatched to a newly switched instance profile, verifying that no transport failures,
//! authentication errors, or immediate rate-limits occur before declaring switch success.

use std::collections::HashMap;
use std::sync::Mutex;
use tracing::{debug, info, warn};

/// Target number of consecutive clean requests to verify post-switch health.
pub const DEFAULT_RECOVERY_PROBING_COUNT: usize = 3;

/// Status of the recovery tracking phase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SwitchRecoveryStatus {
    /// In progress; currently completed N out of target clean requests.
    Verifying {
        successful_requests: usize,
        target: usize,
    },
    /// Successfully completed all required clean verification requests.
    Stabilized,
    /// Encountered failure during verification; immediate rollback recommended.
    Failed { reason: String },
}

/// Recovery state for an instance after an account switch.
#[derive(Debug, Clone)]
struct InstanceRecoveryState {
    account_id: String,
    successful_requests: usize,
    target: usize,
    has_failed: bool,
    failure_reason: Option<String>,
}

/// Tracker monitoring post-switch instance stabilization.
pub struct SwitchRecoveryTracker {
    states: Mutex<HashMap<String, InstanceRecoveryState>>,
}

impl Default for SwitchRecoveryTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl SwitchRecoveryTracker {
    pub fn new() -> Self {
        Self {
            states: Mutex::new(HashMap::new()),
        }
    }

    /// Begins monitoring an instance post-switch.
    pub fn start_tracking(
        &self,
        instance_id: String,
        account_id: String,
        target_count: Option<usize>,
    ) {
        let state = InstanceRecoveryState {
            account_id,
            successful_requests: 0,
            target: target_count.unwrap_or(DEFAULT_RECOVERY_PROBING_COUNT),
            has_failed: false,
            failure_reason: None,
        };

        let mut map = match self.states.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        map.insert(instance_id, state);
    }

    /// Records the outcome of a request on the switched instance.
    pub fn record_outcome(
        &self,
        instance_id: &str,
        is_success: bool,
        error_message: Option<&str>,
    ) -> SwitchRecoveryStatus {
        let mut map = match self.states.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };

        if let Some(state) = map.get_mut(instance_id) {
            if is_success {
                state.successful_requests += 1;
                if state.successful_requests >= state.target {
                    info!(
                        instance = instance_id,
                        account = %state.account_id,
                        "Instance switch fully stabilized after verification checks"
                    );
                    return SwitchRecoveryStatus::Stabilized;
                } else {
                    debug!(
                        instance = instance_id,
                        completed = state.successful_requests,
                        target = state.target,
                        "Post-switch recovery request succeeded"
                    );
                    return SwitchRecoveryStatus::Verifying {
                        successful_requests: state.successful_requests,
                        target: state.target,
                    };
                }
            } else {
                let reason = error_message.unwrap_or("Unknown error").to_string();
                warn!(
                    instance = instance_id,
                    account = %state.account_id,
                    reason = %reason,
                    "Post-switch recovery verification encountered failure"
                );
                state.has_failed = true;
                state.failure_reason = Some(reason.clone());
                return SwitchRecoveryStatus::Failed { reason };
            }
        }

        // If not actively tracked, assume stabilized
        SwitchRecoveryStatus::Stabilized
    }

    /// Clears tracking state for an instance.
    pub fn clear(&self, instance_id: &str) {
        let mut map = match self.states.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        map.remove(instance_id);
    }
}
