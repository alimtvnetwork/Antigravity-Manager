//! SQLite Database connection and schema initialization for `errors.db`.

use crate::modules::account;
use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;

/// Resolve path to the SQLite `errors.db` database
pub fn get_error_db_path() -> Result<PathBuf, String> {
    // Check if custom data dir or local data dir exists
    let local_data_dir = PathBuf::from("data");
    if local_data_dir.exists() && local_data_dir.join("errors.db").exists() {
        return Ok(local_data_dir.join("errors.db"));
    }

    let data_dir = account::get_data_dir().map_err(|e| format!("Failed to get data dir: {e}"))?;
    if let Err(e) = fs::create_dir_all(&data_dir) {
        return Err(format!("Failed to create data dir: {e}"));
    }
    Ok(data_dir.join("errors.db"))
}

/// Open and initialize connection to `errors.db` with WAL mode
pub fn connect_error_db() -> Result<Connection, String> {
    let db_path = get_error_db_path()?;
    if let Some(parent) = db_path.parent() {
        // Justification: best-effort directory creation; file open below will report critical error if path is invalid
        crate::error::record_ignored(fs::create_dir_all(parent), "create_dir_all_error_db");
    }

    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Failed to open error database at '{db_path:?}': {e}"))?;

    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA busy_timeout = 5000;
         PRAGMA foreign_keys = ON;

         CREATE TABLE IF NOT EXISTS errors (
             id TEXT PRIMARY KEY,
             error_code TEXT NOT NULL,
             severity TEXT NOT NULL,
             message TEXT NOT NULL,
             stack_trace TEXT,
             source TEXT,
             trigger_action TEXT,
             metadata_json TEXT,
             created_at INTEGER NOT NULL
         );

         CREATE INDEX IF NOT EXISTS idx_errors_created_at ON errors(created_at DESC);
         CREATE INDEX IF NOT EXISTS idx_errors_code ON errors(error_code);
         CREATE INDEX IF NOT EXISTS idx_errors_severity ON errors(severity);",
    )
    .map_err(|e| format!("Failed to initialize errors schema: {e}"))?;

    Ok(conn)
}
