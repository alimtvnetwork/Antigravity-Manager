//! Database File Lock Checker.
//! Detects active process write locks or WAL contention on SQLite files.

use rusqlite::{Connection, OpenFlags};
use std::path::Path;
use std::time::Duration;

/// Tests if a SQLite file is locked by an external process.
pub fn is_database_locked(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }

    match Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    ) {
        Ok(conn) => {
            let _ = conn.busy_timeout(Duration::from_millis(100));
            // Attempt immediate write transaction test
            match conn.execute("BEGIN IMMEDIATE;", []) {
                Ok(_) => {
                    let _ = conn.execute("ROLLBACK;", []);
                    false
                }
                Err(e) => {
                    let err_str = e.to_string().to_lowercase();
                    err_str.contains("busy") || err_str.contains("locked")
                }
            }
        }
        Err(e) => {
            let err_str = e.to_string().to_lowercase();
            err_str.contains("busy") || err_str.contains("locked")
        }
    }
}
