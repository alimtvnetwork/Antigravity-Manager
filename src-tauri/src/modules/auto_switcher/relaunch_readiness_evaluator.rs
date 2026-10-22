//! Instance relaunch readiness evaluator.
//! Validates that a newly switched IDE instance has completely finished
//! process startup, window initialization, and the mandatory 10-second settling window.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

pub const MANDATORY_SETTLING_DURATION_SECS: u64 = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelaunchReadinessResult {
    pub is_ready: bool,
    pub settling_elapsed_secs: f64,
    pub settling_remaining_secs: f64,
    pub pending_reasons: Vec<String>,
}

pub struct RelaunchReadinessEvaluator;

impl RelaunchReadinessEvaluator {
    /// Evaluates whether an instance is ready for queued prompt re-injection.
    pub fn evaluate(
        switch_completed_at: Instant,
        is_process_alive: bool,
        is_cdp_port_open: bool,
    ) -> RelaunchReadinessResult {
        let elapsed = switch_completed_at.elapsed();
        let settling_target = Duration::from_secs(MANDATORY_SETTLING_DURATION_SECS);

        let mut pending = Vec::new();

        if !is_process_alive {
            pending.push("Target IDE process PID is not alive".to_string());
        }

        if elapsed < settling_target {
            let rem = (settling_target - elapsed).as_secs_f64();
            pending.push(format!("Mandatory 10s settling delay in progress ({:.1}s remaining)", rem));
        }

        if !is_cdp_port_open {
            pending.push("CDP remote debugging port is not yet responding".to_string());
        }

        let is_ready = pending.is_empty();
        let elapsed_secs = elapsed.as_secs_f64();
        let rem_secs = if elapsed < settling_target {
            (settling_target - elapsed).as_secs_f64()
        } else {
            0.0
        };

        RelaunchReadinessResult {
            is_ready,
            settling_elapsed_secs: elapsed_secs,
            settling_remaining_secs: rem_secs,
            pending_reasons: pending,
        }
    }
}
