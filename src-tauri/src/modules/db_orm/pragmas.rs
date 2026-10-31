//! High-Performance SQLite Pragmas & Connection Tuner.
//! Configures optimal cache sizing, memory mapping, WAL synchronization, and busy timeouts.

use rusqlite::{Connection, Result};

/// Applies optimal production tuning pragmas to an open SQLite connection.
pub fn apply_performance_pragmas(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "PRAGMA synchronous = NORMAL;
         PRAGMA journal_mode = WAL;
         PRAGMA cache_size = -64000;
         PRAGMA temp_store = MEMORY;
         PRAGMA mmap_size = 268435456;
         PRAGMA busy_timeout = 5000;",
    )?;
    Ok(())
}
