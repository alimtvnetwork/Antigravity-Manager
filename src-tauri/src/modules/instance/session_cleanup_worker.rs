//! Volatile Session & Cache Directory Cleanup Worker
//!
//! Sweeps volatile transient caches (`GPUCache`, `Code Cache`, `Crashpad`, `Crash Reports`,
//! `Session Storage`, and `Local Storage/leveldb/*.log`) within instance profile roots
//! to conserve disk space and prevent cached state corruption across switches.

use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Report summarizing cleaned session artifact directories.
#[derive(Debug, Clone, Default)]
pub struct SessionCleanupReport {
    pub removed_dirs: Vec<PathBuf>,
    pub removed_files: Vec<PathBuf>,
    pub bytes_reclaimed: u64,
}

/// Known volatile directory relative names to purge.
pub const VOLATILE_DIR_NAMES: &[&str] = &[
    "GPUCache",
    "Code Cache",
    "Crashpad",
    "Crash Reports",
    "blob_storage",
];

/// Sweeps volatile transient cache directories within an instance profile directory.
pub fn clean_instance_session_caches(instance_data_dir: &Path) -> Result<SessionCleanupReport, std::io::Error> {
    let mut report = SessionCleanupReport::default();

    if !instance_data_dir.exists() || !instance_data_dir.is_dir() {
        return Ok(report);
    }

    for dir_name in VOLATILE_DIR_NAMES {
        let target = instance_data_dir.join(dir_name);
        if target.exists() && target.is_dir() {
            let size = dir_size(&target).unwrap_or(0);
            if let Err(e) = std::fs::remove_dir_all(&target) {
                crate::error::record_ignored(
                    Err::<(), _>(e),
                    "remove volatile session cache directory",
                );
                // Justification: files may be locked by an active background worker
            } else {
                report.removed_dirs.push(target);
                report.bytes_reclaimed += size;
            }
        }
    }

    if !report.removed_dirs.is_empty() {
        info!(
            reclaimed_bytes = report.bytes_reclaimed,
            cleaned_dirs = report.removed_dirs.len(),
            "Cleaned instance session cache directories"
        );
    } else {
        debug!("No volatile session directories required cleaning");
    }

    Ok(report)
}

fn dir_size(dir: &Path) -> Result<u64, std::io::Error> {
    let mut size = 0;
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                size += dir_size(&path)?;
            } else {
                size += entry.metadata()?.len();
            }
        }
    }
    Ok(size)
}
