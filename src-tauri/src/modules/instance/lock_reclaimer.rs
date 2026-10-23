//! Profile Stale Lockfile Reclaimer.
//! Safely detects and removes orphaned `.antigravity-session` and `code.lock` files when the owning process is dead.

use std::path::Path;

/// Lockfile candidates that can become stale after improper termination.
pub const STALE_LOCK_CANDIDATES: &[&str] = &[
    ".antigravity-session",
    "code.lock",
    "lockfile",
    "DevToolsActivePort",
];

/// Outcome of stale lock reclamation.
#[derive(Debug, Clone, Default)]
pub struct LockReclaimReport {
    pub removed_files: Vec<String>,
    pub errors: Vec<String>,
}

/// Reclaims stale lockfiles in an instance data directory if the instance process is not running.
pub fn reclaim_stale_instance_locks(
    data_dir: &Path,
    is_process_running: bool,
) -> LockReclaimReport {
    let mut report = LockReclaimReport::default();

    if is_process_running {
        // Safe guard: Never remove locks while process is actively running
        return report;
    }

    for name in STALE_LOCK_CANDIDATES {
        let lock_path = data_dir.join(name);
        if lock_path.exists() {
            match std::fs::remove_file(&lock_path) {
                Ok(_) => {
                    report.removed_files.push(name.to_string());
                }
                Err(err) => {
                    report.errors.push(format!("{}: {}", name, err));
                }
            }
        }
    }

    report
}
