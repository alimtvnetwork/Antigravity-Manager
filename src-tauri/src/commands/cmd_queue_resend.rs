//! Queue resend IPC commands (Task-03).
//!
//! Exposes the durable queued-prompt backup + FIFO re-send pipeline to the UI.
//! The `resend_queued_prompts` command is the GUI twin of
//! `agm prompts resend-queued`: both call
//! `repo_db::resend_queued_prompts_for_instance` (CLI/GUI parity).

/// Re-send all enqueued prompts in FIFO order for an instance.
///
/// Takes a durable FIFO snapshot of the live queue first, re-injects
/// unrestored queued backup records, then runs a SINGLE dispatch pass.
/// Already-dispatched prompts are skipped (RCA-15 dedup). Returns real
/// counts — nothing is reported as sent unless it was actually dispatched.
#[tauri::command]
pub fn resend_queued_prompts(
    instance_id: Option<String>,
    limit: Option<usize>,
) -> Result<crate::modules::repo_db::ResendQueuedOutcome, String> {
    let scope = instance_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty() && *s != "all");
    crate::modules::repo_db::resend_queued_prompts_for_instance(scope, limit.unwrap_or(20).max(1))
}

/// Durably back up ONLY the currently queued prompts, preserving strict FIFO
/// enqueue order. GUI twin of `agm prompts backup-queued`.
#[tauri::command]
pub fn backup_queued_prompts(
    instance_id: Option<String>,
) -> Result<crate::modules::backup_prompts_db::BackupBatchInfo, String> {
    let scope = instance_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty() && *s != "all");
    let (batch, _records) =
        crate::modules::backup_prompts_db::backup_queued_prompts_for_instance(scope, None)?;
    Ok(batch)
}

/// Re-send all running commands for an instance or all instances
#[tauri::command]
pub fn resend_running_prompts(
    instance_id: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<crate::modules::repo_db::ActivePrompt>, String> {
    let scope = instance_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty() && *s != "all");
    crate::modules::repo_db::resend_running_commands_for_instance(scope, limit.unwrap_or(20).max(1))
}

/// Re-send both running and enqueued prompts for an instance or all instances
#[tauri::command]
pub fn resend_all_prompts(
    instance_id: Option<String>,
    limit: Option<usize>,
) -> Result<usize, String> {
    let scope = instance_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty() && *s != "all");
    let lim = limit.unwrap_or(20).max(1);
    let running_count = crate::modules::repo_db::resend_running_commands_for_instance(scope, lim)
        .map(|v| v.len())
        .unwrap_or(0);
    let queued_outcome = crate::modules::repo_db::resend_queued_prompts_for_instance(scope, lim)
        .map(|o| o.dispatched)
        .unwrap_or(0);
    Ok(running_count + queued_outcome)
}

/// Durably back up running and queued prompts for an instance
#[tauri::command]
pub fn backup_running_and_queued_prompts(
    instance_id: Option<String>,
) -> Result<crate::modules::backup_prompts_db::BackupBatchInfo, String> {
    let scope = instance_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty() && *s != "all");
    let (batch, _records) =
        crate::modules::backup_prompts_db::backup_active_running_prompts_for_instance(scope, None)?;
    Ok(batch)
}

/// Clear and cancel all queued prompts for an instance (or across all instances).
#[tauri::command]
pub fn clear_queued_prompts(instance_id: Option<String>) -> Result<usize, String> {
    let scope = instance_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty() && *s != "all");
    crate::modules::repo_db::clear_queued_prompts_for_instance(scope)
}
