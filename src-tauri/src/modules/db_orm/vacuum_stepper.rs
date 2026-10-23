//! SQLite Auto-Vacuum Incremental Stepper.
//! Executes bounded `PRAGMA incremental_vacuum(N)` in chunks of 100 pages, reclaiming freelist pages incrementally.

use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};

/// Default page chunk size per vacuum step: 100 pages (~400KB in standard 4KB page size).
pub const DEFAULT_VACUUM_CHUNK_SIZE: i64 = 100;
/// Default maximum pages to reclaim in a single maintenance pass: 1,000 pages (~4MB).
pub const DEFAULT_MAX_PAGES_PER_PASS: i64 = 1000;

/// Report detailing freelist status and incremental vacuum pages reclaimed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IncrementalVacuumReport {
    pub freelist_before: i64,
    pub freelist_after: i64,
    pub pages_reclaimed: i64,
    pub steps_executed: usize,
}

/// Incrementally reclaims freelist pages in bounded chunks to avoid prolonged lock duration.
pub fn step_incremental_vacuum(
    conn: &Connection,
    max_pages: Option<i64>,
    chunk_size: Option<i64>,
) -> Result<IncrementalVacuumReport> {
    let freelist_before: i64 = conn.query_row("PRAGMA freelist_count", [], |r| r.get(0))?;
    if freelist_before <= 0 {
        return Ok(IncrementalVacuumReport {
            freelist_before: 0,
            freelist_after: 0,
            pages_reclaimed: 0,
            steps_executed: 0,
        });
    }

    let chunk = chunk_size.unwrap_or(DEFAULT_VACUUM_CHUNK_SIZE).max(1);
    let limit = max_pages.unwrap_or(DEFAULT_MAX_PAGES_PER_PASS).min(freelist_before);

    let mut remaining = limit;
    let mut steps_executed = 0;

    while remaining > 0 {
        let current_step = remaining.min(chunk);
        let sql = format!("PRAGMA incremental_vacuum({})", current_step);
        conn.execute_batch(&sql)?;
        remaining -= current_step;
        steps_executed += 1;
    }

    let freelist_after: i64 = conn.query_row("PRAGMA freelist_count", [], |r| r.get(0))?;
    let pages_reclaimed = (freelist_before - freelist_after).max(0);

    Ok(IncrementalVacuumReport {
        freelist_before,
        freelist_after,
        pages_reclaimed,
        steps_executed,
    })
}
