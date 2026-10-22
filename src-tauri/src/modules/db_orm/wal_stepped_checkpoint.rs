//! WAL Stepped Incremental Checkpointer
//!
//! Provides progressive SQLite WAL checkpointing: begins with non-blocking `PASSIVE`
//! checkpointing, measures uncheckpointed frame counts, and escalates to `RESTART` or
//! `TRUNCATE` only when lock contention is low and log size warrants truncation.

use rusqlite::Connection;
use tracing::{debug, warn};

/// Checkpoint mode to apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalStepMode {
    Passive,
    Restart,
    Truncate,
}

impl WalStepMode {
    pub fn as_pragma_arg(&self) -> &'static str {
        match self {
            WalStepMode::Passive => "PASSIVE",
            WalStepMode::Restart => "RESTART",
            WalStepMode::Truncate => "TRUNCATE",
        }
    }
}

/// Result of a stepped checkpoint execution.
#[derive(Debug, Clone)]
pub struct SteppedCheckpointResult {
    pub busy: bool,
    pub log_frames: i32,
    pub checkpointed_frames: i32,
    pub mode_used: WalStepMode,
}

/// Executes a stepped WAL checkpoint on the given connection.
pub fn execute_stepped_wal_checkpoint(
    conn: &Connection,
    max_acceptable_uncheckpointed_frames: i32,
) -> Result<SteppedCheckpointResult, rusqlite::Error> {
    // Step 1: Attempt PASSIVE checkpoint first (never blocks readers or writers)
    let passive_res = run_checkpoint_pragma(conn, WalStepMode::Passive)?;

    let uncheckpointed = passive_res.log_frames - passive_res.checkpointed_frames;

    // If passive checkpoint succeeded and remaining frames are within budget, return early
    if !passive_res.busy && uncheckpointed <= max_acceptable_uncheckpointed_frames {
        debug!(
            log_frames = passive_res.log_frames,
            checkpointed = passive_res.checkpointed_frames,
            "PASSIVE WAL checkpoint completed within threshold"
        );
        return Ok(passive_res);
    }

    // Step 2: If many uncheckpointed frames remain and database is not busy, escalate to RESTART
    if !passive_res.busy && uncheckpointed > max_acceptable_uncheckpointed_frames {
        debug!(
            uncheckpointed = uncheckpointed,
            "Escalating to RESTART WAL checkpoint"
        );
        let restart_res = run_checkpoint_pragma(conn, WalStepMode::Restart)?;
        if !restart_res.busy {
            return Ok(restart_res);
        }
    }

    warn!(
        busy = passive_res.busy,
        uncheckpointed = uncheckpointed,
        "WAL checkpoint concluded with residual frames or active readers"
    );

    Ok(passive_res)
}

fn run_checkpoint_pragma(
    conn: &Connection,
    mode: WalStepMode,
) -> Result<SteppedCheckpointResult, rusqlite::Error> {
    let sql = format!("PRAGMA wal_checkpoint({});", mode.as_pragma_arg());
    let mut stmt = conn.prepare(&sql)?;

    let mut rows = stmt.query([])?;
    if let Some(row) = rows.next()? {
        let busy_int: i32 = row.get(0).unwrap_or(0);
        let log_frames: i32 = row.get(1).unwrap_or(0);
        let checkpointed_frames: i32 = row.get(2).unwrap_or(0);

        Ok(SteppedCheckpointResult {
            busy: busy_int != 0,
            log_frames,
            checkpointed_frames,
            mode_used: mode,
        })
    } else {
        Ok(SteppedCheckpointResult {
            busy: false,
            log_frames: 0,
            checkpointed_frames: 0,
            mode_used: mode,
        })
    }
}
