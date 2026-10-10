use crate::modules::repo_db::{self, ActivePrompt};
use chrono::Utc;
use rusqlite::{params, Connection};
use std::fs;

use super::*;

/// Backward compatible wrapper with instance support
pub fn restore_running_prompts(
    instance_id: Option<&str>,
    keep_backup: bool,
    custom_file: Option<&str>,
) -> Result<Vec<PromptBackupRecord>, String> {
    restore_running_prompts_for_instance(instance_id, keep_backup, custom_file)
}

/// Restore running prompts: re-inserts them into repo_db and updates restoration flags scoped to an instance
pub fn restore_running_prompts_for_instance(
    instance_id: Option<&str>,
    keep_backup: bool,
    custom_file: Option<&str>,
) -> Result<Vec<PromptBackupRecord>, String> {
    let target_inst = instance_id.unwrap_or("default");
    let conn = connect_backup_db(custom_file)?;
    let now = Utc::now().timestamp();

    // Query unrestored prompts scoped strictly to target instance
    let is_target_default = target_inst == "default" || target_inst == "__default__";
    let (query, update_query) = if is_target_default {
        (
            "SELECT id, backup_batch_id, prompt_id, project_name, project_path, project_id,
                    conversation_id, conversation_name, sequence_id, prompt_text, has_images,
                    images_payload, status, created_at, is_restored, restored_at, instance_id
             FROM prompt_backups 
             WHERE is_restored = 0
               AND (instance_id = 'default' OR instance_id IS NULL OR instance_id = '')
             ORDER BY created_at ASC, sequence_id ASC",
            "UPDATE prompt_backups SET is_restored = 1, restored_at = ?1 WHERE is_restored = 0 AND (instance_id = 'default' OR instance_id IS NULL OR instance_id = '')",
        )
    } else {
        (
            "SELECT id, backup_batch_id, prompt_id, project_name, project_path, project_id,
                    conversation_id, conversation_name, sequence_id, prompt_text, has_images,
                    images_payload, status, created_at, is_restored, restored_at, instance_id
             FROM prompt_backups 
             WHERE is_restored = 0
               AND instance_id = ?1
             ORDER BY created_at ASC, sequence_id ASC",
            "UPDATE prompt_backups SET is_restored = 1, restored_at = ?1 WHERE is_restored = 0 AND instance_id = ?2",
        )
    };

    let mut stmt = conn
        .prepare(query)
        .map_err(|e| format!("Failed to prepare restore query: {}", e))?;

    let records: Vec<PromptBackupRecord> = if is_target_default {
        stmt.query_map([], |row| parse_prompt_record(row))
            .map_err(|e| format!("Failed to query unrestored prompts: {}", e))?
            .flatten()
            .collect()
    } else {
        stmt.query_map(params![target_inst], |row| parse_prompt_record(row))
            .map_err(|e| format!("Failed to query unrestored prompts: {}", e))?
            .flatten()
            .collect()
    };

    // Reset dispatched prompts cache to allow restored prompts to execute post-switch
    repo_db::reset_dispatched_prompts_cache();

    // Re-inject into repo_db active_prompts table if records exist in backup DB.
    // Queued prompts stay queued. Anything that was running is marked backed_up so
    // dispatch sends that prompt again and leaves the queue untouched.
    for rec in &records {
        let eff_inst = rec.instance_id.as_deref().unwrap_or(target_inst);
        let restored_status = prompt_status_after_restore(&rec.status);
        crate::modules::logger::log_info(&format!(
            "[BackupDB] Restore prompt {} on {}: saved '{}' -> '{}'",
            rec.prompt_id, eff_inst, rec.status, restored_status
        ));
        let active_p = ActivePrompt {
            id: rec.prompt_id.clone(),
            project_id: rec.project_id.clone(),
            instance_id: eff_inst.to_string(),
            repo_path: rec.project_path.clone(),
            prompt_content: rec.prompt_text.clone(),
            model: Some("gemini-3.8-flash-high".to_string()),
            session_id: Some(rec.conversation_id.clone()),
            status: restored_status.to_string(),
            created_at: now,
            updated_at: now,
            image_payload: rec.images_payload.clone(),
        };
        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
        crate::error::record_ignored(
            repo_db::save_or_requeue_prompt(&active_p),
            "save_or_requeue_prompt",
        );
    }

    if !keep_backup && !records.is_empty() {
        if is_target_default {
            // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
            crate::error::record_ignored(conn.execute(update_query, params![now]), "db execute");
        } else {
            // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
            crate::error::record_ignored(
                conn.execute(update_query, params![now, target_inst]),
                "db execute",
            );
        }
        // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
        crate::error::record_ignored(conn.execute(
            "UPDATE backup_batches SET is_fully_restored = 1 WHERE id IN (
                SELECT backup_batch_id FROM prompt_backups GROUP BY backup_batch_id HAVING min(is_restored) = 1
             )",
            [],
        ), "db execute");
    }

    // Automatically trigger resend and execute restored prompts via CLI scoped to this instance
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(
        repo_db::resend_running_commands_for_instance(Some(target_inst), 20),
        "resend_running_commands_for_instance",
    );
    // Justification: non-Result return value intentionally discarded — no error channel to track
    let _ = repo_db::ensure_prompt_goals_running_for_instance(target_inst);

    Ok(records)
}

/// Queued stays queued. A running, dispatched, or already backed-up prompt is
/// marked backed_up so the dispatcher sends it again.
pub(crate) fn prompt_status_after_restore(saved: &str) -> &'static str {
    if saved == "queued" {
        "queued"
    } else {
        "backed_up"
    }
}

/// Query storage information of the split database
pub fn get_storage_info(custom_file: Option<&str>) -> Result<serde_json::Value, String> {
    let db_path = get_backup_prompts_db_path(custom_file)?;
    let size_bytes = fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0);
    let size_kb = (size_bytes as f64) / 1024.0;

    let conn = connect_backup_db(custom_file)?;
    let total_records: usize = conn
        .query_row("SELECT COUNT(*) FROM prompt_backups", [], |r| r.get(0))
        .unwrap_or(0);
    let restored_records: usize = conn
        .query_row(
            "SELECT COUNT(*) FROM prompt_backups WHERE is_restored = 1",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let batches_count: usize = conn
        .query_row("SELECT COUNT(*) FROM backup_batches", [], |r| r.get(0))
        .unwrap_or(0);

    Ok(serde_json::json!({
        "database_path": db_path.to_string_lossy(),
        "size_bytes": size_bytes,
        "size_kb": format!("{:.2} KB", size_kb),
        "total_prompt_records": total_records,
        "restored_records": restored_records,
        "unrestored_records": total_records.saturating_sub(restored_records),
        "total_batches": batches_count,
        "retention_days": 1,
    }))
}
