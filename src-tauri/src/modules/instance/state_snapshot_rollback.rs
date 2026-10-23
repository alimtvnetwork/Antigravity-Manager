//! Instance state snapshot and atomic rollback engine.
//! Takes transactional backups of instance configuration files (`settings.json`, `storage.json`)
//! before executing account switches, permitting instant rollback if launch fails.

use crate::error::AppError;
use std::path::{Path, PathBuf};

pub struct StateSnapshotRollback;

impl StateSnapshotRollback {
    /// Creates a timestamped backup copy of a configuration file.
    pub fn create_snapshot(file_path: &Path) -> Result<PathBuf, AppError> {
        if !file_path.is_file() {
            return Err(AppError::Config(format!(
                "Cannot snapshot non-existent file: {}",
                file_path.display()
            )));
        }

        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);

        let backup_path = PathBuf::from(format!("{}.bak.{}", file_path.display(), now_ms));
        std::fs::copy(file_path, &backup_path).map_err(AppError::Io)?;

        Ok(backup_path)
    }

    /// Restores a configuration file from its backup snapshot.
    pub fn rollback_snapshot(target_path: &Path, snapshot_path: &Path) -> Result<(), AppError> {
        if !snapshot_path.is_file() {
            return Err(AppError::Config(format!(
                "Snapshot file does not exist: {}",
                snapshot_path.display()
            )));
        }

        std::fs::copy(snapshot_path, target_path).map_err(AppError::Io)?;

        // Cleanup temporary snapshot
        let _ = std::fs::remove_file(snapshot_path);
        Ok(())
    }
}
