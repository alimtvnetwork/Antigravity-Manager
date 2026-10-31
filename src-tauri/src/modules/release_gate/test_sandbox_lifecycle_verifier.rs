//! Transient Test Sandbox Lifecycle & Zero-Orphan Verifier.
//!
//! Enforces test isolation per AGENTS.md:
//! Integration tests must always generate transient, isolated test sandboxes
//! (`agm instances test-create`) and clean them up completely (`agm instances test-clean`),
//! leaving zero orphan processes or filesystem remnants.

use serde::{Deserialize, Serialize};

pub const TRANSIENT_SANDBOX_PREFIX: &str = "agm-test-sandbox-";

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SandboxCleanupReport {
    pub is_clean: bool,
    pub orphan_paths: Vec<String>,
    pub orphan_pids: Vec<u32>,
    pub summary: String,
}

/// Generates a standardized transient test sandbox instance ID.
pub fn generate_transient_sandbox_id(run_id: u64) -> String {
    format!("{}{}", TRANSIENT_SANDBOX_PREFIX, run_id)
}

/// Detects whether an instance ID belongs to a transient test sandbox.
pub fn is_transient_sandbox_id(instance_id: &str) -> bool {
    instance_id.trim().starts_with(TRANSIENT_SANDBOX_PREFIX)
}

/// Validates that test cleanup left zero remnant files or orphan processes.
pub fn validate_sandbox_cleanup(
    remnant_paths: &[&str],
    orphan_pids: &[u32],
) -> SandboxCleanupReport {
    let orphan_paths: Vec<String> = remnant_paths
        .iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let orphan_pids_vec = orphan_pids.to_vec();

    let is_clean = orphan_paths.is_empty() && orphan_pids_vec.is_empty();

    let summary = if is_clean {
        "Zero-orphan test cleanup verified: no remnant paths or orphan processes".to_string()
    } else {
        format!(
            "Cleanup violation: {} remnant paths, {} orphan PIDs detected",
            orphan_paths.len(),
            orphan_pids_vec.len()
        )
    };

    SandboxCleanupReport {
        is_clean,
        orphan_paths,
        orphan_pids: orphan_pids_vec,
        summary,
    }
}
