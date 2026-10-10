//! Instance filesystem paths and registry/sqlite locations.
use super::*;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Get base directory for storing instance profiles
pub fn get_instances_dir() -> Result<PathBuf, String> {
    let base_dir = crate::modules::account::get_data_dir()
        .map_err(|e| format!("Failed to get config dir: {}", e))?;
    let instances_dir = base_dir.join("instances");
    if !instances_dir.exists() {
        fs::create_dir_all(&instances_dir)
            .map_err(|e| format!("Failed to create instances directory: {}", e))?;
    }
    Ok(instances_dir)
}

/// Resolve or create the isolated home directory for an instance
pub fn get_instance_home_dir(instance_id: &str) -> Result<PathBuf, String> {
    let resolved_id = resolve_instance_id(instance_id).unwrap_or_else(|_| instance_id.to_string());
    let instances_root = get_instances_dir()?;
    let home_dir = instances_root.join(&resolved_id).join("home");
    if !home_dir.exists() {
        fs::create_dir_all(&home_dir)
            .map_err(|e| format!("Failed to create instance home directory: {}", e))?;
    }
    Ok(home_dir)
}

/// Path to instances.json registry
pub fn get_registry_path() -> Result<PathBuf, String> {
    let dir = get_instances_dir()?;
    Ok(dir.join("instances.json"))
}

/// Path to instances.db SQLite database
pub fn get_instance_db_path() -> Result<PathBuf, String> {
    let dir = get_instances_dir()?;
    Ok(dir.join("instances.db"))
}

/// Open and initialize the instance SQLite database with WAL mode
pub fn open_instance_db() -> Result<rusqlite::Connection, String> {
    let db_path = get_instance_db_path()?;
    let conn = rusqlite::Connection::open(db_path).map_err(|e| e.to_string())?;

    // Justification: pragma is performance/concurrency tuning; the connection stays usable without it
    crate::error::record_ignored(
        conn.pragma_update(None, "journal_mode", "WAL"),
        "set sqlite pragma",
    );
    // Justification: pragma is performance/concurrency tuning; the connection stays usable without it
    crate::error::record_ignored(
        conn.pragma_update(None, "busy_timeout", 5000),
        "set sqlite pragma",
    );
    // Justification: pragma is performance/concurrency tuning; the connection stays usable without it
    crate::error::record_ignored(
        conn.pragma_update(None, "synchronous", "NORMAL"),
        "set sqlite pragma",
    );

    conn.execute(
        "CREATE TABLE IF NOT EXISTS instance_processes (
            instance_id TEXT PRIMARY KEY,
            pid INTEGER NOT NULL,
            data_dir TEXT NOT NULL,
            launched_at INTEGER NOT NULL,
            is_active INTEGER NOT NULL DEFAULT 1
        )",
        [],
    )
    .map_err(|e| format!("Failed to create instance_processes table: {}", e))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS active_instance_selection (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            instance_id TEXT NOT NULL,
            updated_at INTEGER NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("Failed to create active_instance_selection table: {}", e))?;

    Ok(conn)
}
