//! Split Database File Size Watchdog
//! Monitors on-disk file sizes of split databases (main, wal, shm)
//! to detect explosive growth and trigger proactive retention pruning.

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbSizeReport {
    pub db_name: String,
    pub main_db_bytes: u64,
    pub wal_bytes: u64,
    pub total_bytes: u64,
    pub is_over_threshold: bool,
}

pub struct DbSizeWatchdog;

impl DbSizeWatchdog {
    pub const DEFAULT_MAX_DB_BYTES: u64 = 100 * 1024 * 1024; // 100 MiB

    /// Inspects the file size of a database and its associated WAL file.
    pub fn inspect_db_size(db_path: &Path, max_bytes: Option<u64>) -> DbSizeReport {
        let threshold = max_bytes.unwrap_or(Self::DEFAULT_MAX_DB_BYTES);
        let main_bytes = db_path.metadata().map(|m| m.len()).unwrap_or(0);

        let wal_path = db_path.with_extension("db-wal");
        let wal_bytes = wal_path.metadata().map(|m| m.len()).unwrap_or(0);

        let total_bytes = main_bytes + wal_bytes;
        let is_over_threshold = total_bytes >= threshold;

        let db_name = db_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown.db")
            .to_string();

        DbSizeReport {
            db_name,
            main_db_bytes: main_bytes,
            wal_bytes,
            total_bytes,
            is_over_threshold,
        }
    }
}
