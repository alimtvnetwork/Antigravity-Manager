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

/// Keep a location that starts at `src/` or `src-tauri/` so the audit row
/// never stores a machine path.
pub(crate) fn relative_frame_location(raw: &str) -> String {
    let normalized = raw.replace('\\', "/");
    for marker in ["src-tauri/src/", "src/"] {
        if let Some(index) = normalized.find(marker) {
            return normalized[index..].to_string();
        }
    }
    normalized
}

/// Parse a `Backtrace` display into the first `limit` function + file pairs,
/// skipping the audit recorder itself.
pub(crate) fn caller_frames_from_backtrace(text: &str, limit: usize) -> Vec<(String, String)> {
    let mut frames = Vec::new();
    let mut pending_function: Option<String> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(location) = trimmed.strip_prefix("at ") {
            let Some(function_name) = pending_function.take() else {
                continue;
            };
            if function_name.contains("audit::recorder")
                || function_name.contains("record_ignored")
                || function_name.contains("std::backtrace")
                || function_name.contains("backtrace::")
            {
                continue;
            }
            frames.push((function_name, relative_frame_location(location)));
            if frames.len() == limit {
                break;
            }
            continue;
        }
        if trimmed.contains("::") {
            let function_name = trimmed
                .split_once(':')
                .map(|(_, rest)| rest.trim())
                .unwrap_or(trimmed);
            pending_function = Some(function_name.to_string());
        }
    }
    frames
}

fn attach_caller_frames(params: Option<serde_json::Value>) -> String {
    let mut value = params.unwrap_or_else(|| serde_json::json!({}));
    let captured = std::backtrace::Backtrace::force_capture().to_string();
    let frames = caller_frames_from_backtrace(&captured, 2);
    if let Some(object) = value.as_object_mut() {
        let already_has_frames = object
            .get("stack_frames")
            .and_then(|value| value.as_array())
            .is_some_and(|frames| !frames.is_empty());
        if already_has_frames {
            return value.to_string();
        }
        let listed: Vec<serde_json::Value> = frames
            .into_iter()
            .map(|(function_name, location)| {
                serde_json::json!({
                    "function": function_name,
                    "location": location,
                })
            })
            .collect();
        object.insert("stack_frames".to_string(), serde_json::Value::Array(listed));
    }
    value.to_string()
}

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
    let params = attach_caller_frames(input.params);
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
            &attach_caller_frames(Some(params)),
            result,
            message,
        ),
        "record_if_active ui_audit",
    );
}

#[cfg(test)]
mod tests {
    use super::{caller_frames_from_backtrace, relative_frame_location};

    #[test]
    fn caller_frames_keep_two_locations_and_skip_the_recorder() {
        let sample = "\
   0: agm_ui::modules::audit::recorder::record_interaction\n\
             at D:/work/antigravity-manager/src-tauri/src/modules/audit/recorder.rs:120:5\n\
   1: agm_ui::proxy::server::admin_ui::handle_element_action\n\
             at D:/work/antigravity-manager/src-tauri/src/proxy/server/admin_ui.rs:215:9\n\
   2: agm_ui::proxy::server::admin_ui::click_ui\n\
             at D:/work/antigravity-manager/src-tauri/src/proxy/server/admin_ui.rs:260:5\n";
        let frames = caller_frames_from_backtrace(sample, 2);
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].1, "src-tauri/src/proxy/server/admin_ui.rs:215:9");
        assert_eq!(frames[1].0, "agm_ui::proxy::server::admin_ui::click_ui");
        assert_eq!(
            relative_frame_location(r"C:\repo\src\hooks\useAuditRecorder.ts:49"),
            "src/hooks/useAuditRecorder.ts:49"
        );
    }
}
