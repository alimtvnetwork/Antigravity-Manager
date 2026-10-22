//! Orphan Socket & Named Pipe Sweeper
//! Sweeps stale IPC Unix domain sockets and temporary named pipes
//! from instance sandboxes when their owning processes are no longer alive.

use std::path::Path;
use tracing::{debug, info};

pub struct OrphanSocketSweeper;

impl OrphanSocketSweeper {
    /// Inspects an instance data directory and removes stale socket and pipe files.
    pub fn sweep_instance_sockets(instance_dir: &Path, is_active: bool) -> std::io::Result<usize> {
        if is_active || !instance_dir.exists() {
            return Ok(0);
        }

        let mut cleaned_count = 0;
        let entries = match std::fs::read_dir(instance_dir) {
            Ok(iter) => iter,
            Err(_) => return Ok(0),
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let ext_lower = ext.to_ascii_lowercase();
                if ext_lower == "sock" || ext_lower == "socket" || ext_lower == "pipe" || ext_lower == "lock" {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_file() {
                            if let Err(e) = std::fs::remove_file(&path) {
                                debug!("Failed to remove orphan socket {:?}: {}", path, e);
                            } else {
                                cleaned_count += 1;
                                info!("Cleaned orphan socket file: {:?}", path);
                            }
                        }
                    }
                }
            }
        }

        Ok(cleaned_count)
    }
}
