//! Stale WAL file cleanup sentinel.
//! Audits and recovers orphaned -wal and -shm files resulting from abnormal
//! process crashes, safely triggering truncate checkpoints to consolidate storage.

use crate::error::AppError;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalFileAudit {
    pub main_db_path: String,
    pub has_wal_file: bool,
    pub wal_size_bytes: u64,
    pub has_shm_file: bool,
    pub shm_size_bytes: u64,
    pub was_recovered: bool,
}

pub struct StaleWalCleanupSentinel;

impl StaleWalCleanupSentinel {
    /// Audits WAL and SHM files adjacent to the specified main database file.
    pub fn audit_and_recover(main_db: &Path) -> Result<WalFileAudit, AppError> {
        let wal_path = PathBuf::from(format!("{}-wal", main_db.display()));
        let shm_path = PathBuf::from(format!("{}-shm", main_db.display()));

        let has_wal = wal_path.is_file();
        let wal_bytes = if has_wal {
            std::fs::metadata(&wal_path).map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };

        let has_shm = shm_path.is_file();
        let shm_bytes = if has_shm {
            std::fs::metadata(&shm_path).map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };

        let mut was_recovered = false;

        // If WAL file is significantly large (>4MB) or orphaned, trigger a truncate checkpoint
        if has_wal && wal_bytes > 4 * 1024 * 1024 {
            if let Ok(conn) = Connection::open(main_db) {
                if conn
                    .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
                    .is_ok()
                {
                    was_recovered = true;
                }
            }
        }

        Ok(WalFileAudit {
            main_db_path: main_db.to_string_lossy().to_string(),
            has_wal_file: has_wal,
            wal_size_bytes: wal_bytes,
            has_shm_file: has_shm,
            shm_size_bytes: shm_bytes,
            was_recovered,
        })
    }
}
