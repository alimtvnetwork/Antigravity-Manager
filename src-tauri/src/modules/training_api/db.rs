use rusqlite::{params, Connection};
use std::fs;
use std::path::PathBuf;

use super::*;

pub fn get_training_db_path() -> Result<PathBuf, String> {
    let data_dir = crate::modules::account::get_data_dir()
        .map_err(|e| format!("Failed to get data dir: {}", e))?;
    Ok(data_dir.join("training_vault.db"))
}

/// Connect to the training database with WAL mode and 5000ms busy timeout
pub fn connect_training_db() -> Result<Connection, String> {
    let path = get_training_db_path()?;
    if let Some(parent) = path.parent() {
        // Justification: best-effort directory creation; later file ops fail loudly if the directory is actually needed
        crate::error::record_ignored(fs::create_dir_all(parent), "create_dir_all");
    }
    let conn =
        Connection::open(&path).map_err(|e| format!("Failed to open training database: {}", e))?;
    // Justification: best-effort SQLite pragma; logged
    crate::error::record_ignored(
        conn.pragma_update(None, "journal_mode", "WAL"),
        "pragma_update",
    );
    // Justification: best-effort SQLite pragma; the connection remains usable without it
    crate::error::record_ignored(
        conn.pragma_update(None, "busy_timeout", 5000),
        "busy_timeout",
    );
    init_tables(&conn)?;
    Ok(conn)
}

pub(crate) fn init_tables(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS training_logs (
            id TEXT PRIMARY KEY,
            session_id TEXT,
            model TEXT,
            prompt_type TEXT,
            input_tokens INTEGER,
            output_tokens INTEGER,
            latency_ms INTEGER,
            success INTEGER NOT NULL DEFAULT 1,
            score REAL,
            feedback TEXT,
            adjust_routing INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("Failed to create training_logs table: {}", e))?;

    Ok(())
}
