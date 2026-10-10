use rusqlite::{params, Connection, OpenFlags};
use std::path::PathBuf;

use super::*;

pub fn get_proxy_db_path() -> Result<PathBuf, String> {
    let data_dir = crate::modules::account::get_data_dir()?;
    Ok(data_dir.join("proxy_logs.db"))
}

pub fn get_thinking_db_path() -> Result<PathBuf, String> {
    let data_dir = crate::modules::account::get_data_dir()?;
    Ok(data_dir.join("thinking_store.db"))
}

pub(crate) fn apply_fast_pragmas(conn: &Connection) -> Result<(), String> {
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|e| e.to_string())?;
    conn.pragma_update(None, "busy_timeout", 5000)
        .map_err(|e| e.to_string())?;
    conn.pragma_update(None, "synchronous", "NORMAL")
        .map_err(|e| e.to_string())?;
    // Justification: performance hint; the database opens and works with defaults
    crate::error::record_ignored(
        conn.pragma_update(None, "cache_size", -64000),
        "apply cache_size pragma",
    );
    // Justification: performance hint; the database opens and works with defaults
    crate::error::record_ignored(
        conn.pragma_update(None, "temp_store", "MEMORY"),
        "apply temp_store pragma",
    );
    // Justification: performance hint; the database opens and works with defaults
    crate::error::record_ignored(
        conn.pragma_update(None, "mmap_size", 268435456),
        "apply mmap_size pragma",
    );
    Ok(())
}

pub(crate) fn connect_db() -> Result<Connection, String> {
    let db_path = get_proxy_db_path()?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    apply_fast_pragmas(&conn)?;
    Ok(conn)
}
