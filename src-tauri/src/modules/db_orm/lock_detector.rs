//! Database File Lock Contention Detector.
//! Non-blocking lock probe detecting contentious database locks before executing heavy transactions.

use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Result of lock contention probe on a database file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockContentionStatus {
    pub is_locked: bool,
    pub path: PathBuf,
    pub check_duration_ms: u64,
}

/// Probes whether a SQLite database file has an active exclusive or reserved lock without blocking.
pub fn detect_lock_contention(path: &Path) -> LockContentionStatus {
    let start = Instant::now();
    let mut is_locked = false;

    if path.exists() {
        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX;
        match Connection::open_with_flags(path, flags) {
            Ok(conn) => {
                // Set timeout to 50ms for instantaneous non-blocking probe
                let _ = conn.busy_timeout(std::time::Duration::from_millis(50));
                if conn.execute_batch("BEGIN IMMEDIATE; ROLLBACK;").is_err() {
                    is_locked = true;
                }
            }
            Err(_) => {
                is_locked = true;
            }
        }
    }

    LockContentionStatus {
        is_locked,
        path: path.to_path_buf(),
        check_duration_ms: start.elapsed().as_millis() as u64,
    }
}
