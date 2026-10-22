//! PRAGMA Cache-Size Memory Tuner
//! Dynamically calculates optimal negative KiB PRAGMA cache_size values
//! based on split database role, page size, and memory constraints.

use rusqlite::Connection;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitDbRole {
    RepoPrompts,
    TaskHistory,
    PromptBackups,
    Errors,
    Generic,
}

pub struct CacheSizeTuner;

impl CacheSizeTuner {
    /// Computes the recommended cache_size in negative KiB for a given DB role.
    /// SQLite interprets negative values as KiB (e.g. -4000 = 4MiB cache).
    pub fn recommended_cache_size_kib(role: SplitDbRole) -> i32 {
        match role {
            SplitDbRole::RepoPrompts => -8192,   // 8 MiB (high read/write conversation tree)
            SplitDbRole::TaskHistory => -4096,   // 4 MiB (audit trail and history logs)
            SplitDbRole::PromptBackups => -2048, // 2 MiB (backup snapshots)
            SplitDbRole::Errors => -1024,        // 1 MiB (error logs)
            SplitDbRole::Generic => -2000,       // ~2 MiB standard default
        }
    }

    /// Applies the recommended cache_size to an open SQLite connection.
    pub fn apply_tuned_cache_size(conn: &Connection, role: SplitDbRole) -> Result<i32, rusqlite::Error> {
        let size_kib = Self::recommended_cache_size_kib(role);
        conn.execute_batch(&format!("PRAGMA cache_size = {};", size_kib))?;
        Ok(size_kib)
    }

    /// Reads the currently configured cache_size pragma value.
    pub fn query_current_cache_size(conn: &Connection) -> Result<i32, rusqlite::Error> {
        conn.query_row("PRAGMA cache_size;", [], |row| row.get(0))
    }
}
