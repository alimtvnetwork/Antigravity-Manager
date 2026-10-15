//! In-memory recording state plus orchestration of audit DB writes.

use chrono::Utc;
use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::error::{AppError, AppResult};

use super::storage;
use super::types::{
    format_ui_target, InteractionEntry, RecordInteractionInput, RecordInteractionOutcome,
};

static RECORDING: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(false));

/// Start recording UI interactions; returns the previous recording state.
/// Fails fast when the audit DB is unreachable so records cannot be silently dropped.
pub fn start_recording() -> AppResult<bool> {
    storage::ensure_db()?;
    let was = is_recording();
    storage::set_persisted_recording(true)?;
    RECORDING.store(true, Ordering::SeqCst);
    tracing::info!(
        target: "ui_audit",
        "UI audit recording started (was_recording={})",
        was
    );
    Ok(was)
}

/// Stop recording UI interactions; returns the previous recording state.
pub fn stop_recording() -> bool {
    let was = is_recording();
    // Justification: best-effort persist of recording state on stop; in-memory flag is always updated.
    crate::error::record_ignored(
        storage::set_persisted_recording(false),
        "persist ui audit recording=false",
    );
    RECORDING.store(false, Ordering::SeqCst);
    tracing::info!(
        target: "ui_audit",
        "UI audit recording stopped (was_recording={})",
        was
    );
    was
}

/// Whether the recorder is currently capturing interactions.
/// Checks both in-memory atomic flag and persisted SQLite state so CLI and GUI stay synchronized.
pub fn is_recording() -> bool {
    if let Ok(Some(persisted)) = storage::get_persisted_recording() {
        RECORDING.store(persisted, Ordering::SeqCst);
        return persisted;
    }
    RECORDING.load(Ordering::SeqCst)
}

/// The `result` field accepts only "ok" or "error".
fn validate_result(result: &str) -> AppResult<()> {
    if result == "ok" || result == "error" {
        Ok(())
    } else {
        Err(AppError::Unknown(format!(
            "invalid ui audit result '{result}': expected \"ok\" or \"error\""
        )))
    }
}

/// Record one interaction when recording is enabled. When recording is off
/// this is a logged no-op (not an error) and `recorded` is false.
pub fn record_interaction(input: RecordInteractionInput) -> AppResult<RecordInteractionOutcome> {
    if !is_recording() {
        tracing::debug!(
            target: "ui_audit",
            "record_ui_interaction skipped: recorder is stopped"
        );
        return Ok(RecordInteractionOutcome {
            recorded: false,
            id: None,
        });
    }
    let result = input.result.trim().to_lowercase();
    validate_result(&result)?;
    let method_name = input.method_name.trim();
    if method_name.is_empty() {
        return Err(AppError::Unknown(
            "ui audit record requires a non-empty method_name".to_string(),
        ));
    }
    let route = input.route.trim();
    if route.is_empty() {
        return Err(AppError::Unknown(
            "ui audit record requires a non-empty route".to_string(),
        ));
    }
    let ui_target = format_ui_target(
        route,
        input.component.as_deref(),
        input.element_id.as_deref(),
    );
    let params = input
        .params
        .map(|v| v.to_string())
        .unwrap_or("{}".to_string());
    let message = input.message.unwrap_or_default();
    let id = storage::append_entry(
        Utc::now().timestamp_millis(),
        input.actor.as_str(),
        method_name,
        &ui_target,
        input.trigger.as_str(),
        &params,
        &result,
        &message,
    )?;
    tracing::debug!(
        target: "ui_audit",
        "recorded interaction id={} actor={} method={} target={}",
        id,
        input.actor.as_str(),
        method_name,
        ui_target
    );
    Ok(RecordInteractionOutcome {
        recorded: true,
        id: Some(id),
    })
}

/// Remove all recorded entries; returns the number of rows deleted.
pub fn clear_recording() -> AppResult<u64> {
    storage::clear_entries()
}

/// Dump all recorded entries to a JSON file; returns the file path.
pub fn export_recording() -> AppResult<PathBuf> {
    storage::export_entries()
}

/// List the newest recorded entries, capped at `limit`.
pub fn list_recorded(limit: u32) -> AppResult<Vec<InteractionEntry>> {
    storage::list_entries(limit)
}

/// Convenience helper for CLI / backend callers: appends an entry directly to `ui_audit.db`
/// when the recorder is active.
pub fn record_if_active(
    actor: &str,
    trigger: &str,
    method_name: &str,
    ui_target: &str,
    params: serde_json::Value,
    result: &str,
    message: &str,
) {
    if !is_recording() {
        return;
    }
    // Justification: best-effort audit recording from CLI/backend paths; must not abort primary operation.
    crate::error::record_ignored(
        storage::append_entry(
            Utc::now().timestamp_millis(),
            actor,
            method_name,
            ui_target,
            trigger,
            &params.to_string(),
            result,
            message,
        ),
        "record_if_active ui_audit",
    );
}
