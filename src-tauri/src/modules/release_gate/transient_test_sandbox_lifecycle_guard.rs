//! Transient Test Sandbox Lifecycle Guard (`Imp-694`, Step 280).
//!
//! Enforces test isolation and cleanup per AGENTS.md:
//! "Caller IDE Self-Preservation & Transient Test Sandboxing: Integration tests
//! must always generate transient, isolated test sandboxes (`agm instances test-create`)
//! and clean them up completely (`agm instances test-clean`), leaving zero orphan
//! processes or filesystem remnants."

use serde::{Deserialize, Serialize};
use std::fmt;

/// Standard prefix for transient test sandbox instance IDs.
pub const TRANSIENT_SANDBOX_PREFIX: &str = "agm-test-sandbox-";

/// Lifecycle state of a transient test sandbox.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SandboxStatus {
    Created,
    Active,
    Cleaned,
    LeakedRemnants,
}

impl SandboxStatus {
    /// Returns the string representation of the lifecycle state.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "Created",
            Self::Active => "Active",
            Self::Cleaned => "Cleaned",
            Self::LeakedRemnants => "LeakedRemnants",
        }
    }

    /// Returns true if the sandbox is actively running tests with tracked processes.
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }

    /// Returns true if the sandbox successfully completed zero-orphan cleanup.
    pub fn is_cleaned(&self) -> bool {
        matches!(self, Self::Cleaned)
    }

    /// Returns true if the sandbox leaked orphaned processes or filesystem remnants.
    pub fn is_leaked(&self) -> bool {
        matches!(self, Self::LeakedRemnants)
    }
}

impl fmt::Display for SandboxStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Metadata record tracking an allocated transient test sandbox environment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestSandboxRecord {
    pub sandbox_id: String,
    pub base_dir: String,
    pub created_epoch_ms: u64,
    pub spawned_pids: Vec<u32>,
    pub status: SandboxStatus,
}

impl TestSandboxRecord {
    /// Creates a new test sandbox record in the `Created` state.
    pub fn new(
        sandbox_id: impl Into<String>,
        base_dir: impl Into<String>,
        created_epoch_ms: u64,
    ) -> Self {
        Self {
            sandbox_id: sandbox_id.into(),
            base_dir: base_dir.into(),
            created_epoch_ms,
            spawned_pids: Vec::new(),
            status: SandboxStatus::Created,
        }
    }

    /// Associates a spawned process PID with this sandbox and advances state to `Active`.
    pub fn track_child_process(&mut self, pid: u32) {
        if !self.spawned_pids.contains(&pid) {
            self.spawned_pids.push(pid);
        }
        if self.status == SandboxStatus::Created {
            self.status = SandboxStatus::Active;
        }
    }

    /// Transitions the sandbox status to `Cleaned`.
    pub fn mark_cleaned(&mut self) {
        self.status = SandboxStatus::Cleaned;
    }

    /// Transitions the sandbox status to `LeakedRemnants`.
    pub fn mark_leaked(&mut self) {
        self.status = SandboxStatus::LeakedRemnants;
    }
}

/// Detailed post-execution cleanliness audit report.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SandboxCleanupReport {
    pub sandbox_id: String,
    pub is_clean: bool,
    pub killed_processes_count: usize,
    pub deleted_files_count: usize,
    pub surviving_pids: Vec<u32>,
    pub surviving_paths: Vec<String>,
    pub violations: Vec<String>,
}

impl SandboxCleanupReport {
    /// Human-readable summary of the cleanup audit.
    pub fn summary(&self) -> String {
        if self.is_clean {
            format!(
                "Transient sandbox '{}' cleanly reclaimed ({} processes terminated, 0 surviving remnants).",
                self.sandbox_id, self.killed_processes_count
            )
        } else {
            format!(
                "Transient sandbox '{}' CLEANUP LEAK: {} surviving PIDs, {} surviving paths. Violations: {}",
                self.sandbox_id,
                self.surviving_pids.len(),
                self.surviving_paths.len(),
                self.violations.join("; ")
            )
        }
    }
}

impl fmt::Display for SandboxCleanupReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.summary())
    }
}

/// Registers a new transient test sandbox record.
pub fn register_sandbox(
    sandbox_id: &str,
    base_dir: &str,
    created_epoch_ms: u64,
) -> TestSandboxRecord {
    TestSandboxRecord::new(sandbox_id, base_dir, created_epoch_ms)
}

/// Tracks a child process PID spawned under the given test sandbox.
pub fn track_child_process(sandbox: &mut TestSandboxRecord, pid: u32) {
    sandbox.track_child_process(pid);
}

/// Normalizes path string for cross-platform comparison.
fn normalize_path_str(p: &str) -> String {
    p.trim()
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_lowercase()
}

/// Checks if a candidate path resides inside or equals the sandbox base directory.
fn is_path_inside_or_equal(candidate: &str, base: &str, sandbox_id: &str) -> bool {
    if !sandbox_id.is_empty() && candidate.contains(&sandbox_id.to_lowercase()) {
        return true;
    }
    if base.is_empty() {
        return false;
    }
    candidate == base || candidate.starts_with(&format!("{}/", base))
}

/// Audits whether a test sandbox has been completely torn down without remnants.
///
/// Verifies:
/// 1. Zero orphan child processes: none of the sandbox's `spawned_pids` appear in `active_os_pids`.
/// 2. Zero filesystem remnants: none of `existing_paths` reside in or match `sandbox.base_dir`.
pub fn audit_sandbox_cleanliness(
    sandbox: &TestSandboxRecord,
    active_os_pids: &[u32],
    existing_paths: &[&str],
) -> SandboxCleanupReport {
    let mut surviving_pids = Vec::new();
    for pid in &sandbox.spawned_pids {
        if active_os_pids.contains(pid) && !surviving_pids.contains(pid) {
            surviving_pids.push(*pid);
        }
    }

    let killed_processes_count = sandbox
        .spawned_pids
        .len()
        .saturating_sub(surviving_pids.len());

    let normalized_base = normalize_path_str(&sandbox.base_dir);

    let mut surviving_paths = Vec::new();
    for path in existing_paths {
        let trimmed = path.trim();
        if trimmed.is_empty() {
            continue;
        }
        let normalized = normalize_path_str(trimmed);
        if is_path_inside_or_equal(&normalized, &normalized_base, &sandbox.sandbox_id) {
            if !surviving_paths.contains(&trimmed.to_string()) {
                surviving_paths.push(trimmed.to_string());
            }
        }
    }

    let mut violations = Vec::new();
    if !surviving_pids.is_empty() {
        violations.push(format!(
            "Orphan child processes still active: {:?}",
            surviving_pids
        ));
    }
    if !surviving_paths.is_empty() {
        violations.push(format!(
            "Filesystem remnants still present in '{}': {:?}",
            sandbox.base_dir, surviving_paths
        ));
    }

    let is_clean = surviving_pids.is_empty() && surviving_paths.is_empty();

    SandboxCleanupReport {
        sandbox_id: sandbox.sandbox_id.clone(),
        is_clean,
        killed_processes_count,
        deleted_files_count: if is_clean && !existing_paths.is_empty() {
            existing_paths.len()
        } else {
            0
        },
        surviving_pids,
        surviving_paths,
        violations,
    }
}

/// Audits cleanliness and updates the sandbox record status accordingly.
pub fn audit_and_update_sandbox(
    sandbox: &mut TestSandboxRecord,
    active_os_pids: &[u32],
    existing_paths: &[&str],
) -> SandboxCleanupReport {
    let report = audit_sandbox_cleanliness(sandbox, active_os_pids, existing_paths);
    if report.is_clean {
        sandbox.mark_cleaned();
    } else {
        sandbox.mark_leaked();
    }
    report
}
