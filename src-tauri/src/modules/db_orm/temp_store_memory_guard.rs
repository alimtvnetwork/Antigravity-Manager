// Temporary Storage Memory Guard (Imp-497)
// Enforces PRAGMA temp_store = MEMORY on SQLite connections to ensure
// intermediate sorting, hash joins, and temporary indices reside in RAM.

use rusqlite::Connection;
use tracing::debug;

pub const TEMP_STORE_MEMORY_MODE: i32 = 2; // 0=DEFAULT, 1=FILE, 2=MEMORY

pub struct TempStoreMemoryGuard;

impl TempStoreMemoryGuard {
    /// Applies PRAGMA temp_store = MEMORY to the connection and verifies setting.
    pub fn enforce_memory_temp_store(conn: &Connection) -> Result<bool, rusqlite::Error> {
        conn.execute("PRAGMA temp_store = MEMORY;", [])?;

        let mut stmt = conn.prepare("PRAGMA temp_store;")?;
        let current_mode: i32 = stmt.query_row([], |row| row.get(0))?;

        let is_memory = current_mode == TEMP_STORE_MEMORY_MODE;
        if is_memory {
            debug!("PRAGMA temp_store successfully configured to MEMORY mode");
        } else {
            tracing::warn!(
                current_mode,
                "SQLite temp_store pragma did not take effect, operating in mode {}",
                current_mode
            );
        }

        Ok(is_memory)
    }
}
