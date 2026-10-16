//! SQLite persistence for UI interaction audit entries.
//!
//! Storage convention mirrors `task_history_db`: a local `data/ui-audit`
//! folder wins for dev/tests, otherwise `<agm-data-dir>/ui-audit` resolved
//! via `crate::modules::account::get_data_dir()`.

use crate::error::{AppError, AppResult};
use rusqlite::{params, Connection};
use std::fs;
use std::path::{Path, PathBuf};

use super::types::InteractionEntry;

const DB_FILE_NAME: &str = "ui_audit.db";

const SCHEMA_SQL: &str = "
CREATE TABLE IF NOT EXISTS interactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp INTEGER NOT NULL,
    actor TEXT NOT NULL,
    method_name TEXT NOT NULL,
    ui_target TEXT NOT NULL,
    trigger TEXT NOT NULL,
    params TEXT NOT NULL DEFAULT '{}',
    result TEXT NOT NULL,
    message TEXT NOT NULL DEFAULT '',
    stack_trace TEXT
);
CREATE INDEX IF NOT EXISTS idx_interactions_timestamp ON interactions(timestamp);
CREATE TABLE IF NOT EXISTS recorder_state (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
";

/// Attach filesystem context to an IO error before routing it into `AppError`.
fn io_with_context(path: &Path, op: &str, err: std::io::Error) -> AppError {
    AppError::Io(std::io::Error::new(
        err.kind(),
        format!("{op} {path:?}: {err}"),
    ))
}

/// Resolve the audit DB directory following the repo storage convention.
pub fn audit_dir() -> AppResult<PathBuf> {
    let local = PathBuf::from("data").join("ui-audit");
    if local.exists() {
        return Ok(local);
    }
    let data_dir = crate::modules::account::get_data_dir()
        .map_err(|err| AppError::Unknown(format!("resolve AGM data dir: {err}")))?;
    let dir = data_dir.join("ui-audit");
    fs::create_dir_all(&dir).map_err(|err| io_with_context(&dir, "create ui-audit folder", err))?;
    Ok(dir)
}

fn open_db() -> AppResult<Connection> {
    let dir = audit_dir()?;
    let conn = Connection::open(dir.join(DB_FILE_NAME))?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.execute_batch(SCHEMA_SQL)?;
    // Justification: idempotent schema migration; failure is expected when the column already exists.
    crate::error::record_ignored(
        conn.execute("ALTER TABLE interactions ADD COLUMN stack_trace TEXT", []),
        "add stack_trace column migration",
    );
    Ok(conn)
}

/// Fail fast on DB access problems (used at `start` so a broken DB surfaces
/// immediately instead of silently dropping later records).
pub fn ensure_db() -> AppResult<()> {
    open_db().map(|_| ())
}

/// Append one interaction entry; returns the new row id.
#[allow(clippy::too_many_arguments)]
pub fn append_entry(
    timestamp: i64,
    actor: &str,
    method_name: &str,
    ui_target: &str,
    trigger: &str,
    params: &str,
    result: &str,
    message: &str,
    stack_trace: Option<&str>,
) -> AppResult<i64> {
    let conn = open_db()?;
    conn.execute(
        "INSERT INTO interactions
            (timestamp, actor, method_name, ui_target, trigger, params, result, message, stack_trace)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            timestamp,
            actor,
            method_name,
            ui_target,
            trigger,
            params,
            result,
            message,
            stack_trace
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// List the newest entries, capped at `limit` (clamped to 1..=5000).
pub fn list_entries(limit: u32) -> AppResult<Vec<InteractionEntry>> {
    let conn = open_db()?;
    let mut stmt = conn.prepare(
        "SELECT id, timestamp, actor, method_name, ui_target, trigger, params, result, message, stack_trace
         FROM interactions
         ORDER BY timestamp DESC, id DESC
         LIMIT ?1",
    )?;
    let rows = stmt.query_map(params![limit.clamp(1, 5000)], |row| {
        Ok(InteractionEntry {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            actor: row.get(2)?,
            method_name: row.get(3)?,
            ui_target: row.get(4)?,
            trigger: row.get(5)?,
            params: row.get(6)?,
            result: row.get(7)?,
            message: row.get(8)?,
            // NULL for rows recorded before the trace column existed.
            stack_trace: row.get(9)?,
        })
    })?;
    let mut entries = Vec::new();
    for row in rows {
        entries.push(row?);
    }
    Ok(entries)
}

/// Delete all recorded entries; returns the number of rows removed.
pub fn clear_entries() -> AppResult<u64> {
    let conn = open_db()?;
    let deleted = conn.execute("DELETE FROM interactions", [])?;
    Ok(deleted as u64)
}

/// Persist whether the UI interaction recorder is enabled across processes.
pub fn set_persisted_recording(enabled: bool) -> AppResult<()> {
    let conn = open_db()?;
    let val = if enabled { "1" } else { "0" };
    conn.execute(
        "INSERT INTO recorder_state (key, value) VALUES ('recording', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![val],
    )?;
    Ok(())
}

/// Read persisted recording state from SQLite (`None` when never set).
pub fn get_persisted_recording() -> AppResult<Option<bool>> {
    let conn = open_db()?;
    let mut stmt =
        conn.prepare("SELECT value FROM recorder_state WHERE key = 'recording' LIMIT 1")?;
    let mut rows = stmt.query([])?;
    if let Some(row) = rows.next()? {
        let val: String = row.get(0)?;
        return Ok(Some(val == "1" || val.eq_ignore_ascii_case("true")));
    }
    Ok(None)
}

/// Dump all entries to a timestamped JSON file in the audit dir; returns its path.
pub fn export_entries() -> AppResult<PathBuf> {
    let entries = list_entries(5000)?;
    let dir = audit_dir()?;
    let file_name = format!(
        "ui-audit-export-{}.json",
        chrono::Utc::now().timestamp_millis()
    );
    let path = dir.join(file_name);
    let json = serde_json::to_string_pretty(&entries)
        .map_err(|err| AppError::Unknown(format!("serialize ui audit export: {err}")))?;
    fs::write(&path, json).map_err(|err| io_with_context(&path, "write ui audit export", err))?;
    tracing::info!(
        target: "ui_audit",
        "exported {} ui audit entries to {:?}",
        entries.len(),
        path
    );
    Ok(path)
}
