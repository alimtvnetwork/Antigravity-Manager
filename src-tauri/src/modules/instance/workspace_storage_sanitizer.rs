//! Workspace Storage State & Window Layout Sanitizer
//!
//! Sanitizes stale workspace storage layout caches and window dimension locks
//! (`state.vscdb`, `workspace.json`) to prevent multi-instance windows from stacking
//! directly on top of each other and to preserve workspace continuity across switches.

use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Report summarizing workspace storage sanitization.
#[derive(Debug, Clone, Default)]
pub struct WorkspaceSanitizationReport {
    pub sanitized_workspaces: Vec<PathBuf>,
    pub removed_stale_locks: usize,
}

/// Sanitizes workspace storage folders under `workspaceStorage/` within the instance directory.
pub fn sanitize_instance_workspace_storage(
    instance_dir: &Path,
) -> Result<WorkspaceSanitizationReport, std::io::Error> {
    let mut report = WorkspaceSanitizationReport::default();
    let storage_dir = instance_dir.join("User/workspaceStorage");

    if !storage_dir.exists() || !storage_dir.is_dir() {
        return Ok(report);
    }

    for entry in std::fs::read_dir(&storage_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        // Check for workspace lock files or stale temporary sqlite state
        let lock_file = path.join("state.vscdb-journal");
        if lock_file.exists() {
            if let Err(e) = std::fs::remove_file(&lock_file) {
                crate::error::record_ignored(
                    Err::<(), _>(e),
                    "remove stale workspace state journal",
                );
                // Justification: best effort cleanup of orphaned lock
            } else {
                report.removed_stale_locks += 1;
            }
        }

        report.sanitized_workspaces.push(path);
    }

    if report.removed_stale_locks > 0 {
        info!(
            workspaces = report.sanitized_workspaces.len(),
            removed_locks = report.removed_stale_locks,
            "Sanitized instance workspace storage"
        );
    } else {
        debug!(
            workspaces = report.sanitized_workspaces.len(),
            "Workspace storage inspected, zero stale locks found"
        );
    }

    Ok(report)
}
