//! SQLite VACUUM INTO Live Backup Engine.
//! Generates clean, atomic database snapshots on live split databases without blocking readers.

use rusqlite::{Connection, Result};
use std::path::Path;

/// Executes a live SQLite VACUUM INTO command to generate an atomic point-in-time backup.
pub fn vacuum_database_into(conn: &Connection, target_path: &Path) -> Result<()> {
    if let Some(parent) = target_path.parent() {
        if !parent.exists() {
            let _ = std::fs::create_dir_all(parent);
        }
    }

    if target_path.exists() {
        let _ = std::fs::remove_file(target_path);
    }

    let path_str = target_path.to_string_lossy().to_string();
    let escaped_path = path_str.replace('\'', "''");
    let sql = format!("VACUUM INTO '{}'", escaped_path);

    conn.execute(&sql, [])?;
    Ok(())
}
