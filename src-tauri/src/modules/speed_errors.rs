//! Speed Error SQLite Telemetry Logger (E1001-E9001).
use crate::modules::error_db;

/// Imp-27: Record a diagnostic error code into errors.db asynchronously
pub fn record_speed_error(
    error_code: &str,
    severity: &str,
    message: &str,
    source: &str,
    trigger_action: &str,
    metadata_json: Option<&str>,
) {
    let err_id = format!("err-{}", uuid::Uuid::new_v4().simple());
    let code = if error_code.is_empty() {
        "E1001"
    } else {
        error_code
    }
    .to_string();
    let sev = severity.to_string();
    let msg = message.to_string();
    let src = source.to_string();
    let act = trigger_action.to_string();
    let meta = metadata_json.unwrap_or("{}").to_string();
    let now = chrono::Utc::now().timestamp();

    tauri::async_runtime::spawn(async move {
        if let Ok(conn) = error_db::connect_error_db() {
            // Justification: best-effort telemetry logging into errors.db
            crate::error::record_ignored(
                conn.execute(
                    "INSERT INTO errors (id, error_code, severity, message, source, trigger_action, metadata_json, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    rusqlite::params![err_id, code, sev, msg, src, act, meta, now],
                ),
                "record_speed_error insert",
            );
        }
    });
}

/// Imp-77: Database Slow Query Telemetry
/// Records an E2002 diagnostic record into errors.db when a query exceeds threshold_ms.
pub fn record_slow_query(
    db_name: &str,
    query_preview: &str,
    elapsed_ms: u64,
    threshold_ms: u64,
) {
    if elapsed_ms >= threshold_ms {
        let meta = format!(
            r#"{{"database":"{}","elapsed_ms":{},"threshold_ms":{},"query":"{}"}}"#,
            db_name,
            elapsed_ms,
            threshold_ms,
            query_preview.replace('"', "\\\"")
        );
        record_speed_error(
            "E2002",
            "warn",
            &format!("Slow SQLite query on {db_name} ({}ms >= {}ms threshold)", elapsed_ms, threshold_ms),
            "sqlite::query",
            "slow_query_detector",
            Some(&meta),
        );
    }
}
