// Passive WAL Checkpoint Scheduler (Imp-523)
// Executes non-blocking PRAGMA wal_checkpoint(PASSIVE) during idle periods
// to checkpoint committed frames to the main database file without blocking readers.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tracing::debug;

pub const DEFAULT_IDLE_PASSIVE_INTERVAL_SECS: u64 = 60;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassiveCheckpointStats {
    pub busy: bool,
    pub log_frames: i32,
    pub checkpointed_frames: i32,
    pub remaining_uncheckpointed: i32,
}

pub struct WalPassiveCheckpointScheduler;

impl WalPassiveCheckpointScheduler {
    /// Executes a PASSIVE checkpoint on the provided SQLite connection.
    pub fn execute_passive_checkpoint(conn: &Connection) -> Result<PassiveCheckpointStats, rusqlite::Error> {
        let mut stmt = conn.prepare("PRAGMA wal_checkpoint(PASSIVE);")?;
        let stats = stmt.query_row([], |row| {
            let busy_code: i32 = row.get(0)?;
            let log_frames: i32 = row.get(1)?;
            let checkpointed: i32 = row.get(2)?;
            Ok(PassiveCheckpointStats {
                busy: busy_code != 0,
                log_frames,
                checkpointed_frames: checkpointed,
                remaining_uncheckpointed: log_frames.saturating_sub(checkpointed),
            })
        })?;

        debug!(
            busy = stats.busy,
            log_frames = stats.log_frames,
            checkpointed = stats.checkpointed_frames,
            "Executed SQLite PASSIVE checkpoint"
        );

        Ok(stats)
    }
}
