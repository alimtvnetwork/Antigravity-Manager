// Periodic WAL Truncate Checkpoint Scheduler (Imp-493)
// Evaluates WAL log growth and executes PRAGMA wal_checkpoint(TRUNCATE)
// to prevent disk bloat while handling concurrency gracefully.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

pub const DEFAULT_MAX_WAL_PAGES: i64 = 500;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalCheckpointResult {
    pub busy: bool,
    pub log_pages: i64,
    pub checkpointed_pages: i64,
    pub truncated: bool,
}

pub struct WalTruncateScheduler;

impl WalTruncateScheduler {
    /// Executes a TRUNCATE checkpoint on the SQLite connection.
    pub fn execute_truncate_checkpoint(
        conn: &Connection,
    ) -> Result<WalCheckpointResult, rusqlite::Error> {
        let mut stmt = conn.prepare("PRAGMA wal_checkpoint(TRUNCATE);")?;
        let result = stmt.query_row([], |row| {
            let busy_code: i32 = row.get(0)?;
            let log_pages: i64 = row.get(1)?;
            let checkpointed: i64 = row.get(2)?;
            Ok(WalCheckpointResult {
                busy: busy_code != 0,
                log_pages,
                checkpointed_pages: checkpointed,
                truncated: busy_code == 0 && log_pages == 0,
            })
        })?;

        if result.busy {
            warn!("WAL TRUNCATE checkpoint was blocked by concurrent readers");
        } else {
            debug!(
                log_pages = result.log_pages,
                checkpointed = result.checkpointed_pages,
                "WAL TRUNCATE checkpoint finished successfully"
            );
        }

        Ok(result)
    }

    /// Checks if a truncate checkpoint is advised based on current WAL page count.
    pub fn is_checkpoint_advised(current_wal_pages: i64, threshold: Option<i64>) -> bool {
        let max_pages = threshold.unwrap_or(DEFAULT_MAX_WAL_PAGES);
        current_wal_pages >= max_pages
    }
}
