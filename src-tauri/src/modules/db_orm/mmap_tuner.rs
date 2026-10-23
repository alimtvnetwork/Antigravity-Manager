//! Platform Memory-Mapped I/O Tuner
//! Configures optimal PRAGMA mmap_size values based on target architecture
//! address space limits (256MB for 64-bit systems, 0 or 32MB for 32-bit systems).

use rusqlite::Connection;

pub struct MmapTuner;

impl MmapTuner {
    /// Recommended mmap_size in bytes for current platform architecture.
    pub const RECOMMENDED_MMAP_BYTES: i64 = if cfg!(target_pointer_width = "64") {
        268_435_456 // 256 MiB
    } else {
        33_554_432 // 32 MiB
    };

    /// Applies the recommended mmap_size to an open SQLite connection.
    pub fn apply_mmap_size(conn: &Connection) -> Result<i64, rusqlite::Error> {
        conn.execute_batch(&format!(
            "PRAGMA mmap_size = {};",
            Self::RECOMMENDED_MMAP_BYTES
        ))?;
        Self::query_current_mmap_size(conn)
    }

    /// Queries currently configured mmap_size.
    pub fn query_current_mmap_size(conn: &Connection) -> Result<i64, rusqlite::Error> {
        conn.query_row("PRAGMA mmap_size;", [], |row| row.get(0))
    }
}
