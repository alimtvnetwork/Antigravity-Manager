//! Transient Sandbox Lifecycle & Cleanup Sentinel (`Imp-713`, Step 299).
//!
//! Enforces caller IDE self-preservation and transient test sandboxing per AGENTS.md:
//! "Caller IDE Self-Preservation & Transient Test Sandboxing: Automated tests, CLI commands,
//! and auto-switcher daemons MUST detect the caller's active IDE process (`detect_caller_instance_id()`)
//! to prevent terminating the developer's working IDE session. Commands targeting the caller instance
//! (`agm instances stop`, `rm`, `switch`) must fail-fast with exit code 2 unless `--force` is explicitly provided.
//! Integration tests must always generate transient, isolated test sandboxes (`agm instances test-create`)
//! and clean them up completely (`agm instances test-clean`), leaving zero orphan processes or filesystem remnants."

use serde::{Deserialize, Serialize};
use std::fmt;

/// Non-negotiable process exit code when targeting the caller IDE session without `--force`.
pub const CALLER_IDE_FAIL_FAST_EXIT_CODE: i32 = 2;

/// Standard naming prefix identifying transient test sandbox instances.
pub const TRANSIENT_SANDBOX_PREFIX: &str = "agm-test-sandbox-";

/// Lifecycle phases of a transient test sandbox environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransientSandboxState {
    /// Newly allocated sandbox metadata descriptor.
    Created,
    /// Active sandbox with running test workloads or tracked child processes.
    Active,
    /// Teardown requested; awaiting process termination and filesystem purging.
    PendingCleanup,
    /// Completely torn down with zero orphan processes and zero filesystem remnants.
    Purged,
    /// Incomplete teardown; surviving PIDs or directories detected.
    OrphanRemnantsDetected,
}

impl TransientSandboxState {
    /// Returns the canonical label of this state.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "Created",
            Self::Active => "Active",
            Self::PendingCleanup => "PendingCleanup",
            Self::Purged => "Purged",
            Self::OrphanRemnantsDetected => "OrphanRemnantsDetected",
        }
    }

    /// Returns true if the sandbox is actively running test workloads.
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }

    /// Returns true if the sandbox has successfully and cleanly purged.
    pub fn is_purged(&self) -> bool {
        matches!(self, Self::Purged)
    }

    /// Returns true if the sandbox requires cleanup actions.
    pub fn requires_cleanup(&self) -> bool {
        matches!(
            self,
            Self::Created | Self::Active | Self::PendingCleanup | Self::OrphanRemnantsDetected
        )
    }
}

impl fmt::Display for TransientSandboxState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Metadata descriptor of a candidate sandbox instance evaluated for eviction or cleanup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxInstanceDescriptor {
    pub sandbox_id: String,
    pub instance_dir: String,
    pub created_epoch_ms: u64,
    pub is_caller_ide: bool,
}

impl SandboxInstanceDescriptor {
    /// Creates a new sandbox instance descriptor.
    pub fn new(
        sandbox_id: impl Into<String>,
        instance_dir: impl Into<String>,
        created_epoch_ms: u64,
        is_caller_ide: bool,
    ) -> Self {
        Self {
            sandbox_id: sandbox_id.into(),
            instance_dir: instance_dir.into(),
            created_epoch_ms,
            is_caller_ide,
        }
    }

    /// Convenience constructor for a caller IDE instance descriptor.
    pub fn caller_ide(sandbox_id: impl Into<String>, instance_dir: impl Into<String>) -> Self {
        Self {
            sandbox_id: sandbox_id.into(),
            instance_dir: instance_dir.into(),
            created_epoch_ms: 0,
            is_caller_ide: true,
        }
    }

    /// Convenience constructor for a transient test sandbox descriptor.
    pub fn transient(sandbox_id: impl Into<String>, instance_dir: impl Into<String>) -> Self {
        Self {
            sandbox_id: sandbox_id.into(),
            instance_dir: instance_dir.into(),
            created_epoch_ms: 0,
            is_caller_ide: false,
        }
    }

    /// Checks if this sandbox ID matches the standard transient test prefix.
    pub fn is_transient_sandbox_id(&self) -> bool {
        is_transient_sandbox_id(&self.sandbox_id)
    }
}

/// Eviction safety verdict determining whether an instance can be safely purged.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SandboxEvictionVerdict {
    /// Target is caller IDE session without `--force`; MUST fail-fast with exit code 2.
    TargetIsCallerIdeFailFast { exit_code: i32 },
    /// Instance is safe to terminate and delete.
    SafeToPurge,
    /// Instance was already cleaned; no action necessary.
    AlreadyCleaned,
}

impl SandboxEvictionVerdict {
    /// Returns true if purging can proceed safely.
    pub fn is_safe_to_purge(&self) -> bool {
        matches!(self, Self::SafeToPurge)
    }

    /// Returns true if caller IDE fail-fast condition is triggered.
    pub fn is_caller_ide_fail_fast(&self) -> bool {
        matches!(self, Self::TargetIsCallerIdeFailFast { .. })
    }

    /// Returns true if the target is already purged or cleaned.
    pub fn is_already_cleaned(&self) -> bool {
        matches!(self, Self::AlreadyCleaned)
    }

    /// Returns exit code if fail-fast is required.
    pub fn exit_code(&self) -> Option<i32> {
        match self {
            Self::TargetIsCallerIdeFailFast { exit_code } => Some(*exit_code),
            _ => None,
        }
    }
}

impl fmt::Display for SandboxEvictionVerdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TargetIsCallerIdeFailFast { exit_code } => {
                write!(
                    f,
                    "TargetIsCallerIdeFailFast(exit_code={}) - Caller IDE session protected",
                    exit_code
                )
            }
            Self::SafeToPurge => write!(f, "SafeToPurge"),
            Self::AlreadyCleaned => write!(f, "AlreadyCleaned"),
        }
    }
}

/// Comprehensive plan or execution report for transient sandbox cleanup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxCleanupReport {
    pub sandbox_id: String,
    pub directories_removed: Vec<String>,
    pub processes_terminated: Vec<u32>,
    pub verdict: SandboxEvictionVerdict,
    pub is_fully_purged: bool,
}

impl SandboxCleanupReport {
    /// Generates a human-readable summary of the cleanup report.
    pub fn summary(&self) -> String {
        if self.is_fully_purged {
            format!(
                "Sandbox '{}' cleanly purged: {} directories removed, {} processes terminated.",
                self.sandbox_id,
                self.directories_removed.len(),
                self.processes_terminated.len()
            )
        } else {
            format!(
                "Sandbox '{}' cleanup blocked: verdict={}",
                self.sandbox_id, self.verdict
            )
        }
    }
}

impl fmt::Display for SandboxCleanupReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.summary())
    }
}

/// Checks whether an instance identifier indicates a transient test sandbox.
pub fn is_transient_sandbox_id(id: &str) -> bool {
    let trimmed = id.trim();
    trimmed.starts_with(TRANSIENT_SANDBOX_PREFIX)
        || trimmed.starts_with("test-sandbox-")
        || trimmed.starts_with("transient-")
}

/// Generates a standardized transient sandbox identifier.
pub fn generate_transient_sandbox_id(suffix: &str) -> String {
    format!("{}{}", TRANSIENT_SANDBOX_PREFIX, suffix.trim())
}

/// Evaluates whether evicting or cleaning up a sandbox instance is safe.
///
/// Caller IDE Self-Preservation Invariant:
/// If the target is the active caller IDE (indicated by `descriptor.is_caller_ide`
/// or matching `caller_instance_id`), this function returns
/// `SandboxEvictionVerdict::TargetIsCallerIdeFailFast { exit_code: 2 }` unless
/// `force` is explicitly true.
pub fn evaluate_sandbox_eviction_safety(
    descriptor: &SandboxInstanceDescriptor,
    caller_instance_id: Option<&str>,
    force: bool,
) -> SandboxEvictionVerdict {
    // 1. Determine if target matches caller IDE
    let is_caller = descriptor.is_caller_ide
        || caller_instance_id
            .map(|caller_id| {
                let trimmed_caller = caller_id.trim();
                !trimmed_caller.is_empty()
                    && trimmed_caller.eq_ignore_ascii_case(descriptor.sandbox_id.trim())
            })
            .unwrap_or(false);

    if is_caller {
        if force {
            return SandboxEvictionVerdict::SafeToPurge;
        }
        return SandboxEvictionVerdict::TargetIsCallerIdeFailFast {
            exit_code: CALLER_IDE_FAIL_FAST_EXIT_CODE,
        };
    }

    // 2. Check if already cleaned
    let trimmed_dir = descriptor.instance_dir.trim();
    if trimmed_dir.is_empty()
        || trimmed_dir.eq_ignore_ascii_case("<cleaned>")
        || trimmed_dir.eq_ignore_ascii_case("<purged>")
    {
        return SandboxEvictionVerdict::AlreadyCleaned;
    }

    // 3. Otherwise safe to purge
    SandboxEvictionVerdict::SafeToPurge
}

/// Plans transient sandbox cleanup, verifying caller protection and building the report.
pub fn plan_transient_sandbox_cleanup(
    descriptor: &SandboxInstanceDescriptor,
) -> SandboxCleanupReport {
    plan_transient_sandbox_cleanup_with_options(descriptor, None, false, &[])
}

/// Plans transient sandbox cleanup with customizable caller detection, force override, and tracked PIDs.
pub fn plan_transient_sandbox_cleanup_with_options(
    descriptor: &SandboxInstanceDescriptor,
    caller_instance_id: Option<&str>,
    force: bool,
    tracked_pids: &[u32],
) -> SandboxCleanupReport {
    let verdict = evaluate_sandbox_eviction_safety(descriptor, caller_instance_id, force);

    match verdict {
        SandboxEvictionVerdict::TargetIsCallerIdeFailFast { .. } => SandboxCleanupReport {
            sandbox_id: descriptor.sandbox_id.clone(),
            directories_removed: Vec::new(),
            processes_terminated: Vec::new(),
            verdict,
            is_fully_purged: false,
        },
        SandboxEvictionVerdict::AlreadyCleaned => SandboxCleanupReport {
            sandbox_id: descriptor.sandbox_id.clone(),
            directories_removed: Vec::new(),
            processes_terminated: Vec::new(),
            verdict,
            is_fully_purged: true,
        },
        SandboxEvictionVerdict::SafeToPurge => {
            let mut dirs = Vec::new();
            let trimmed = descriptor.instance_dir.trim();
            if !trimmed.is_empty() {
                dirs.push(trimmed.to_string());
            }

            SandboxCleanupReport {
                sandbox_id: descriptor.sandbox_id.clone(),
                directories_removed: dirs,
                processes_terminated: tracked_pids.to_vec(),
                verdict,
                is_fully_purged: true,
            }
        }
    }
}
