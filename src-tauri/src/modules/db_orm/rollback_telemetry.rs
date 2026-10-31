//! Transaction Rollback Telemetry Reporter.
//! Intercepts and records SQLite transaction aborts and explicit rollbacks,
//! emitting diagnostic telemetry records with error reason and callsite info.

use serde::{Deserialize, Serialize};
use tracing::warn;

/// Telemetry record emitted when a transaction fails or rolls back.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackTelemetryRecord {
    pub db_name: String,
    pub transaction_label: String,
    pub error_reason: String,
    pub callsite_file: &'static str,
    pub callsite_line: u32,
    pub timestamp_epoch_secs: u64,
}

/// Helper macro or function to capture rollback telemetry.
pub fn record_transaction_rollback(
    db_name: &str,
    label: &str,
    err_msg: &str,
    file: &'static str,
    line: u32,
) -> RollbackTelemetryRecord {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let record = RollbackTelemetryRecord {
        db_name: db_name.to_string(),
        transaction_label: label.to_string(),
        error_reason: err_msg.to_string(),
        callsite_file: file,
        callsite_line: line,
        timestamp_epoch_secs: now,
    };

    warn!(
        target: "db_orm::rollback",
        db = %record.db_name,
        label = %record.transaction_label,
        error = %record.error_reason,
        file = %record.callsite_file,
        line = record.callsite_line,
        "Database transaction rolled back"
    );

    record
}
