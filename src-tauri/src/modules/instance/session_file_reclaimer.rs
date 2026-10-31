//! Singleton Session Lockfile Reclaimer
//! Cleans stale Chromium singleton locks and VS Code session markers
//! from instance profile directories prior to launch to prevent startup blockades.

use std::fs;
use std::path::Path;
use tracing::{debug, info};

pub struct SessionFileReclaimer;

impl SessionFileReclaimer {
    pub const STALE_SESSION_FILES: &'static [&'static str] = &[
        ".antigravity-session",
        "SingletonLock",
        "SingletonCookie",
        "SingletonSocket",
        "lockfile",
        "code.lock",
        "DevToolsActivePort",
    ];

    /// Reclaims stale session locks in an instance data directory.
    pub fn reclaim_stale_files(instance_data_dir: &Path) -> std::io::Result<usize> {
        if !instance_data_dir.exists() {
            return Ok(0);
        }

        let mut reclaimed = 0;
        for &rel_file in Self::STALE_SESSION_FILES {
            let target = instance_data_dir.join(rel_file);
            if target.exists() {
                // Strip read-only attributes on Windows if present
                #[cfg(target_os = "windows")]
                {
                    if let Ok(metadata) = fs::metadata(&target) {
                        let mut perms = metadata.permissions();
                        perms.set_readonly(false);
                        let _ = fs::set_permissions(&target, perms);
                    }
                }

                if let Err(e) = fs::remove_file(&target) {
                    debug!("Failed to remove stale session file {:?}: {}", target, e);
                } else {
                    reclaimed += 1;
                    info!("Reclaimed stale session file: {:?}", target);
                }
            }
        }

        Ok(reclaimed)
    }
}
