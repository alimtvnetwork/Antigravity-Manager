//! Windows Job Object Breakaway Fallback Ladder.
//!
//! Enforces process spawning safety per AGENTS.md:
//! Never unconditionally specify `CREATE_BREAKAWAY_FROM_JOB` (`0x01000000`) without a fallback
//! ladder (`spawn_with_job_breakaway_fallback`). If running inside a restricted Windows Job Object,
//! `CreateProcessW` fails with `os error 5: Access is denied`.

use serde::{Deserialize, Serialize};

/// Win32 process creation flag: CREATE_BREAKAWAY_FROM_JOB (0x01000000)
pub const CREATE_BREAKAWAY_FROM_JOB: u32 = 0x01000000;

/// Win32 ERROR_ACCESS_DENIED code
pub const ERROR_ACCESS_DENIED: i32 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpawnCreationFlagsDecision {
    /// Initial attempt: spawn with CREATE_BREAKAWAY_FROM_JOB flag
    WithBreakaway(u32),
    /// Fallback attempt: step down ladder to standard spawn without breakaway flags
    StandardWithoutBreakaway,
    /// Terminal failure: cannot fall back further
    TerminalFailure,
}

/// Evaluates the multi-stage spawn strategy based on attempt index and previous OS error codes.
pub fn determine_spawn_strategy(
    attempt: usize,
    last_os_error_code: Option<i32>,
) -> SpawnCreationFlagsDecision {
    match attempt {
        0 => SpawnCreationFlagsDecision::WithBreakaway(CREATE_BREAKAWAY_FROM_JOB),
        1 => {
            if let Some(err) = last_os_error_code {
                if is_job_object_access_denied(err) {
                    // Restricted Job Object detected (e.g. CI runner or sandbox) - step down
                    SpawnCreationFlagsDecision::StandardWithoutBreakaway
                } else {
                    SpawnCreationFlagsDecision::TerminalFailure
                }
            } else {
                SpawnCreationFlagsDecision::StandardWithoutBreakaway
            }
        }
        _ => SpawnCreationFlagsDecision::TerminalFailure,
    }
}

/// Returns true if the OS error indicates Windows Job Object breakaway restriction.
pub fn is_job_object_access_denied(os_error_code: i32) -> bool {
    os_error_code == ERROR_ACCESS_DENIED
}
