//! Single-file pre-migration backup exporter.
//!
//! Dumps all historical Split-DB databases (`errors.db`, `repo_prompts.db`, `prompt-backups.db`, `task-history.db`)
//! and accounts verification metadata into a single unified JSON snapshot file.

use super::types::*;
use crate::error::AppError;
use crate::modules::db_migration::account_guard::{
    capture_accounts_snapshot, capture_accounts_snapshot_in,
};

pub use super::types::{
    AccountVerificationSnapshot, AccountsVerification, DatabaseExportPayload, MigrationSnapshot,
    PreMigrationSnapshot,
};
use rusqlite::Connection;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Dumps all rows from a given SQLite connection and table into a vector of generic JSON objects.
pub fn dump_table_rows(
    conn: &Connection,
    table_name: &str,
) -> Result<Vec<serde_json::Value>, AppError> {
    // Check if table exists first
    let table_exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?",
            [table_name],
            |row| row.get(0),
        )
        .map_err(AppError::Database)?;

    if table_exists == 0 {
        return Ok(Vec::new());
    }

    let mut stmt = conn
        .prepare(&format!("SELECT * FROM {}", table_name))
        .map_err(AppError::Database)?;

    let column_names: Vec<String> = stmt
        .column_names()
        .into_iter()
        .map(|s| s.to_string())
        .collect();

    let rows = stmt
        .query_map([], |row| {
            let mut map = serde_json::Map::new();
            for (i, col) in column_names.iter().enumerate() {
                let val_ref = row.get_ref(i)?;
                let json_val = match val_ref {
                    rusqlite::types::ValueRef::Null => serde_json::Value::Null,
                    rusqlite::types::ValueRef::Integer(n) => serde_json::json!(n),
                    rusqlite::types::ValueRef::Real(f) => serde_json::json!(f),
                    rusqlite::types::ValueRef::Text(t) => {
                        serde_json::json!(String::from_utf8_lossy(t).into_owned())
                    }
                    rusqlite::types::ValueRef::Blob(b) => {
                        serde_json::json!(base64::Engine::encode(
                            &base64::engine::general_purpose::STANDARD,
                            b
                        ))
                    }
                };
                map.insert(col.clone(), json_val);
            }
            Ok(serde_json::Value::Object(map))
        })
        .map_err(AppError::Database)?;

    let mut items = Vec::new();
    for r in rows {
        items.push(r.map_err(AppError::Database)?);
    }
    Ok(items)
}

/// Dumps a set of candidate tables from a specific SQLite file if it exists on disk.
pub fn dump_database_tables(
    db_path: &Path,
    db_key: &str,
    candidate_tables: &[&str],
) -> Option<DatabaseExportPayload> {
    if !db_path.exists() {
        return None;
    }

    let conn = match Connection::open(db_path) {
        Ok(c) => c,
        Err(err) => {
            crate::modules::logger::log_warn(&format!(
                "[DB Migration] Warning: Failed to open database at {:?}: {}",
                db_path, err
            ));
            return None;
        }
    };

    let mut payload = DatabaseExportPayload::new(db_key);

    for tbl in candidate_tables {
        match dump_table_rows(&conn, tbl) {
            Ok(rows) => {
                if !rows.is_empty() {
                    payload.add_table(*tbl, rows);
                }
            }
            Err(err) => {
                crate::modules::logger::log_warn(&format!(
                    "[DB Migration] Warning: Failed to dump table '{}' from {:?}: {}",
                    tbl, db_path, err
                ));
            }
        }
    }

    if payload.total_rows > 0 || !payload.tables.is_empty() {
        Some(payload)
    } else {
        None
    }
}

/// Resolves default location for pre-migration snapshot file: `~/.antigravity_tools/migration/pre_migration_v4_snapshot.json`.
pub fn get_default_snapshot_path() -> Result<PathBuf, AppError> {
    get_default_snapshot_path_in(None)
}

/// Resolves default location for pre-migration snapshot file with optional data directory override.
pub fn get_default_snapshot_path_in(data_dir_override: Option<&Path>) -> Result<PathBuf, AppError> {
    let data_dir = match data_dir_override {
        Some(d) => d.to_path_buf(),
        None => crate::modules::account::get_data_dir().map_err(AppError::Config)?,
    };
    let migration_dir = data_dir.join("migration");
    if !migration_dir.exists() {
        fs::create_dir_all(&migration_dir).map_err(AppError::Io)?;
    }
    Ok(migration_dir.join("pre_migration_v4_snapshot.json"))
}

/// Exports all historical databases and accounts status into a single unified JSON snapshot.
pub fn export_pre_migration_snapshot(custom_path: Option<&Path>) -> Result<PathBuf, AppError> {
    export_pre_migration_snapshot_in(custom_path, None)
}

/// Exports all historical databases and accounts status into a single unified JSON snapshot with optional directory override.
pub fn export_pre_migration_snapshot_in(
    custom_path: Option<&Path>,
    data_dir_override: Option<&Path>,
) -> Result<PathBuf, AppError> {
    let target_path = match custom_path {
        Some(p) => p.to_path_buf(),
        None => get_default_snapshot_path_in(data_dir_override)?,
    };

    if let Some(parent) = target_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(AppError::Io)?;
        }
    }

    let accounts_verif = capture_accounts_snapshot_in(data_dir_override)?;
    let mut db_dumps: HashMap<String, HashMap<String, Vec<serde_json::Value>>> = HashMap::new();
    let mut db_payloads: Vec<DatabaseExportPayload> = Vec::new();

    let data_dir = match data_dir_override {
        Some(d) => d.to_path_buf(),
        None => crate::modules::account::get_data_dir().map_err(AppError::Config)?,
    };

    // 1. Dump errors.db
    let errors_db_path = data_dir.join("errors.db");
    if let Some(payload) = dump_database_tables(
        &errors_db_path,
        "errors.db",
        &["errors", "ErrorLog", "SchemaVersion"],
    ) {
        db_dumps.insert("errors.db".to_string(), payload.tables.clone());
        db_payloads.push(payload);
    }

    // 2. Dump repo_prompts.db
    let repo_db_candidates = [
        data_dir.join("repo_prompts.db"),
        data_dir.join("repo-prompts.db"),
    ];
    let repo_tables = &[
        "running_projects",
        "active_prompts",
        "agm_project_sequences",
        "agm_conversation_sequences",
        "failed_commands",
        "RunningProject",
        "ActivePrompt",
        "ProjectSequence",
        "ConversationSequence",
        "FailedCommand",
        "PromptTreeCache",
        "MediaVault",
        "SchemaVersion",
    ];
    for candidate in &repo_db_candidates {
        if candidate.exists() {
            if let Some(payload) = dump_database_tables(candidate, "repo_prompts.db", repo_tables) {
                db_dumps.insert("repo_prompts.db".to_string(), payload.tables.clone());
                db_payloads.push(payload);
                break;
            }
        }
    }

    // 3. Dump backup-prompts.db
    let backup_candidates = [
        data_dir.join("backup-prompts").join("backup-prompts.db"),
        data_dir.join("backup-prompts").join("prompt-backups.db"),
        data_dir.join("prompt-backups.db"),
        data_dir.join("backup_prompts.db"),
    ];
    let backup_tables = &[
        "backup_batches",
        "prompt_backups",
        "green_projects",
        "BackupBatch",
        "PromptBackup",
        "GreenProject",
        "SchemaVersion",
    ];
    for candidate in &backup_candidates {
        if candidate.exists() {
            if let Some(payload) =
                dump_database_tables(candidate, "prompt-backups.db", backup_tables)
            {
                db_dumps.insert("prompt-backups.db".to_string(), payload.tables.clone());
                db_payloads.push(payload);
                break;
            }
        }
    }

    // 4. Dump task-history.db
    let task_history_candidates = [
        data_dir.join("task-history.db"),
        data_dir.join("task-history").join("task_index.db"),
        data_dir.join("task-history").join("task-history.db"),
    ];
    let task_history_tables = &[
        "task_history",
        "split_catalog",
        "hot_tasks_cache",
        "tasks",
        "TaskHistory",
        "SchemaVersion",
    ];
    let mut task_history_dumped = false;
    for candidate in &task_history_candidates {
        if candidate.exists() {
            if let Some(payload) =
                dump_database_tables(candidate, "task-history.db", task_history_tables)
            {
                db_dumps.insert("task-history.db".to_string(), payload.tables.clone());
                db_payloads.push(payload);
                task_history_dumped = true;
                break;
            }
        }
    }

    // If split directory exists, also check for any history-*.db split files
    let task_history_dir = data_dir.join("task-history");
    if task_history_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&task_history_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                let file_name = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
                if file_name.starts_with("history-") && file_name.ends_with(".db") {
                    if let Some(split_payload) =
                        dump_database_tables(&p, file_name, &["tasks", "TaskHistory"])
                    {
                        // Merge or append to task-history dump if not already present
                        let key = format!("task-history/{}", file_name);
                        db_dumps.insert(key, split_payload.tables.clone());
                        db_payloads.push(split_payload);
                    }
                }
            }
        }
    }

    let snapshot = MigrationSnapshot {
        format_version: "1.0".to_string(),
        exported_at: chrono::Utc::now().timestamp(),
        agm_version: env!("CARGO_PKG_VERSION").to_string(),
        accounts_verification: accounts_verif,
        databases: db_dumps,
    };

    let json_bytes = serde_json::to_vec_pretty(&snapshot).map_err(|e| {
        AppError::Config(format!("Failed to serialize pre-migration snapshot: {}", e))
    })?;

    fs::write(&target_path, json_bytes).map_err(AppError::Io)?;

    crate::modules::logger::log_info(&format!(
        "[DB Migration] Successfully exported pre-migration snapshot to {:?}",
        target_path
    ));

    Ok(target_path)
}
