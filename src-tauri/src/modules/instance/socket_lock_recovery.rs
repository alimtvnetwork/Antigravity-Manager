// Stale Socket Lockfile Recovery Sentinel (Imp-500)
// Detects and cleans abandoned socket lockfiles in instance data directories
// left behind by unclean shutdowns or crashed IDE processes.

use std::fs;
use std::path::Path;
use tracing::{debug, warn};

pub const COMMON_LOCK_FILENAMES: &[&str] = &[
    "antigravity.lock",
    "SingletonLock",
    "SingletonSocket",
    ".antigravity-session.lock",
];

pub struct SocketLockRecovery;

impl SocketLockRecovery {
    /// Inspects the target instance directory and removes abandoned lock files.
    /// Returns the list of cleaned lockfile paths.
    pub fn purge_stale_locks<P: AsRef<Path>>(data_dir: P) -> Vec<String> {
        let dir = data_dir.as_ref();
        let mut cleaned = Vec::new();

        if !dir.exists() || !dir.is_dir() {
            return cleaned;
        }

        for &lock_name in COMMON_LOCK_FILENAMES {
            let lock_path = dir.join(lock_name);
            if lock_path.exists() {
                match fs::remove_file(&lock_path) {
                    Ok(_) => {
                        debug!(path = %lock_path.display(), "Purged stale instance lockfile");
                        cleaned.push(lock_path.to_string_lossy().to_string());
                    }
                    Err(e) => {
                        warn!(
                            path = %lock_path.display(),
                            error = %e,
                            "Failed to remove instance lockfile (may still be held by running process)"
                        );
                    }
                }
            }
        }

        cleaned
    }
}
