//! Transient Test Sandbox Lifecycle & Zero-Orphan Verifier.
//!
//! Enforces caller IDE self-preservation and test sandboxing per AGENTS.md:
//! Integration tests must always generate transient, isolated test sandboxes (`agm instances test-create`)
//! and clean them up completely (`agm instances test-clean`), leaving zero orphan processes or filesystem remnants.

use serde::{Deserialize, Serialize};

/// Lifecycle phases through which a transient test sandbox transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SandboxLifecyclePhase {
    Created,
    Active,
    CleanedUp,
    FailedCleanup,
}

impl SandboxLifecyclePhase {
    /// Returns true if the sandbox is currently in an active running state.
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }

    /// Returns true if the sandbox completed zero-orphan cleanup.
    pub fn is_cleaned_up(&self) -> bool {
        matches!(self, Self::CleanedUp)
    }
}

/// Violations identified during transient sandbox lifecycle auditing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SandboxRemnantsViolation {
    OrphanProcessesRemaining {
        sandbox_id: String,
        orphan_pids: Vec<u32>,
    },
    FilesystemRemnantsRemaining {
        sandbox_id: String,
        remaining_paths: Vec<String>,
    },
    CallerIdeProcessTargeted {
        caller_pid: u32,
        guidance: String,
    },
}

impl std::fmt::Display for SandboxRemnantsViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OrphanProcessesRemaining {
                sandbox_id,
                orphan_pids,
            } => {
                write!(
                    f,
                    "Sandbox '{}' has orphan child processes remaining: {:?}",
                    sandbox_id, orphan_pids
                )
            }
            Self::FilesystemRemnantsRemaining {
                sandbox_id,
                remaining_paths,
            } => {
                write!(
                    f,
                    "Sandbox '{}' has filesystem remnants remaining: {:?}",
                    sandbox_id, remaining_paths
                )
            }
            Self::CallerIdeProcessTargeted {
                caller_pid,
                guidance,
            } => {
                write!(
                    f,
                    "Caller IDE process {} targeted: {}",
                    caller_pid, guidance
                )
            }
        }
    }
}

impl std::error::Error for SandboxRemnantsViolation {}

/// State tracking metadata for an allocated transient test sandbox.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransientSandboxState {
    pub sandbox_id: String,
    pub phase: SandboxLifecyclePhase,
    pub transient_dir: String,
    pub tracked_child_pids: Vec<u32>,
    pub caller_pid: Option<u32>,
}

impl TransientSandboxState {
    /// Marks the sandbox active when processes or tests start running.
    pub fn mark_active(&mut self) {
        self.phase = SandboxLifecyclePhase::Active;
    }

    /// Appends a child process ID to tracked processes.
    pub fn track_child_pid(&mut self, pid: u32) {
        if pid != 0 && !self.tracked_child_pids.contains(&pid) {
            self.tracked_child_pids.push(pid);
        }
    }
}

/// Verification report detailing remnants audit outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxLifecycleAuditReport {
    pub is_clean: bool,
    pub sandbox_id: String,
    pub violations: Vec<SandboxRemnantsViolation>,
    pub zero_remnants_confirmed: bool,
}

/// Validates that the sandbox identifier contains only safe alphanumeric and hyphen/underscore characters.
fn is_valid_sandbox_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// Initializes a transient test sandbox environment record.
pub fn initialize_transient_sandbox(
    sandbox_id: &str,
    caller_pid: Option<u32>,
) -> Result<TransientSandboxState, String> {
    if !is_valid_sandbox_id(sandbox_id) {
        return Err(format!(
            "Invalid sandbox_id '{}': must match ^[a-zA-Z0-9_-]+$",
            sandbox_id
        ));
    }

    let transient_dir = format!("target/sandboxes/{}", sandbox_id);

    Ok(TransientSandboxState {
        sandbox_id: sandbox_id.to_string(),
        phase: SandboxLifecyclePhase::Created,
        transient_dir,
        tracked_child_pids: Vec::new(),
        caller_pid,
    })
}

/// Detects lingering child processes or caller IDE conflicts.
fn detect_process_violations(
    state: &TransientSandboxState,
    running_pids: &[u32],
    force: bool,
) -> Option<SandboxRemnantsViolation> {
    let orphan_pids: Vec<u32> = state
        .tracked_child_pids
        .iter()
        .copied()
        .filter(|pid| running_pids.contains(pid))
        .collect();

    if orphan_pids.is_empty() {
        return None;
    }

    let caller_is_running = state
        .caller_pid
        .map_or(false, |c_pid| running_pids.contains(&c_pid));

    if caller_is_running && !force {
        let caller_pid = state.caller_pid.unwrap_or(0);
        Some(SandboxRemnantsViolation::CallerIdeProcessTargeted {
            caller_pid,
            guidance: format!(
                "Caller IDE process {} targeted during sandbox cleanup; pass --force to override",
                caller_pid
            ),
        })
    } else {
        Some(SandboxRemnantsViolation::OrphanProcessesRemaining {
            sandbox_id: state.sandbox_id.clone(),
            orphan_pids,
        })
    }
}

/// Checks existing file paths for files remaining inside the transient directory.
fn detect_filesystem_violations(
    sandbox_id: &str,
    transient_dir: &str,
    existing_paths: &[String],
) -> Option<SandboxRemnantsViolation> {
    let norm_dir = transient_dir
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_string();

    let remaining_paths: Vec<String> = existing_paths
        .iter()
        .filter(|p| {
            let norm_p = p.replace('\\', "/");
            let trimmed_p = norm_p.trim_end_matches('/');
            trimmed_p == norm_dir || norm_p.starts_with(&format!("{}/", norm_dir))
        })
        .cloned()
        .collect();

    if remaining_paths.is_empty() {
        None
    } else {
        Some(SandboxRemnantsViolation::FilesystemRemnantsRemaining {
            sandbox_id: sandbox_id.to_string(),
            remaining_paths,
        })
    }
}

/// Verifies that test sandbox cleanup left zero orphan processes and zero filesystem remnants.
pub fn verify_sandbox_cleanup(
    state: &mut TransientSandboxState,
    running_pids: &[u32],
    existing_paths: &[String],
    force: bool,
) -> SandboxLifecycleAuditReport {
    let mut violations = Vec::new();

    if let Some(proc_violation) = detect_process_violations(state, running_pids, force) {
        violations.push(proc_violation);
    }

    if let Some(fs_violation) =
        detect_filesystem_violations(&state.sandbox_id, &state.transient_dir, existing_paths)
    {
        violations.push(fs_violation);
    }

    let is_clean = violations.is_empty();
    if is_clean {
        state.phase = SandboxLifecyclePhase::CleanedUp;
    } else {
        state.phase = SandboxLifecyclePhase::FailedCleanup;
    }

    SandboxLifecycleAuditReport {
        is_clean,
        sandbox_id: state.sandbox_id.clone(),
        violations,
        zero_remnants_confirmed: is_clean,
    }
}
