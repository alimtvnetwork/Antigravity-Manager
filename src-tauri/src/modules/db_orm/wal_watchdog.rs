//! SQLite WAL File Size Watchdog.
//! Monitors `<dbname>.db-wal` byte size; triggers auto-checkpoint (`PRAGMA wal_checkpoint(TRUNCATE)`) if size exceeds threshold.

use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Default WAL checkpoint threshold: 16 MB.
pub const DEFAULT_MAX_WAL_BYTES: u64 = 16 * 1024 * 1024;

/// Status and outcome of WAL size check and checkpoint execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalStatus {
    pub wal_path: PathBuf,
    pub size_bytes: u64,
    pub checkpoint_triggered: bool,
    pub busy: i32,
    pub log: i32,
    pub checkpointed: i32,
}

/// Checks the WAL file size for the specified database file and issues a TRUNCATE checkpoint if exceeded.
pub fn check_and_checkpoint_wal(
    conn: &Connection,
    db_path: &Path,
    max_wal_bytes: Option<u64>,
) -> Result<WalStatus> {
    let wal_path = PathBuf::from(format!("{}-wal", db_path.to_string_lossy()));
    let threshold = max_wal_bytes.unwrap_or(DEFAULT_MAX_WAL_BYTES);

    let size_bytes = if wal_path.exists() {
        std::fs::metadata(&wal_path).map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };

    let mut checkpoint_triggered = false;
    let mut busy = 0;
    let mut log = 0;
    let mut checkpointed = 0;

    if size_bytes >= threshold {
        checkpoint_triggered = true;
        // PRAGMA wal_checkpoint(TRUNCATE) returns 3 integers: (busy, log, checkpointed)
        conn.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |row| {
            busy = row.get(0)?;
            log = row.get(1)?;
            checkpointed = row.get(2)?;
            Ok(())
        })?;
    }

    Ok(WalStatus {
        wal_path,
        size_bytes,
        checkpoint_triggered,
        busy,
        log,
        checkpointed,
    })
}
