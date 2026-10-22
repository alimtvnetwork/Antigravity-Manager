//! Account Switch Settling Barrier & State Prober
//!
//! Enforces the mandatory post-switch settling barrier (10-second settling invariant),
//! coordinating progressive non-blocking health checks before releasing the IDE instance
//! to resume prompt dispatch and workspace file observation.

use std::time::{Duration, Instant};
use tracing::{debug, info};

/// Default mandatory settling delay duration (10 seconds).
pub const MANDATORY_SETTLING_DELAY_SECS: u64 = 10;

/// State of an instance progressing through the settling barrier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettlingBarrierState {
    /// Settling timer is still active.
    Waiting { elapsed_secs: u64, remaining_secs: u64 },
    /// Settling period elapsed, probing runtime liveness.
    ProbingRuntime,
    /// Settling and probes completed successfully; instance is ready.
    SettledAndReady,
    /// Probe failed (e.g. process crashed during settling).
    Failed(String),
}

/// Barrier tracking settling progress for a specific instance account switch.
#[derive(Debug, Clone)]
pub struct SwitchSettlingBarrier {
    pub instance_id: String,
    pub target_account_id: String,
    pub started_at: Instant,
    pub duration: Duration,
}

impl SwitchSettlingBarrier {
    /// Initializes a new settling barrier with default 10-second delay.
    pub fn new(instance_id: String, target_account_id: String) -> Self {
        Self {
            instance_id,
            target_account_id,
            started_at: Instant::now(),
            duration: Duration::from_secs(MANDATORY_SETTLING_DELAY_SECS),
        }
    }

    /// Checks the current state of the barrier with optional process liveness probe.
    pub fn check_state<F>(&self, is_alive_probe: F) -> SettlingBarrierState
    where
        F: FnOnce() -> bool,
    {
        let now = Instant::now();
        let elapsed = now.duration_since(self.started_at);

        if elapsed < self.duration {
            let elapsed_secs = elapsed.as_secs();
            let remaining_secs = (self.duration - elapsed).as_secs();
            debug!(
                instance_id = %self.instance_id,
                remaining_secs = remaining_secs,
                "Instance is settling post-switch"
            );
            return SettlingBarrierState::Waiting {
                elapsed_secs,
                remaining_secs,
            };
        }

        // Settling time elapsed, check process liveness
        if !is_alive_probe() {
            return SettlingBarrierState::Failed(
                "Instance process is not responsive after settling period".to_string(),
            );
        }

        info!(
            instance_id = %self.instance_id,
            account_id = %self.target_account_id,
            "Instance switch settled successfully, ready for prompts"
        );
        SettlingBarrierState::SettledAndReady
    }
}
