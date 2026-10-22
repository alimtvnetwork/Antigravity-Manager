//! WAL Log Recovery Checkpoint Sentinel.
//! Executes startup recovery checkpoints (`PRAGMA wal_checkpoint(TRUNCATE);`)
//! across split database files, flushing uncommitted WAL frames and recovering from unclean terminations.

use rusqlite::Connection;

/// Summary metrics of a WAL recovery operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WalRecoveryResult {
    pub busy: i32,
    pub log_frames: i32,
    pub checkpointed_frames: i32,
}

/// Executes a WAL checkpoint with the specified mode.
pub fn execute_wal_recovery(
    conn: &Connection,
    mode: &str,
) -> Result<WalRecoveryResult, rusqlite::Error> {
    let sql = format!("PRAGMA wal_checkpoint({});", mode);
    let mut stmt = conn.prepare(&sql)?;

    let result = stmt.query_row([], |row| {
        Ok(WalRecoveryResult {
            busy: row.get(0)?,
            log_frames: row.get(1)?,
            checkpointed_frames: row.get(2)?,
        })
    })?;

    Ok(result)
}

/// Runs a safe two-stage recovery: first PASSIVE, then TRUNCATE.
pub fn recover_interrupted_wal(conn: &Connection) -> Result<WalRecoveryResult, rusqlite::Error> {
    // Stage 1: Non-blocking passive flush
    let _ = execute_wal_recovery(conn, "PASSIVE");

    // Stage 2: Truncate to reset log file size to zero
    execute_wal_recovery(conn, "TRUNCATE")
}
