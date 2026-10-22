//! SQLite Page Cache Memory Manager.
//! Computes and applies optimal negative `PRAGMA cache_size` (in Kibibytes) based on
//! platform memory resources and split database usage intensity.

use rusqlite::Connection;

/// Target memory profiles for page caching.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryProfile {
    LowMemory,     // 8 MB cache
    Standard,      // 32 MB cache
    HighThroughput, // 128 MB cache
}

impl MemoryProfile {
    /// Returns negative KiB value for `PRAGMA cache_size = -N;`.
    pub fn cache_size_kib(&self) -> i32 {
        match self {
            Self::LowMemory => -8192,         // 8 MB
            Self::Standard => -32768,        // 32 MB
            Self::HighThroughput => -131072, // 128 MB
        }
    }
}

/// Computes recommended memory profile based on detected physical RAM in Megabytes.
pub fn select_profile_for_ram(total_ram_mb: u64) -> MemoryProfile {
    if total_ram_mb <= 4096 {
        MemoryProfile::LowMemory
    } else if total_ram_mb <= 16384 {
        MemoryProfile::Standard
    } else {
        MemoryProfile::HighThroughput
    }
}

/// Applies negative `PRAGMA cache_size` to a SQLite connection.
pub fn apply_page_cache_size(
    conn: &Connection,
    profile: MemoryProfile,
) -> Result<i32, rusqlite::Error> {
    let size_kib = profile.cache_size_kib();
    conn.execute(&format!("PRAGMA cache_size = {};", size_kib), [])?;
    Ok(size_kib)
}

/// Queries current configured page cache size (`PRAGMA cache_size;`).
pub fn get_current_cache_size(conn: &Connection) -> Result<i32, rusqlite::Error> {
    conn.query_row("PRAGMA cache_size;", [], |row| row.get(0))
}
