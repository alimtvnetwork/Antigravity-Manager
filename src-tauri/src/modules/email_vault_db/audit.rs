use base64::prelude::*;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};

use super::*;

// ---------------------------------------------------------------------------
// Inbound Audit Log & Replay Guard
// ---------------------------------------------------------------------------

/// Record an inbound command in audit log
pub fn record_inbound_audit_log(entry: EmailInboundAuditLog) -> Result<(), String> {
    let conn = connect_vault_db()?;
    conn.execute(
        "INSERT INTO email_inbound_audit_log 
         (id, message_id, sender_email, subject, action_type, action_payload, execution_status, execution_result, received_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            &entry.id,
            &entry.message_id,
            &entry.sender_email,
            &entry.subject,
            &entry.action_type,
            &entry.action_payload,
            &entry.execution_status,
            &entry.execution_result,
            entry.received_at,
        ],
    )
    .map_err(|e| format!("Failed to record audit log: {}", e))?;
    Ok(())
}

/// Check if message_id has already been processed to prevent replay
pub fn is_message_already_processed(message_id: &str) -> Result<bool, String> {
    let conn = connect_vault_db()?;
    let mut stmt = conn
        .prepare("SELECT COUNT(1) FROM email_inbound_audit_log WHERE message_id = ?")
        .map_err(|e| format!("Failed to check message replay: {}", e))?;

    let count: i64 = stmt
        .query_row(params![message_id], |r| r.get(0))
        .unwrap_or(0);

    let has_seen = count > 0;
    Ok(has_seen)
}

/// Check if a duplicate action from this sender was successfully executed within the rate limit window (e.g. 600s / 10 min)
pub fn is_rate_limited_in_sqlite(
    sender: &str,
    action_type: &str,
    window_seconds: i64,
) -> Result<bool, String> {
    let conn = connect_vault_db()?;
    let now = chrono::Utc::now().timestamp();
    let cutoff = now - window_seconds;
    let mut stmt = conn
        .prepare(
            "SELECT COUNT(1) FROM email_inbound_audit_log \
             WHERE sender_email = ?1 AND action_type = ?2 AND received_at > ?3 \
             AND execution_status = 'success'",
        )
        .map_err(|e| format!("Failed to check rate limit: {}", e))?;

    let count: i64 = stmt
        .query_row(params![sender, action_type, cutoff], |r| r.get(0))
        .unwrap_or(0);

    Ok(count > 0)
}
