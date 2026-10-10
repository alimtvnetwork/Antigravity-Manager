use crate::proxy::monitor::ProxyRequestLog;
use rusqlite::{params, Connection, OpenFlags};

use super::*;

/// Cleanup old logs (keep last N days)
pub fn cleanup_old_logs(days: i64) -> Result<usize, String> {
    let conn = connect_db()?;

    // Note: Request log timestamp is stored in milliseconds (chrono::Utc::now().timestamp_millis())
    let cutoff_timestamp_ms = chrono::Utc::now().timestamp_millis() - (days * 24 * 3600 * 1000);

    let deleted = conn
        .execute(
            "DELETE FROM request_logs WHERE timestamp < ?1",
            [cutoff_timestamp_ms],
        )
        .map_err(|e| e.to_string())?;

    // Only execute VACUUM when substantial rows were deleted to avoid saturating disk I/O on startup
    if deleted >= 500 {
        if let Err(e) = conn.execute("VACUUM", []) {
            tracing::warn!("VACUUM failed after log cleanup: {}", e);
        }
    }

    Ok(deleted)
}

/// Limit maximum log count (keep newest N records)
#[allow(dead_code)]
pub fn limit_max_logs(max_count: usize) -> Result<usize, String> {
    let conn = connect_db()?;

    let deleted = conn
        .execute(
            "DELETE FROM request_logs WHERE id NOT IN (
            SELECT id FROM request_logs ORDER BY timestamp DESC LIMIT ?1
        )",
            [max_count],
        )
        .map_err(|e| e.to_string())?;

    // Only execute VACUUM when substantial rows were deleted
    if deleted >= 500 {
        if let Err(e) = conn.execute("VACUUM", []) {
            tracing::warn!("VACUUM failed after limit_max_logs: {}", e);
        }
    }

    Ok(deleted)
}

pub fn clear_logs() -> Result<(), String> {
    let _guard = LOG_WRITE_LOCK.lock().map_err(|e| e.to_string())?;
    let conn = connect_db()?;
    conn.execute("DELETE FROM request_logs", [])
        .map_err(|e| e.to_string())?;
    // Full vacuum to reclaim all disk space immediately
    // Justification: post-clear space reclamation; the logs were already deleted
    crate::error::record_ignored(
        conn.execute("VACUUM", []),
        "vacuum logs database after clear",
    );
    // Justification: WAL checkpoint is space hygiene; the logs were already deleted
    crate::error::record_ignored(
        conn.pragma_update(None, "wal_checkpoint", "TRUNCATE"),
        "checkpoint WAL after log clear",
    );
    Ok(())
}

/// Get total count of logs in database
pub fn get_logs_count() -> Result<u64, String> {
    let conn = connect_db()?;

    let count: u64 = conn
        .query_row("SELECT COUNT(*) FROM request_logs", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    Ok(count)
}

/// Get count of logs matching search filter
/// filter: search text to match in url, method, model, or status
/// errors_only: if true, only count logs with status < 200 or >= 400
pub fn get_logs_count_filtered(filter: &str, errors_only: bool) -> Result<u64, String> {
    let conn = connect_db()?;

    let filter_pattern = format!("%{}%", filter);

    let sql = if errors_only {
        "SELECT COUNT(*) FROM request_logs WHERE (status < 200 OR status >= 400)"
    } else if filter.is_empty() {
        "SELECT COUNT(*) FROM request_logs"
    } else {
        "SELECT COUNT(*) FROM request_logs WHERE
            (url LIKE ?1 OR method LIKE ?1 OR model LIKE ?1 OR CAST(status AS TEXT) LIKE ?1 OR account_email LIKE ?1)"
    };

    let count: u64 = if filter.is_empty() && !errors_only {
        conn.query_row(sql, [], |row| row.get(0))
    } else if errors_only {
        conn.query_row(sql, [], |row| row.get(0))
    } else {
        conn.query_row(sql, [&filter_pattern], |row| row.get(0))
    }
    .map_err(|e| e.to_string())?;

    Ok(count)
}

/// Get logs with search filter and pagination
/// filter: search text to match in url, method, model, or status
/// errors_only: if true, only return logs with status < 200 or >= 400
pub fn get_logs_filtered(
    filter: &str,
    errors_only: bool,
    limit: usize,
    offset: usize,
) -> Result<Vec<ProxyRequestLog>, String> {
    let conn = connect_db()?;

    let filter_pattern = format!("%{}%", filter);

    let sql = if errors_only {
        "SELECT id, timestamp, method, url, status, duration, model, substr(error, 1, 1024),
                NULL as request_body, NULL as upstream_request_body, NULL as response_body,
                input_tokens, output_tokens, cached_tokens, account_email, mapped_model, protocol, client_ip, username,
                NULL as request_headers, NULL as upstream_request_headers, NULL as response_headers,
                session_id
         FROM request_logs
         WHERE (status < 200 OR status >= 400)
         ORDER BY timestamp DESC
         LIMIT ?1 OFFSET ?2"
    } else if filter.is_empty() {
        "SELECT id, timestamp, method, url, status, duration, model, substr(error, 1, 1024),
                NULL as request_body, NULL as upstream_request_body, NULL as response_body,
                input_tokens, output_tokens, cached_tokens, account_email, mapped_model, protocol, client_ip, username,
                NULL as request_headers, NULL as upstream_request_headers, NULL as response_headers,
                session_id
         FROM request_logs
         ORDER BY timestamp DESC
         LIMIT ?1 OFFSET ?2"
    } else {
        "SELECT id, timestamp, method, url, status, duration, model, substr(error, 1, 1024),
                NULL as request_body, NULL as upstream_request_body, NULL as response_body,
                input_tokens, output_tokens, cached_tokens, account_email, mapped_model, protocol, client_ip, username,
                NULL as request_headers, NULL as upstream_request_headers, NULL as response_headers,
                session_id
         FROM request_logs
         WHERE (url LIKE ?3 OR method LIKE ?3 OR model LIKE ?3 OR CAST(status AS TEXT) LIKE ?3 OR account_email LIKE ?3 OR client_ip LIKE ?3 OR session_id LIKE ?3)
         ORDER BY timestamp DESC
         LIMIT ?1 OFFSET ?2"
    };

    let logs: Vec<ProxyRequestLog> = if filter.is_empty() && !errors_only {
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        let logs_iter = stmt
            .query_map([limit, offset], map_request_log_row)
            .map_err(|e| e.to_string())?;
        logs_iter.filter_map(|r| r.ok()).collect()
    } else if errors_only {
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        let logs_iter = stmt
            .query_map([limit, offset], map_request_log_row)
            .map_err(|e| e.to_string())?;
        logs_iter.filter_map(|r| r.ok()).collect()
    } else {
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        let logs_iter = stmt
            .query_map(
                rusqlite::params![limit, offset, filter_pattern],
                map_request_log_row,
            )
            .map_err(|e| e.to_string())?;
        logs_iter.filter_map(|r| r.ok()).collect()
    };

    Ok(logs)
}

/// Get all logs with full details for export
pub fn get_all_logs_for_export() -> Result<Vec<ProxyRequestLog>, String> {
    let conn = connect_db()?;

    let mut stmt = conn
        .prepare(
            "SELECT id, timestamp, method, url, status, duration, model, error,
                request_body, upstream_request_body, response_body, input_tokens, output_tokens,
                cached_tokens, account_email, mapped_model, protocol, client_ip, username,
                request_headers, upstream_request_headers, response_headers,
                session_id
         FROM request_logs
         ORDER BY timestamp DESC",
        )
        .map_err(|e| e.to_string())?;

    let logs_iter = stmt
        .query_map([], map_request_log_row)
        .map_err(|e| e.to_string())?;

    let mut logs = Vec::new();
    for log in logs_iter {
        logs.push(log.map_err(|e| e.to_string())?);
    }
    Ok(logs)
}
