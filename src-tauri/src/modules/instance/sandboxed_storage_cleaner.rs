//! Sandboxed Storage Obsolete File Cleaner
//!
//! Scans instance directories for obsolete backup files (`*.bak`, `*.old`, `temp-snapshot-*`),
//! cleaning leftover artifacts to prevent profile storage bloat across repeated switches.

use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Report summarizing obsolete file cleanup.
#[derive(Debug, Clone, Default)]
pub struct StorageCleanerReport {
    pub removed_files: Vec<PathBuf>,
    pub bytes_reclaimed: u64,
}

/// Sweeps obsolete files within an instance directory.
pub fn sweep_obsolete_instance_files(instance_dir: &Path) -> Result<StorageCleanerReport, std::io::Error> {
    let mut report = StorageCleanerReport::default();

    if !instance_dir.exists() || !instance_dir.is_dir() {
        return Ok(report);
    }

    for entry in std::fs::read_dir(instance_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            let is_obsolete = name.ends_with(".bak")
                || name.ends_with(".old")
                || name.starts_with("temp-snapshot-")
                || name.ends_with(".tmp");

            if is_obsolete {
                if let Ok(meta) = path.metadata() {
                    let len = meta.len();
                    if let Err(e) = std::fs::remove_file(&path) {
                        crate::error::record_ignored(
                            Err::<(), _>(e),
                            "remove obsolete instance storage file",
                        );
                        // Justification: safe to ignore if temporary file locked
                    } else {
                        report.removed_files.push(path);
                        report.bytes_reclaimed += len;
                    }
                }
            }
        }
    }

    if !report.removed_files.is_empty() {
        info!(
            files_count = report.removed_files.len(),
            reclaimed_bytes = report.bytes_reclaimed,
            "Cleaned obsolete files from instance directory"
        );
    } else {
        debug!("No obsolete files found in instance directory");
    }

    Ok(report)
}
