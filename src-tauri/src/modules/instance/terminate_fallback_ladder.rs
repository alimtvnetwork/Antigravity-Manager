//! Conscious Process Terminate Fallback Ladder.
//! Implements multi-stage progressive termination ladder:
//! Stage 1: Graceful close request (WM_CLOSE / SIGINT) with settling delay
//! Stage 2: Forced termination signal (SIGTERM)
//! Stage 3: Immediate kernel kill (SIGKILL / TerminateProcess) as last resort.

use std::time::Duration;

/// Configuration for the termination fallback ladder.
#[derive(Debug, Clone, Copy)]
pub struct TerminateLadderConfig {
    /// Grace period before escalating from graceful close to forced termination.
    pub graceful_grace_period: Duration,
    /// Grace period before escalating to immediate kernel SIGKILL.
    pub forced_grace_period: Duration,
}

impl Default for TerminateLadderConfig {
    fn default() -> Self {
        Self {
            graceful_grace_period: Duration::from_millis(1500),
            forced_grace_period: Duration::from_millis(1000),
        }
    }
}

/// The stages of the process termination ladder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminationStage {
    Stage1GracefulClose,
    Stage2ForcedTerminate,
    Stage3ImmediateKill,
    AlreadyTerminated,
}

/// Execution plan descriptor for terminating an instance PID.
#[derive(Debug, Clone)]
pub struct TerminationPlan {
    pub pid: u32,
    pub instance_name: String,
    pub current_stage: TerminationStage,
    pub config: TerminateLadderConfig,
}

impl TerminationPlan {
    /// Creates a termination plan for a given PID.
    pub fn new(pid: u32, instance_name: &str, config: Option<TerminateLadderConfig>) -> Self {
        Self {
            pid,
            instance_name: instance_name.to_string(),
            current_stage: TerminationStage::Stage1GracefulClose,
            config: config.unwrap_or_default(),
        }
    }

    /// Escalates to the next termination stage if PID remains alive.
    pub fn escalate(&mut self) -> TerminationStage {
        self.current_stage = match self.current_stage {
            TerminationStage::Stage1GracefulClose => TerminationStage::Stage2ForcedTerminate,
            TerminationStage::Stage2ForcedTerminate => TerminationStage::Stage3ImmediateKill,
            TerminationStage::Stage3ImmediateKill => TerminationStage::Stage3ImmediateKill,
            TerminationStage::AlreadyTerminated => TerminationStage::AlreadyTerminated,
        };
        self.current_stage
    }
}
