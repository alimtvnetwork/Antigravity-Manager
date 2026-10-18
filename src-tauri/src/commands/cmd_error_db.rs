//! Tauri commands for Speed Error Database (`errors.db`) and self-healing engine.

use crate::modules::error_db::{
    self, ErrorQueryOptions, ErrorRecord, NewErrorRecord, SelfHealingReport,
};

#[tauri::command]
pub async fn log_client_error(entry: NewErrorRecord) -> Result<String, String> {
    error_db::record_error(&entry)
}

#[tauri::command]
pub async fn list_client_errors(
    limit: Option<usize>,
    severity: Option<String>,
) -> Result<Vec<ErrorRecord>, String> {
    let options = ErrorQueryOptions {
        limit,
        offset: None,
        severity,
        error_code: None,
        search: None,
    };
    error_db::list_errors(&options)
}

#[tauri::command]
pub async fn clear_client_errors() -> Result<usize, String> {
    error_db::clear_errors()
}

#[tauri::command]
pub async fn run_error_diagnostics(apply_fixes: Option<bool>) -> Result<SelfHealingReport, String> {
    let do_fix = apply_fixes.unwrap_or(true);
    error_db::run_self_healing(do_fix)
}
