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

/// Maximum stack frames stored per audit entry.
///
/// Cost rationale: `std::backtrace::Backtrace::force_capture()` already walks
/// the full stack once per recorded interaction (symbol resolution is the
/// dominant cost); parsing its textual dump is linear and bounded by this
/// limit, so keeping more frames adds only a small `Vec` — no extra syscalls.
/// 8 frames cover the whole UI -> recorder path while keeping rows compact.
const STACK_TRACE_FRAME_LIMIT: usize = 8;

/// The params column keeps just the two innermost frames for compactness;
/// the full bounded trace lives in the dedicated `stack_trace` column.
const PARAMS_FRAME_LIMIT: usize = 2;

/// Serialize one `{function, location}` frame, matching the shape the
/// frontend recorder already sends as `stack_frames`.
fn frame_json(function_name: &str, location: &str) -> serde_json::Value {
    serde_json::json!({
        "function": function_name,
        "location": location,
    })
}

/// Capture the recording code path's caller frames once per interaction,
/// skipping the audit recorder itself. Locations stay repo-relative so rows
/// never store machine paths.
fn capture_caller_frames() -> Vec<(String, String)> {
    let captured = std::backtrace::Backtrace::force_capture().to_string();
    caller_frames_from_backtrace(&captured, STACK_TRACE_FRAME_LIMIT)
}

fn attach_caller_frames(params: Option<serde_json::Value>, frames: &[(String, String)]) -> String {
    let mut value = params.unwrap_or_else(|| serde_json::json!({}));
    if let Some(object) = value.as_object_mut() {
        let already_has_frames = object
            .get("stack_frames")
            .and_then(|value| value.as_array())
            .is_some_and(|frames| !frames.is_empty());
        if already_has_frames {
            return value.to_string();
        }
        let listed: Vec<serde_json::Value> = frames
            .iter()
            .take(PARAMS_FRAME_LIMIT)
            .map(|(function_name, location)| frame_json(function_name, location))
            .collect();
        object.insert("stack_frames".to_string(), serde_json::Value::Array(listed));
    }
    value.to_string()
}

/// Read a caller-supplied trace out of the params payload (the frontend
/// recorder already sends `stack_frames`, plus the `stackTrace`/`stackFrames`
/// aliases) and normalize it into frame JSON. Returns an empty vec when the
/// payload carries no usable frames, letting the caller fall back to the
/// backend capture.
fn frontend_frames_from_params(params: &serde_json::Value) -> Vec<serde_json::Value> {
    let mut frames = Vec::new();
    let candidate = params
        .get("stack_frames")
        .or_else(|| params.get("stackFrames"))
        .or_else(|| params.get("stackTrace"));
    let Some(array) = candidate.and_then(|value| value.as_array()) else {
        return frames;
    };
    for item in array.iter().take(STACK_TRACE_FRAME_LIMIT) {
        let function = item
            .get("function")
            .or_else(|| item.get("functionName"))
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .trim();
        let location = item
            .get("location")
            .or_else(|| item.get("file"))
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .trim();
        if function.is_empty() && location.is_empty() {
            continue;
        }
        frames.push(frame_json(
            if function.is_empty() {
                "unknown"
            } else {
                function
            },
            if location.is_empty() {
                "unknown"
            } else {
                location
            },
        ));
    }
    frames
}

/// Resolve the trace stored in the dedicated `stack_trace` column: prefer the
/// caller-supplied frontend trace (it names the actual UI call path) and fall
/// back to the backend's own captured frames. Returns `None` when no usable
/// frame exists so the row stores NULL ("unavailable" in the panel).
fn resolve_stack_trace(
    params: &serde_json::Value,
    backend_frames: &[(String, String)],
) -> Option<String> {
    let supplied = frontend_frames_from_params(params);
    let frames: Vec<serde_json::Value> = if supplied.is_empty() {
        backend_frames
            .iter()
            .map(|(function_name, location)| frame_json(function_name, location))
            .collect()
    } else {
        supplied
    };
    if frames.is_empty() {
        return None;
    }
    match serde_json::to_string(&frames) {
        Ok(json) => Some(json),
        // Justification: trace serialization is best-effort; a failure must
        // not abort the audit record it describes.
        Err(err) => {
            crate::error::record_ignored(
                Err::<(), serde_json::Error>(err),
                "serialize audit stack trace",
            );
            None
        }
    }
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
    let caller_frames = capture_caller_frames();
    let params_value = input.params.unwrap_or_else(|| serde_json::json!({}));
    let stack_trace = resolve_stack_trace(&params_value, &caller_frames);
    let params = attach_caller_frames(Some(params_value), &caller_frames);
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
        stack_trace.as_deref(),
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
    let caller_frames = capture_caller_frames();
    let stack_trace = resolve_stack_trace(&params, &caller_frames);
    // Justification: best-effort audit recording from CLI/backend paths; must not abort primary operation.
    crate::error::record_ignored(
        storage::append_entry(
            Utc::now().timestamp_millis(),
            actor,
            method_name,
            ui_target,
            trigger,
            &attach_caller_frames(Some(params), &caller_frames),
            result,
            message,
            stack_trace.as_deref(),
        ),
        "record_if_active ui_audit",
    );
}

#[cfg(test)]
mod tests {
    use super::{
        attach_caller_frames, caller_frames_from_backtrace, relative_frame_location,
        resolve_stack_trace,
    };

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

    #[test]
    fn resolve_stack_trace_prefers_frontend_frames() {
        let params = serde_json::json!({
            "stack_frames": [
                { "function": "handleClick", "location": "src/components/x.tsx:10" },
                { "function": "", "location": "" },
            ],
        });
        let backend = vec![("backend_fn".to_string(), "src-tauri/src/a.rs:1".to_string())];
        let resolved = resolve_stack_trace(&params, &backend).expect("trace json");
        assert!(resolved.contains("handleClick"));
        assert!(!resolved.contains("backend_fn"));
    }

    #[test]
    fn resolve_stack_trace_falls_back_to_backend_frames() {
        let backend = vec![("backend_fn".to_string(), "src-tauri/src/a.rs:1".to_string())];
        let from_empty = resolve_stack_trace(&serde_json::json!({}), &backend).expect("trace json");
        assert!(from_empty.contains("backend_fn"));
        let from_invalid = resolve_stack_trace(
            &serde_json::json!({ "stack_frames": { "nope": true } }),
            &backend,
        )
        .expect("trace json");
        assert!(from_invalid.contains("backend_fn"));
        assert!(resolve_stack_trace(&serde_json::json!({}), &[]).is_none());
    }

    #[test]
    fn attach_caller_frames_keeps_frontend_frames_and_limits_injected() {
        let frames = vec![
            ("one".to_string(), "a.rs:1".to_string()),
            ("two".to_string(), "b.rs:2".to_string()),
            ("three".to_string(), "c.rs:3".to_string()),
        ];
        // Frontend-supplied frames are preserved untouched.
        let kept = attach_caller_frames(
            Some(
                serde_json::json!({ "stack_frames": [{ "function": "ui", "location": "x.tsx:1" }] }),
            ),
            &frames,
        );
        assert!(kept.contains("\"ui\""));
        assert!(!kept.contains("\"one\""));
        // Backend injection is capped at PARAMS_FRAME_LIMIT frames.
        let injected = attach_caller_frames(None, &frames);
        assert!(injected.contains("\"one\""));
        assert!(injected.contains("\"two\""));
        assert!(!injected.contains("\"three\""));
    }
}
