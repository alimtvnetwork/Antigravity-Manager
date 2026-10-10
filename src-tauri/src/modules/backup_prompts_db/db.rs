use crate::modules::account;
use chrono::Utc;
use rusqlite::{params, Connection};
use std::fs;
use std::path::{Path, PathBuf};

use super::*;

/// Determine target database path for running prompt backups
pub fn get_backup_prompts_db_path(custom_file: Option<&str>) -> Result<PathBuf, String> {
    if let Some(file_str) = custom_file {
        let trimmed = file_str.trim();
        if !trimmed.is_empty() {
            let path = PathBuf::from(trimmed);
            if path.extension().is_none() {
                return Ok(path.with_extension("db"));
            }
            return Ok(path);
        }
    }

    // Check if local data/backup-prompts exists close to the CLI
    let local_backup_dir = PathBuf::from("data").join("backup-prompts");
    if local_backup_dir.exists() {
        if local_backup_dir.join("SQL.db").exists() {
            return Ok(local_backup_dir.join("SQL.db"));
        }
        return Ok(local_backup_dir.join("backup-prompts.db"));
    }

    let data_dir = account::get_data_dir()?;
    let backup_dir = data_dir.join("backup-prompts");
    if let Err(e) = fs::create_dir_all(&backup_dir) {
        return Err(format!("Failed to create backup-prompts folder: {}", e));
    }
    if backup_dir.join("SQL.db").exists() {
        return Ok(backup_dir.join("SQL.db"));
    }
    Ok(backup_dir.join("backup-prompts.db"))
}

/// Open and initialize the split SQLite database connection
pub fn connect_backup_db(custom_file: Option<&str>) -> Result<Connection, String> {
    let db_path = get_backup_prompts_db_path(custom_file)?;
    if let Some(parent) = db_path.parent() {
        // Justification: best-effort directory creation; later file ops fail loudly if the directory is actually needed
        crate::error::record_ignored(fs::create_dir_all(parent), "create_dir_all");
    }

    let conn = Connection::open(&db_path).map_err(|e| {
        format!(
            "Failed to open backup prompts database '{:?}': {}",
            db_path, e
        )
    })?;

    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA foreign_keys = ON;

         CREATE TABLE IF NOT EXISTS backup_batches (
             id TEXT PRIMARY KEY,
             created_at INTEGER NOT NULL,
             prompts_count INTEGER NOT NULL,
             file_path TEXT NOT NULL,
             retention_days INTEGER NOT NULL DEFAULT 1,
             is_fully_restored BOOLEAN NOT NULL DEFAULT 0
         );

         CREATE TABLE IF NOT EXISTS prompt_backups (
             id TEXT PRIMARY KEY,
             backup_batch_id TEXT NOT NULL,
             prompt_id TEXT NOT NULL,
             project_name TEXT NOT NULL,
             project_path TEXT NOT NULL,
             project_id TEXT NOT NULL,
             conversation_id TEXT NOT NULL,
             conversation_name TEXT,
             sequence_id INTEGER NOT NULL,
             prompt_text TEXT NOT NULL,
             has_images BOOLEAN NOT NULL DEFAULT 0,
             images_payload TEXT,
             status TEXT NOT NULL DEFAULT 'queued',
             created_at INTEGER NOT NULL,
             is_restored BOOLEAN NOT NULL DEFAULT 0,
             restored_at INTEGER,
             instance_id TEXT DEFAULT 'default',
             FOREIGN KEY(backup_batch_id) REFERENCES backup_batches(id) ON DELETE CASCADE
         );
         CREATE INDEX IF NOT EXISTS idx_prompt_backups_batch ON prompt_backups(backup_batch_id);
         CREATE INDEX IF NOT EXISTS idx_prompt_backups_restored ON prompt_backups(is_restored, restored_at);

         CREATE TABLE IF NOT EXISTS green_projects (
             id TEXT PRIMARY KEY,
             project_identifier TEXT NOT NULL UNIQUE,
             project_path TEXT NOT NULL,
             added_at INTEGER NOT NULL,
             status TEXT NOT NULL DEFAULT 'pending',
             last_checked_at INTEGER
         );",
    )
    .map_err(|e| format!("Failed to initialize backup prompts tables: {}", e))?;

    // Migrate existing DB if instance_id is missing
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(
        conn.execute(
            "ALTER TABLE prompt_backups ADD COLUMN instance_id TEXT DEFAULT 'default'",
            [],
        ),
        "db execute",
    );
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_prompt_backups_instance ON prompt_backups(instance_id)",
            [],
        ),
        "db execute",
    );

    Ok(conn)
}

/// Automatically clean up records that were restored more than 1 day ago (default 86400s)
pub fn auto_cleanup_expired(
    custom_file: Option<&str>,
    retention_seconds: i64,
) -> Result<usize, String> {
    let conn = connect_backup_db(custom_file)?;
    let cutoff = Utc::now().timestamp() - retention_seconds;

    let removed = conn
        .execute(
            "DELETE FROM prompt_backups WHERE is_restored = 1 AND restored_at IS NOT NULL AND restored_at <= ?",
            params![cutoff],
        )
        .map_err(|e| format!("Failed to delete expired prompt backups: {}", e))?;

    // Also clean up empty fully-restored batches
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(conn.execute(
        "DELETE FROM backup_batches WHERE is_fully_restored = 1 AND id NOT IN (SELECT DISTINCT backup_batch_id FROM prompt_backups)",
        [],
    ), "db execute");

    Ok(removed)
}

/// Force clean all backup prompts and batches
pub fn force_clean_all(custom_file: Option<&str>) -> Result<usize, String> {
    let conn = connect_backup_db(custom_file)?;
    let count = conn
        .execute("DELETE FROM prompt_backups", [])
        .map_err(|e| format!("Failed to clear prompt backups: {}", e))?;
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(conn.execute("DELETE FROM backup_batches", []), "db execute");
    Ok(count)
}
