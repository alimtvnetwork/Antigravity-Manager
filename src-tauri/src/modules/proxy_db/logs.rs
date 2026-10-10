use crate::proxy::config::LogRetentionConfig;
use crate::proxy::monitor::ProxyRequestLog;
use rusqlite::{params, Connection, OpenFlags};
use std::io::{Read, Write};

use super::*;

pub fn save_log(log: ProxyRequestLog) -> Result<(), String> {
    let _guard = LOG_WRITE_LOCK.lock().map_err(|e| e.to_string())?;
    // Read the file for every admitted write, including after a runtime budget change.
    let policy = crate::modules::config::load_app_config()?
        .proxy
        .log_retention;
    let conn = connect_db()?;
    save_log_with_connection(&conn, log, &policy)
}

pub(crate) fn save_log_with_connection(
    conn: &Connection,
    mut log: ProxyRequestLog,
    policy: &LogRetentionConfig,
) -> Result<(), String> {
    conn.busy_timeout(std::time::Duration::from_millis(250))
        .map_err(|e| e.to_string())?;
    log.error = log
        .error
        .as_ref()
        .map(|error| error.chars().take(1024).collect());
    let budget = policy.budget_bytes();
    let summary_bytes = [&log.id, &log.method, &log.url]
        .iter()
        .map(|s| s.len() as u64)
        .sum::<u64>()
        + [
            &log.model,
            &log.mapped_model,
            &log.account_email,
            &log.client_ip,
            &log.error,
            &log.protocol,
            &log.username,
        ]
        .iter()
        .filter_map(|s| s.as_ref())
        .map(|s| s.len() as u64)
        .sum::<u64>()
        + 1024;
    let body_bytes = [
        &log.request_body,
        &log.upstream_request_body,
        &log.response_body,
        &log.request_headers,
        &log.upstream_request_headers,
        &log.response_headers,
    ]
    .iter()
    .filter_map(|s| s.as_ref())
    .map(|s| s.len() as u64)
    .sum::<u64>();
    let mut log_bytes = summary_bytes.saturating_add(body_bytes);
    if log_bytes.saturating_mul(3).saturating_add(64 * 1024) > budget / 5 * 4 {
        log.request_body = None;
        log.upstream_request_body = None;
        log.response_body = None;
        log.request_headers = None;
        log.upstream_request_headers = None;
        log.response_headers = None;
        log_bytes = summary_bytes;
    }
    if log_bytes.saturating_mul(3).saturating_add(64 * 1024) > budget {
        return Err("proxy log summary exceeds disk budget".to_string());
    }
    make_room(conn, budget, log_bytes)?;

    conn.execute(
        "INSERT INTO request_logs (id, timestamp, method, url, status, duration, model, error, request_body, upstream_request_body, response_body, input_tokens, output_tokens, cached_tokens, account_email, mapped_model, protocol, client_ip, username, request_headers, upstream_request_headers, response_headers, session_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23)",
        params![
            log.id,
            log.timestamp,
            log.method,
            log.url,
            log.status,
            log.duration,
            log.model,
            log.error,
            log.request_body,
            log.upstream_request_body,
            log.response_body,
            log.input_tokens,
            log.output_tokens,
            log.cached_tokens,
            log.account_email,
            log.mapped_model,
            log.protocol,
            log.client_ip,
            log.username,
            log.request_headers,
            log.upstream_request_headers,
            log.response_headers,
            log.session_id,
        ],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

/// Get logs summary (without large request_body and response_body fields) with pagination
pub fn get_logs_summary(limit: usize, offset: usize) -> Result<Vec<ProxyRequestLog>, String> {
    let conn = connect_db()?;

    let mut stmt = conn
        .prepare(
            "SELECT id, timestamp, method, url, status, duration, model, substr(error, 1, 1024),
                NULL as request_body, NULL as upstream_request_body, NULL as response_body,
                input_tokens, output_tokens, cached_tokens, account_email, mapped_model, protocol, client_ip, username,
                NULL as request_headers, NULL as upstream_request_headers, NULL as response_headers,
                session_id
         FROM request_logs 
         ORDER BY timestamp DESC 
         LIMIT ?1 OFFSET ?2",
        )
        .map_err(|e| e.to_string())?;

    let logs_iter = stmt
        .query_map([limit, offset], map_request_log_row)
        .map_err(|e| e.to_string())?;

    let mut logs = Vec::new();
    for log in logs_iter {
        logs.push(log.map_err(|e| e.to_string())?);
    }
    Ok(logs)
}

/// Get logs (backward compatible, calls get_logs_summary)
pub fn get_logs(limit: usize) -> Result<Vec<ProxyRequestLog>, String> {
    get_logs_summary(limit, 0)
}

pub fn get_stats() -> Result<crate::proxy::monitor::ProxyStats, String> {
    let conn = connect_db()?;

    // Optimized: Use single query instead of three separate queries
    // Use COALESCE to handle NULL values when table is empty (SUM returns NULL for empty set)
    let (total_requests, success_count, error_count): (u64, u64, u64) = conn
        .query_row(
            "SELECT
            COUNT(*) as total,
            COALESCE(SUM(CASE WHEN status >= 200 AND status < 400 THEN 1 ELSE 0 END), 0) as success,
            COALESCE(SUM(CASE WHEN status < 200 OR status >= 400 THEN 1 ELSE 0 END), 0) as error
         FROM request_logs",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|e| e.to_string())?;

    Ok(crate::proxy::monitor::ProxyStats {
        total_requests,
        success_count,
        error_count,
    })
}

/// Get single log detail (with request_body and response_body)
pub fn get_log_detail(log_id: &str) -> Result<ProxyRequestLog, String> {
    let conn = connect_db()?;

    let mut stmt = conn
        .prepare(
            "SELECT id, timestamp, method, url, status, duration, model, error,
                request_body, upstream_request_body, response_body, input_tokens, output_tokens,
                cached_tokens, account_email, mapped_model, protocol, client_ip, username,
                request_headers, upstream_request_headers, response_headers,
                session_id
         FROM request_logs
         WHERE id = ?1",
        )
        .map_err(|e| e.to_string())?;

    stmt.query_row([log_id], map_request_log_row)
        .map_err(|e| e.to_string())
}
