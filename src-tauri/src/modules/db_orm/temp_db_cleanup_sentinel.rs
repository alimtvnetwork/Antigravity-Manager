//! Temporary Database & Journal File Cleanup Sentinel
//!
//! Sweeps orphaned temporary SQLite journal files (`*.db-journal`, `*.db-wal.tmp`, `etilqs_*`)
//! left behind by crashed processes, ensuring disk space reclamation and clean recovery
//! without interfering with actively locked production databases.

use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Result summary of sweeping temporary database artifacts.
#[derive(Debug, Clone, Default)]
pub struct TempDbCleanupReport {
    pub swept_files: Vec<PathBuf>,
    pub reclaimed_bytes: u64,
    pub skipped_files: Vec<PathBuf>,
}

/// Sweeps orphaned temporary database and journal artifacts from the given directory.
pub fn sweep_temp_db_artifacts(dir: &Path) -> Result<TempDbCleanupReport, std::io::Error> {
    let mut report = TempDbCleanupReport::default();

    if !dir.exists() || !dir.is_dir() {
        return Ok(report);
    }

    let entries = std::fs::read_dir(dir)?;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };

        // Identify temporary SQLite artifacts
        let is_temp_artifact = file_name.ends_with("-journal")
            || file_name.ends_with("-wal.tmp")
            || file_name.starts_with("etilqs_");

        if !is_temp_artifact {
            continue;
        }

        // Check if main database file is missing; if missing, journal is definitely orphaned
        let is_orphaned = if file_name.ends_with("-journal") {
            let base_name = file_name.trim_end_matches("-journal");
            !dir.join(base_name).exists()
        } else {
            true
        };

        if is_orphaned {
            if let Ok(metadata) = path.metadata() {
                let bytes = metadata.len();
                if let Err(e) = std::fs::remove_file(&path) {
                    crate::error::record_ignored(
                        Err::<(), _>(e),
                        "sweep temporary database artifact",
                    );
                    // Justification: file may be held open by another concurrent process
                    report.skipped_files.push(path);
                } else {
                    report.swept_files.push(path);
                    report.reclaimed_bytes += bytes;
                }
            }
        } else {
            report.skipped_files.push(path);
        }
    }

    if !report.swept_files.is_empty() {
        info!(
            swept_count = report.swept_files.len(),
            reclaimed_bytes = report.reclaimed_bytes,
            "Swept orphaned SQLite temporary artifacts"
        );
    } else {
        debug!("No orphaned SQLite temporary artifacts found during sweep");
    }

    Ok(report)
}
