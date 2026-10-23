//! Explicit Write-Ahead Log Flusher
//! Executes controlled WAL checkpoints (PASSIVE, FULL, RESTART, TRUNCATE)
//! to ensure all dirty SQLite pages commit to disk prior to process handoff.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalCheckpointMode {
    Passive,
    Full,
    Restart,
    Truncate,
}

impl WalCheckpointMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            WalCheckpointMode::Passive => "PASSIVE",
            WalCheckpointMode::Full => "FULL",
            WalCheckpointMode::Restart => "RESTART",
            WalCheckpointMode::Truncate => "TRUNCATE",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalFlushResult {
    pub is_busy: bool,
    pub log_frames: i32,
    pub checkpointed_frames: i32,
}

pub struct WriteAheadFlush;

impl WriteAheadFlush {
    /// Executes a WAL checkpoint with the specified mode.
    /// PRAGMA wal_checkpoint returns (busy: 0|1, log: count, checkpointed: count).
    pub fn flush_wal(
        conn: &Connection,
        mode: WalCheckpointMode,
    ) -> Result<WalFlushResult, rusqlite::Error> {
        let sql = format!("PRAGMA wal_checkpoint({});", mode.as_str());
        conn.query_row(&sql, [], |row| {
            let busy: i32 = row.get(0)?;
            let log_frames: i32 = row.get(1)?;
            let checkpointed_frames: i32 = row.get(2)?;

            Ok(WalFlushResult {
                is_busy: busy != 0,
                log_frames,
                checkpointed_frames,
            })
        })
    }

    /// Convenience method to perform a clean truncate checkpoint.
    pub fn truncate_wal(conn: &Connection) -> Result<WalFlushResult, rusqlite::Error> {
        Self::flush_wal(conn, WalCheckpointMode::Truncate)
    }
}
