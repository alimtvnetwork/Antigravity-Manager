//! CRUD operations for `errors.db`.

use super::db::connect_error_db;
use super::types::{ErrorQueryOptions, ErrorRecord, NewErrorRecord};
use rusqlite::params;
use uuid::Uuid;

/// Insert a new error record into `errors.db`
pub fn record_error(entry: &NewErrorRecord) -> Result<String, String> {
    let conn = connect_error_db()?;

    let id = entry
        .id
        .clone()
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let err_code = if entry.error_code.trim().is_empty() {
        "E9001"
    } else {
        entry.error_code.trim()
    };
    let severity = if entry.severity.trim().is_empty() {
        "error".to_string()
    } else {
        entry.severity.trim().to_lowercase()
    };
    let created_at = entry
        .created_at
        .unwrap_or_else(|| chrono::Utc::now().timestamp_millis());

    conn.execute(
        "INSERT INTO errors (
            id, error_code, severity, message, stack_trace, source, trigger_action, metadata_json, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            id,
            err_code,
            severity,
            entry.message,
            entry.stack_trace,
            entry.source,
            entry.trigger_action,
            entry.metadata_json,
            created_at,
        ],
    )
    .map_err(|e| format!("Failed to insert error record: {e}"))?;

    Ok(id)
}

/// Query errors with flexible options (severity, error_code, limit, offset, search)
pub fn list_errors(options: &ErrorQueryOptions) -> Result<Vec<ErrorRecord>, String> {
    let conn = connect_error_db()?;

    let limit = options.limit.unwrap_or(50).clamp(1, 1000);
    let offset = options.offset.unwrap_or(0);

    let mut query = String::from(
        "SELECT id, error_code, severity, message, stack_trace, source, trigger_action, metadata_json, created_at
         FROM errors WHERE 1=1",
    );
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(ref sev) = options.severity {
        if !sev.trim().is_empty() {
            query.push_str(" AND LOWER(severity) = LOWER(?");
            params_vec.push(Box::new(sev.trim().to_string()));
            query.push_str(&format!("{})", params_vec.len()));
        }
    }

    if let Some(ref code) = options.error_code {
        if !code.trim().is_empty() {
            query.push_str(" AND UPPER(error_code) = UPPER(?");
            params_vec.push(Box::new(code.trim().to_string()));
            query.push_str(&format!("{})", params_vec.len()));
        }
    }

    if let Some(ref s) = options.search {
        if !s.trim().is_empty() {
            let pattern = format!("%{}%", s.trim());
            query.push_str(" AND (message LIKE ?");
            params_vec.push(Box::new(pattern.clone()));
            let p1 = params_vec.len();

            query.push_str(&format!(
                "{p1} OR source LIKE ?{0} OR trigger_action LIKE ?{0})",
                p1 + 1
            ));
            params_vec.push(Box::new(pattern));
        }
    }

    query.push_str(" ORDER BY created_at DESC, id DESC LIMIT ?");
    params_vec.push(Box::new(limit as i64));
    let limit_idx = params_vec.len();

    query.push_str(&format!("{limit_idx} OFFSET ?"));
    params_vec.push(Box::new(offset as i64));
    let offset_idx = params_vec.len();
    query.push_str(&format!("{offset_idx}"));

    let mut stmt = conn
        .prepare(&query)
        .map_err(|e| format!("Failed to prepare list errors statement: {e}"))?;

    let rusqlite_params: Vec<&dyn rusqlite::ToSql> =
        params_vec.iter().map(|b| b.as_ref()).collect();

    let rows = stmt
        .query_map(rusqlite_params.as_slice(), |row| {
            Ok(ErrorRecord {
                id: row.get(0)?,
                error_code: row.get(1)?,
                severity: row.get(2)?,
                message: row.get(3)?,
                stack_trace: row.get(4)?,
                source: row.get(5)?,
                trigger_action: row.get(6)?,
                metadata_json: row.get(7)?,
                created_at: row.get(8)?,
            })
        })
        .map_err(|e| format!("Failed to execute list errors query: {e}"))?;

    let mut records = Vec::new();
    for r in rows {
        records.push(r.map_err(|e| format!("Row mapping error: {e}"))?);
    }

    Ok(records)
}

/// Count recorded errors, optionally filtered by severity
pub fn count_errors(severity: Option<&str>) -> Result<i64, String> {
    let conn = connect_error_db()?;

    let (sql, params_vec): (String, Vec<Box<dyn rusqlite::ToSql>>) = match severity {
        Some(sev) if !sev.trim().is_empty() => (
            "SELECT COUNT(*) FROM errors WHERE LOWER(severity) = LOWER(?1)".to_string(),
            vec![Box::new(sev.trim().to_lowercase())],
        ),
        _ => ("SELECT COUNT(*) FROM errors".to_string(), Vec::new()),
    };

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("Failed to prepare count errors statement: {e}"))?;

    let rusqlite_params: Vec<&dyn rusqlite::ToSql> =
        params_vec.iter().map(|b| b.as_ref()).collect();

    let count: i64 = stmt
        .query_row(rusqlite_params.as_slice(), |row| row.get(0))
        .map_err(|e| format!("Failed to execute count query: {e}"))?;

    Ok(count)
}

/// Retrieve single error record by ID
pub fn get_error_by_id(id: &str) -> Result<Option<ErrorRecord>, String> {
    let conn = connect_error_db()?;

    let mut stmt = conn
        .prepare(
            "SELECT id, error_code, severity, message, stack_trace, source, trigger_action, metadata_json, created_at
             FROM errors WHERE id = ?1 LIMIT 1",
        )
        .map_err(|e| format!("Failed to prepare get_error_by_id statement: {e}"))?;

    let mut rows = stmt
        .query_map(params![id], |row| {
            Ok(ErrorRecord {
                id: row.get(0)?,
                error_code: row.get(1)?,
                severity: row.get(2)?,
                message: row.get(3)?,
                stack_trace: row.get(4)?,
                source: row.get(5)?,
                trigger_action: row.get(6)?,
                metadata_json: row.get(7)?,
                created_at: row.get(8)?,
            })
        })
        .map_err(|e| format!("Failed to query error by id: {e}"))?;

    if let Some(row_res) = rows.next() {
        let record = row_res.map_err(|e| format!("Row mapping error: {e}"))?;
        Ok(Some(record))
    } else {
        Ok(None)
    }
}

/// Clear all errors from `errors.db`
pub fn clear_errors() -> Result<usize, String> {
    let conn = connect_error_db()?;

    let rows_affected = conn
        .execute("DELETE FROM errors", [])
        .map_err(|e| format!("Failed to clear errors from database: {e}"))?;

    // Optimize database file after purge
    // Justification: best-effort vacuum; harmless if locked or skipped
    crate::error::record_ignored(conn.execute("VACUUM", []), "vacuum_errors_db");

    Ok(rows_affected)
}

/// Prune older errors beyond a certain retention limit (FIFO)
pub fn prune_old_errors(keep_count: usize) -> Result<usize, String> {
    let conn = connect_error_db()?;

    let rows_affected = conn
        .execute(
            "DELETE FROM errors WHERE id NOT IN (
                SELECT id FROM errors ORDER BY created_at DESC, id DESC LIMIT ?1
            )",
            params![keep_count as i64],
        )
        .map_err(|e| format!("Failed to prune old errors: {e}"))?;

    Ok(rows_affected)
}
