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

/// Real counts produced by a restore pass (RC-3). `restored` counts backup
/// records actually re-injected into `active_prompts`; `dispatched` counts
/// prompts the dispatch pass actually sent; `skipped_live` counts records
/// preserved because their live row is already `running`/`completed` (RC-6);
/// `failed` counts re-injection failures plus a failed dispatch pass, with
/// `last_error` carrying the most recent failure message.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct RestoreOutcome {
    pub restored: usize,
    pub dispatched: usize,
    pub failed: usize,
    pub skipped_live: usize,
    pub restored_count: usize,
    pub dispatched_count: usize,
    pub failed_count: usize,
    pub last_error: Option<String>,
}

impl RestoreOutcome {
    pub fn new(restored_count: usize, dispatched_count: usize, failed_count: usize) -> Self {
        Self {
            restored: restored_count,
            dispatched: dispatched_count,
            failed: failed_count,
            skipped_live: 0,
            restored_count,
            dispatched_count,
            failed_count,
            last_error: None,
        }
    }

    #[inline]
    pub fn record_restored(&mut self) {
        self.restored += 1;
        self.restored_count = self.restored;
    }

    #[inline]
    pub fn record_dispatched(&mut self, count: usize) {
        self.dispatched = count;
        self.dispatched_count = count;
    }

    #[inline]
    pub fn record_failed(&mut self, err: Option<String>) {
        self.failed += 1;
        self.failed_count = self.failed;
        if err.is_some() {
            self.last_error = err;
        }
    }

    #[inline]
    pub fn record_skipped_live(&mut self) {
        self.skipped_live += 1;
    }

    #[inline]
    pub fn restored_count(&self) -> usize {
        self.restored
    }

    #[inline]
    pub fn dispatched_count(&self) -> usize {
        self.dispatched
    }

    #[inline]
    pub fn failed_count(&self) -> usize {
        self.failed
    }

    #[inline]
    pub fn is_success(&self) -> bool {
        self.failed == 0
    }
}

/// Restore running prompts: re-inserts them into repo_db and updates restoration flags.
///
/// Scoping contract:
/// - `instance_id = Some(id)`: strictly scoped to that instance. The `default`
///   scope also matches legacy rows with NULL/empty `instance_id`.
/// - `instance_id = None`: restores ALL unrestored backups across every instance
///   in FIFO order, re-injecting each prompt under its own recorded instance id.
///   This is the historical behavior the CLI relied on before per-instance
///   scoping was introduced; strict scoping broke post-switch restores because
///   backups are recorded under the instance that was active at backup time,
///   which differs from the active instance at restore time.
pub fn restore_running_prompts_for_instance(
    instance_id: Option<&str>,
    keep_backup: bool,
    custom_file: Option<&str>,
) -> Result<Vec<PromptBackupRecord>, String> {
    let (records, _outcome) =
        restore_running_prompts_for_instance_with_outcome(instance_id, keep_backup, custom_file)?;
    Ok(records)
}

/// Same restore as `restore_running_prompts_for_instance`, additionally
/// returning the real [`RestoreOutcome`] counts instead of swallowing the
/// re-injection and dispatch results (RC-3).
pub fn restore_running_prompts_for_instance_with_outcome(
    instance_id: Option<&str>,
    keep_backup: bool,
    custom_file: Option<&str>,
) -> Result<(Vec<PromptBackupRecord>, RestoreOutcome), String> {
    // Regression guard: restore must never run on the resend path.
    // `resend_queued_prompts_for_instance` runs its own single dispatch pass;
    // a nested restore would dispatch every prompt a second time.
    let resend_active = crate::modules::repo_db::backup::RESEND_IN_PROGRESS.with(|f| f.get());
    if resend_active {
        return Err("restore_running_prompts_for_instance must not run while resend_queued_prompts_for_instance is active (double-dispatch guard)".to_string());
    }

    let target_inst = instance_id.unwrap_or("default");
    // None means "restore everything": no instance predicate at all.
    let unscoped = instance_id.is_none();
    let conn = connect_backup_db(custom_file)?;
    let now = Utc::now().timestamp();
    let mut outcome = RestoreOutcome::default();

    // Query unrestored prompts: unscoped restores every instance (FIFO); otherwise
    // scoped strictly to the target instance.
    let is_target_default = target_inst == "default" || target_inst == "__default__";
    let (query, update_query) = if unscoped {
        (
            "SELECT id, backup_batch_id, prompt_id, project_name, project_path, project_id,
                    conversation_id, conversation_name, sequence_id, prompt_text, has_images,
                    images_payload, status, created_at, is_restored, restored_at, instance_id
             FROM prompt_backups
             WHERE is_restored = 0
             ORDER BY created_at ASC, sequence_id ASC",
            "UPDATE prompt_backups SET is_restored = 1, restored_at = ?1 WHERE is_restored = 0",
        )
    } else if is_target_default {
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

    let records: Vec<PromptBackupRecord> = if unscoped || is_target_default {
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
    // RC-6: probe live statuses first — a row already `running`/`completed`
    // must never be overwritten by the INSERT OR REPLACE in save_or_requeue_prompt.
    let live_conn = repo_db::connect_db().ok();
    for rec in &records {
        let eff_inst = rec.instance_id.as_deref().unwrap_or(target_inst);
        let restored_status = prompt_status_after_restore(&rec.status);
        crate::modules::logger::log_info(&format!(
            "[PROMPT_RESTORE] Restoring prompt {} for instance '{}' (has_images: {}) -> status: {}",
            rec.prompt_id, eff_inst, rec.has_images, restored_status
        ));
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
            created_at: if rec.created_at > 0 {
                rec.created_at
            } else {
                now
            },
            updated_at: now,
            image_payload: rec.images_payload.clone(),
        };
        let live_status = live_conn.as_ref().and_then(|c| {
            live_prompt_status(c, &rec.prompt_id, &rec.project_path, &rec.prompt_text)
        });
        let has_live_final_status = is_live_status_preserved(live_status.as_deref());
        if has_live_final_status {
            outcome.record_skipped_live();
            crate::modules::logger::log_info(&format!(
                "[BackupDB] Restore preserving live prompt {} on {} (live status: {})",
                rec.prompt_id,
                eff_inst,
                live_status.as_deref().unwrap_or("unknown")
            ));
            continue;
        }
        match repo_db::save_or_requeue_prompt(&active_p) {
            Ok(()) => outcome.record_restored(),
            Err(e) => {
                outcome.record_failed(Some(e.clone()));
                crate::modules::logger::log_error(&format!(
                    "[BackupDB] Restore failed to re-inject prompt {} on {}: {}",
                    rec.prompt_id, eff_inst, e
                ));
            }
        }
    }

    if !keep_backup && !records.is_empty() {
        if unscoped || is_target_default {
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

    // Automatically trigger resend and execute restored prompts via CLI.
    // Unscoped restores fan out across every instance the backups belong to;
    // scoped restores stay on the target instance.
    // RC-3: the dispatch pass is no longer swallowed — the real sent count and
    // any failure are returned to the caller as outcome data, not silence.
    match repo_db::resend_running_commands_for_instance(
        if unscoped { None } else { Some(target_inst) },
        20,
    ) {
        Ok(sent) => outcome.record_dispatched(sent.len()),
        Err(e) => {
            outcome.record_failed(Some(e.clone()));
            crate::modules::logger::log_error(&format!(
                "[BackupDB] Restore dispatch pass failed for instance '{}': {}",
                target_inst, e
            ));
        }
    }
    // Re-arm prompt-goal heartbeats for every instance that received restored prompts.
    {
        use std::collections::HashSet;
        let mut goal_instances: HashSet<&str> = HashSet::new();
        for rec in &records {
            goal_instances.insert(rec.instance_id.as_deref().unwrap_or(target_inst));
        }
        if goal_instances.is_empty() {
            // Justification: non-Result return value intentionally discarded — no error channel to track
            let _ = repo_db::ensure_prompt_goals_running_for_instance(target_inst);
        } else {
            for inst in goal_instances {
                // Justification: non-Result return value intentionally discarded — no error channel to track
                let _ = repo_db::ensure_prompt_goals_running_for_instance(inst);
            }
        }
    }

    Ok((records, outcome))
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

/// RC-6 dedup probe: current live `active_prompts` status for a prompt id or signature,
/// when a live row already exists.
fn live_prompt_status(
    conn: &Connection,
    prompt_id: &str,
    repo_path: &str,
    prompt_text: &str,
) -> Option<String> {
    if let Ok(st) = conn.query_row(
        "SELECT status FROM active_prompts WHERE id = ?1",
        params![prompt_id],
        |row| row.get::<_, String>(0),
    ) {
        return Some(st);
    }
    conn.query_row(
        "SELECT status FROM active_prompts WHERE repo_path = ?1 AND prompt_content = ?2 LIMIT 1",
        params![repo_path, prompt_text],
        |row| row.get::<_, String>(0),
    )
    .ok()
}

/// RC-6: a live row that is already `running` or `completed` must never be
/// overwritten by a backup re-injection (`save_or_requeue_prompt` upserts
/// with INSERT OR REPLACE).
pub(crate) fn is_live_status_preserved(status: Option<&str>) -> bool {
    matches!(status, Some("running") | Some("completed"))
}

/// Count unrestored backup records recorded under instance ids other than
/// `instance_id` — records a strictly scoped restore for `instance_id`
/// cannot see (RC-2 fallback probe).
pub fn count_unrestored_prompts_for_other_instances(
    instance_id: &str,
    custom_file: Option<&str>,
) -> Result<usize, String> {
    let conn = connect_backup_db(custom_file)?;
    let is_target_default = instance_id == "default" || instance_id == "__default__";
    let count: i64 = if is_target_default {
        conn.query_row(
            "SELECT COUNT(*) FROM prompt_backups
             WHERE is_restored = 0
               AND instance_id IS NOT NULL
               AND instance_id NOT IN ('', 'default', '__default__')",
            [],
            |row| row.get(0),
        )
    } else {
        conn.query_row(
            "SELECT COUNT(*) FROM prompt_backups
             WHERE is_restored = 0
               AND (instance_id IS NULL OR instance_id != ?1)",
            params![instance_id],
            |row| row.get(0),
        )
    }
    .map_err(|e| {
        format!(
            "Failed to count unrestored prompts for other instances: {}",
            e
        )
    })?;
    Ok(count as usize)
}

/// Same restore as [`restore_running_prompts_scoped_with_fallback`], additionally returning
/// the honest [`RestoreOutcome`].
pub fn restore_running_prompts_scoped_with_fallback_with_outcome(
    instance_id: &str,
    keep_backup: bool,
    custom_file: Option<&str>,
) -> Result<(Vec<PromptBackupRecord>, RestoreOutcome), String> {
    let (records, outcome) = restore_running_prompts_for_instance_with_outcome(
        Some(instance_id),
        keep_backup,
        custom_file,
    )?;
    let has_restored = !records.is_empty();
    if has_restored {
        return Ok((records, outcome));
    }
    let other_count = count_unrestored_prompts_for_other_instances(instance_id, custom_file)?;
    let has_orphaned = other_count > 0;
    if !has_orphaned {
        return Ok((records, outcome));
    }
    crate::modules::logger::log_info(&format!(
        "[BackupDB] Scoped restore for instance '{}' found nothing; {} unrestored record(s) exist under other instances — falling back to unscoped restore",
        instance_id, other_count
    ));
    restore_running_prompts_for_instance_with_outcome(None, keep_backup, custom_file)
}

/// Scoped restore with the RC-2 fallback: when the strictly scoped pass
/// restores nothing but unrestored records exist under other instance ids,
/// retry with unscoped semantics so backups recorded under a previous
/// instance are not orphaned by a launch/switch into a new one.
pub fn restore_running_prompts_scoped_with_fallback(
    instance_id: &str,
    keep_backup: bool,
    custom_file: Option<&str>,
) -> Result<Vec<PromptBackupRecord>, String> {
    let (records, _outcome) = restore_running_prompts_scoped_with_fallback_with_outcome(
        instance_id,
        keep_backup,
        custom_file,
    )?;
    Ok(records)
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
