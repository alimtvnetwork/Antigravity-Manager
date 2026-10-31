//! REST API Workbench Request History Subsystem.
//! Persists executed try-it-out API requests with response codes, timings, and payloads.

use chrono::Utc;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestHistoryEntry {
    pub id: Option<i64>,
    pub method: String,
    pub path: String,
    pub headers_json: Option<String>,
    pub body_json: Option<String>,
    pub status_code: u16,
    pub duration_ms: u64,
    pub created_at: String,
}

/// Ensures the REST execution history table exists in SQLite.
pub fn ensure_rest_history_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS rest_request_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            method TEXT NOT NULL,
            path TEXT NOT NULL,
            headers_json TEXT,
            body_json TEXT,
            status_code INTEGER NOT NULL,
            duration_ms INTEGER NOT NULL,
            created_at TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_rest_history_created_at 
         ON rest_request_history (created_at ASC, id ASC)",
        [],
    )?;

    Ok(())
}

/// Records a completed REST API execution into SQLite.
pub fn record_rest_request(conn: &Connection, entry: &RestHistoryEntry) -> Result<i64> {
    ensure_rest_history_table(conn)?;
    let created_at = if entry.created_at.trim().is_empty() {
        Utc::now().to_rfc3339()
    } else {
        entry.created_at.clone()
    };

    conn.execute(
        "INSERT INTO rest_request_history 
         (method, path, headers_json, body_json, status_code, duration_ms, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            entry.method.to_uppercase(),
            entry.path,
            entry.headers_json,
            entry.body_json,
            entry.status_code as i64,
            entry.duration_ms as i64,
            created_at
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

/// Lists recent REST execution history records adhering to FIFO / chronological ordering.
pub fn list_rest_history(conn: &Connection, limit: usize) -> Result<Vec<RestHistoryEntry>> {
    ensure_rest_history_table(conn)?;
    let bound_limit = if limit == 0 { 50 } else { limit.min(500) };

    let mut stmt = conn.prepare(
        "SELECT id, method, path, headers_json, body_json, status_code, duration_ms, created_at
         FROM rest_request_history
         ORDER BY id DESC
         LIMIT ?1",
    )?;

    let rows = stmt.query_map(params![bound_limit as i64], |row| {
        Ok(RestHistoryEntry {
            id: Some(row.get(0)?),
            method: row.get(1)?,
            path: row.get(2)?,
            headers_json: row.get(3)?,
            body_json: row.get(4)?,
            status_code: row.get::<_, i64>(5)? as u16,
            duration_ms: row.get::<_, i64>(6)? as u64,
            created_at: row.get(7)?,
        })
    })?;

    let mut list = Vec::new();
    for r in rows {
        list.push(r?);
    }
    Ok(list)
}

/// Clears all request history rows.
pub fn clear_rest_history(conn: &Connection) -> Result<usize> {
    ensure_rest_history_table(conn)?;
    conn.execute("DELETE FROM rest_request_history", [])
}
