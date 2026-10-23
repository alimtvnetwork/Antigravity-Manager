//! Instance PID Reclaim Barrier & Liveness Verifier
//!
//! Validates whether an instance process has truly terminated before releasing saved PID locks,
//! preventing PID recycling collision and ensuring child processes are dead before profile release.

use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// Result of evaluating PID reclamation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PidReclaimStatus {
    /// Target PID is genuinely dead; safe to reclaim.
    TerminatedAndSafe,
    /// Target PID is still active in the process table.
    StillActive { pid: u32, elapsed_ms: u64 },
    /// Process failed to terminate within the timeout window.
    TimeoutStubborn { pid: u32 },
}

/// Evaluates whether a PID has terminated within a deadline, using an OS liveness probe.
pub fn verify_pid_reclaimed<F>(
    pid: u32,
    max_wait: Duration,
    mut is_pid_alive: F,
) -> PidReclaimStatus
where
    F: FnMut(u32) -> bool,
{
    let start = Instant::now();

    while start.elapsed() < max_wait {
        if !is_pid_alive(pid) {
            debug!(
                pid = pid,
                elapsed_ms = start.elapsed().as_millis(),
                "PID confirmed terminated"
            );
            return PidReclaimStatus::TerminatedAndSafe;
        }

        std::thread::sleep(Duration::from_millis(50));
    }

    if is_pid_alive(pid) {
        warn!(
            pid = pid,
            timeout_ms = max_wait.as_millis(),
            "PID stubborn, still active after wait limit"
        );
        PidReclaimStatus::TimeoutStubborn { pid }
    } else {
        PidReclaimStatus::TerminatedAndSafe
    }
}
