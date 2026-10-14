//! Tauri IPC commands for the UI interaction audit recorder.

use crate::error::AppResult;

use super::recorder;
use super::types::{InteractionEntry, RecordInteractionInput, RecordInteractionOutcome};

/// Start recording UI interactions; returns the previous recording state.
#[tauri::command]
pub fn start_ui_audit_recording() -> AppResult<bool> {
    recorder::start_recording()
}

/// Stop recording UI interactions; returns the previous recording state.
#[tauri::command]
pub fn stop_ui_audit_recording() -> bool {
    recorder::stop_recording()
}

/// Recording-indicator state for the frontend.
#[tauri::command]
pub fn is_ui_audit_recording() -> bool {
    recorder::is_recording()
}

/// Delete all recorded entries; returns the number of rows removed.
#[tauri::command]
pub fn clear_ui_audit_recording() -> AppResult<u64> {
    recorder::clear_recording()
}

/// Export all recorded entries to a timestamped JSON file; returns the file path.
#[tauri::command]
pub fn export_ui_audit_recording() -> AppResult<String> {
    recorder::export_recording().map(|path| path.to_string_lossy().to_string())
}

/// Append one UI interaction entry while recording is enabled.
#[tauri::command]
pub fn record_ui_interaction(input: RecordInteractionInput) -> AppResult<RecordInteractionOutcome> {
    recorder::record_interaction(input)
}

/// List the newest recorded entries, capped at `limit`.
#[tauri::command]
pub fn list_ui_audit_interactions(limit: u32) -> AppResult<Vec<InteractionEntry>> {
    recorder::list_recorded(limit)
}
