// High-Speed Split-DB Switch Audit Logger (Imp-491)
// Emits structured audit records during account rotation events, capturing
// rationale codes, settling latency, and outcome verification.

use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchAuditEntry {
    pub instance_id: String,
    pub previous_account_id: String,
    pub target_account_id: String,
    pub trigger_reason: String,
    pub settling_duration_ms: u64,
    pub success: bool,
    pub error_message: Option<String>,
    pub timestamp_epoch_secs: u64,
}

pub struct SwitchAuditLogger;

impl SwitchAuditLogger {
    /// Formats and records an audit entry to the active tracing channel.
    pub fn log_switch_event(entry: &SwitchAuditEntry) {
        if entry.success {
            info!(
                instance_id = %entry.instance_id,
                prev_acc = %entry.previous_account_id,
                target_acc = %entry.target_account_id,
                reason = %entry.trigger_reason,
                settling_ms = entry.settling_duration_ms,
                "Auto-switch executed successfully"
            );
        } else {
            tracing::warn!(
                instance_id = %entry.instance_id,
                prev_acc = %entry.previous_account_id,
                target_acc = %entry.target_account_id,
                reason = %entry.trigger_reason,
                settling_ms = entry.settling_duration_ms,
                error = ?entry.error_message,
                "Auto-switch execution encountered a fault"
            );
        }
    }

    /// Converts the audit entry to an SQLite-compatible parameter tuple.
    pub fn to_sql_params(entry: &SwitchAuditEntry) -> (String, String, String, String, i64, i32, Option<String>, i64) {
        (
            entry.instance_id.clone(),
            entry.previous_account_id.clone(),
            entry.target_account_id.clone(),
            entry.trigger_reason.clone(),
            entry.settling_duration_ms as i64,
            if entry.success { 1 } else { 0 },
            entry.error_message.clone(),
            entry.timestamp_epoch_secs as i64,
        )
    }
}
