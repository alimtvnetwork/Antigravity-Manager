//! SQLite Index & Database Health Diagnostic Scanner.
//! Runs `PRAGMA quick_check` and `PRAGMA integrity_check` on demand, returning structured health diagnostics.

use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};

/// Detailed health check report for a split SQLite database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbHealthReport {
    pub is_healthy: bool,
    pub quick_check_ok: bool,
    pub integrity_errors: Vec<String>,
    pub scanned_at_secs: u64,
}

/// Scans the connected database for B-tree and index corruption.
///
/// If `full_check` is false, runs `PRAGMA quick_check(10)` which performs rapid validation.
/// If `full_check` is true, runs `PRAGMA integrity_check(100)` for exhaustive verification.
pub fn scan_database_health(conn: &Connection, full_check: bool) -> Result<DbHealthReport> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // 1. Quick check
    let mut quick_stmt = conn.prepare("PRAGMA quick_check(5)")?;
    let quick_results: Vec<String> = quick_stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .filter_map(|r| r.ok())
        .collect();

    let quick_check_ok = quick_results.len() == 1 && quick_results[0].eq_ignore_ascii_case("ok");

    let mut integrity_errors = Vec::new();

    if full_check || !quick_check_ok {
        let mut integrity_stmt = conn.prepare("PRAGMA integrity_check(50)")?;
        let full_results: Vec<String> = integrity_stmt
            .query_map([], |row| row.get::<_, String>(0))?
            .filter_map(|r| r.ok())
            .collect();

        for msg in full_results {
            if !msg.eq_ignore_ascii_case("ok") {
                integrity_errors.push(msg);
            }
        }
    }

    let is_healthy = quick_check_ok && integrity_errors.is_empty();

    Ok(DbHealthReport {
        is_healthy,
        quick_check_ok,
        integrity_errors,
        scanned_at_secs: now,
    })
}
