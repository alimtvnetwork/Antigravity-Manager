//! Database Temp Store Memory Monitor.
//! Verifies `PRAGMA temp_store` memory allocation status and configures in-memory temporary storage.

use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};

/// SQLite temp_store storage mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TempStoreMode {
    Default = 0,
    File = 1,
    Memory = 2,
    Unknown = -1,
}

impl From<i32> for TempStoreMode {
    fn from(val: i32) -> Self {
        match val {
            0 => Self::Default,
            1 => Self::File,
            2 => Self::Memory,
            _ => Self::Unknown,
        }
    }
}

/// Status and configuration of database temp storage and caching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TempStoreStatus {
    pub mode: TempStoreMode,
    pub is_memory: bool,
    pub page_size: i64,
    pub cache_size: i64,
}

/// Inspects current temporary storage mode and page settings.
pub fn inspect_temp_store_status(conn: &Connection) -> Result<TempStoreStatus> {
    let mode_int: i32 = conn.query_row("PRAGMA temp_store", [], |row| row.get(0))?;
    let mode = TempStoreMode::from(mode_int);
    let is_memory = mode == TempStoreMode::Memory;

    let page_size: i64 = conn.query_row("PRAGMA page_size", [], |row| row.get(0))?;
    let cache_size: i64 = conn.query_row("PRAGMA cache_size", [], |row| row.get(0))?;

    Ok(TempStoreStatus {
        mode,
        is_memory,
        page_size,
        cache_size,
    })
}

/// Ensures that temporary tables and indices reside purely in RAM.
pub fn ensure_temp_store_memory(conn: &Connection) -> Result<()> {
    conn.execute_batch("PRAGMA temp_store = MEMORY;")?;
    Ok(())
}
