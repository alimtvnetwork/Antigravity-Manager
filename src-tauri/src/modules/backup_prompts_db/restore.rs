use crate::error::AppError;
use crate::modules::repo_db::{self, ActivePrompt};
use chrono::Utc;
use rusqlite::{params, Connection};
use std::fs;
use std::path::Path;

use super::*;

/// Type alias for SQLite database connection / pool used across restore routines
pub type SqlitePool = Connection;

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

    pub fn with_details(
        restored_count: usize,
        dispatched_count: usize,
        failed_count: usize,
        skipped_live: usize,
        last_error: Option<String>,
    ) -> Self {
        Self {
            restored: restored_count,
            dispatched: dispatched_count,
            failed: failed_count,
            skipped_live,
            restored_count,
            dispatched_count,
            failed_count,
            last_error,
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
    pub fn skipped_live_count(&self) -> usize {
        self.skipped_live
    }

    #[inline]
    pub fn is_success(&self) -> bool {
        self.failed == 0
    }
}

/// Restore running prompts: re-inserts them into repo_db and updates restoration flags.
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
    restore_running_prompts_for_instance_with_reassign(
        instance_id,
        instance_id,
        keep_backup,
        custom_file,
    )
}

/// Restores prompts for an instance, optionally re-assigning them to target instance.
pub fn restore_running_prompts_for_instance_with_reassign(
    instance_id: Option<&str>,
    reassign_to_instance: Option<&str>,
    keep_backup: bool,
    custom_file: Option<&str>,
) -> Result<(Vec<PromptBackupRecord>, RestoreOutcome), String> {
    let is_resend_active = crate::modules::repo_db::backup::RESEND_IN_PROGRESS.with(|f| f.get());
    if is_resend_active {
        return Err("restore_running_prompts_for_instance must not run while resend_queued_prompts_for_instance is active (double-dispatch guard)".to_string());
    }

    let target_inst = instance_id.unwrap_or("default");
    let is_unscoped = instance_id.is_none();
    let conn = connect_backup_db(custom_file)?;
    let now = Utc::now().timestamp();
    let mut outcome = RestoreOutcome::default();

    let is_target_default = target_inst == "default" || target_inst == "__default__";
    let (query, update_query) = build_restore_queries(is_unscoped, is_target_default);
    let records =
        fetch_unrestored_records(&conn, query, is_unscoped, is_target_default, target_inst)?;

    repo_db::reset_dispatched_prompts_cache();
    let live_conn = acquire_live_conn_best_effort();
    reinject_records_into_active(
        &records,
        live_conn.as_ref(),
        target_inst,
        reassign_to_instance,
        is_unscoped,
        now,
        &mut outcome,
    );

    let has_records = !records.is_empty();
    if !keep_backup && has_records {
        mark_backups_restored(
            &conn,
            update_query,
            is_unscoped,
            is_target_default,
            target_inst,
            reassign_to_instance,
            now,
        );
    }

    dispatch_restored_commands(is_unscoped, target_inst, reassign_to_instance, &mut outcome);
    rearm_restored_goals(&records, target_inst, reassign_to_instance, is_unscoped);

    Ok((records, outcome))
}

fn acquire_live_conn_best_effort() -> Option<Connection> {
    match repo_db::connect_db() {
        Ok(c) => Some(c),
        Err(e) => {
            // Justification: probing live status is best-effort dedup; proceed without live dedup if repo_db unavailable
            crate::error::record_ignored(
                Err::<(), _>(format!("connect repo_db for live probe: {e}")),
                "connect repo_db for live probe",
            );
            None
        }
    }
}

fn build_restore_queries(
    is_unscoped: bool,
    is_target_default: bool,
) -> (&'static str, &'static str) {
    if is_unscoped {
        (
            "SELECT id, backup_batch_id, prompt_id, project_name, project_path, project_id,
                    conversation_id, conversation_name, sequence_id, prompt_text, has_images,
                    images_payload, status, created_at, is_restored, restored_at, instance_id
             FROM prompt_backups
             WHERE is_restored = 0
             ORDER BY created_at ASC, id ASC",
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
             ORDER BY created_at ASC, id ASC",
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
             ORDER BY created_at ASC, id ASC",
            "UPDATE prompt_backups SET is_restored = 1, restored_at = ?1 WHERE is_restored = 0 AND instance_id = ?2",
        )
    }
}

fn fetch_unrestored_records(
    conn: &Connection,
    query: &str,
    is_unscoped: bool,
    is_target_default: bool,
    target_inst: &str,
) -> Result<Vec<PromptBackupRecord>, String> {
    let mut stmt = conn
        .prepare(query)
        .map_err(|e| format!("Failed to prepare restore query: {}", e))?;

    if is_unscoped || is_target_default {
        stmt.query_map([], |row| parse_prompt_record(row))
            .map_err(|e| format!("Failed to query unrestored prompts: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect unrestored prompts: {}", e))
    } else {
        stmt.query_map(params![target_inst], |row| parse_prompt_record(row))
            .map_err(|e| format!("Failed to query unrestored prompts: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect unrestored prompts: {}", e))
    }
}

fn reinject_records_into_active(
    records: &[PromptBackupRecord],
    live_conn: Option<&Connection>,
    target_inst: &str,
    reassign_to: Option<&str>,
    is_unscoped: bool,
    now: i64,
    outcome: &mut RestoreOutcome,
) {
    for rec in records {
        reinject_single_record(
            rec,
            live_conn,
            target_inst,
            reassign_to,
            is_unscoped,
            now,
            outcome,
        );
    }
}

fn reinject_single_record(
    rec: &PromptBackupRecord,
    live_conn: Option<&Connection>,
    target_inst: &str,
    reassign_to: Option<&str>,
    is_unscoped: bool,
    now: i64,
    outcome: &mut RestoreOutcome,
) {
    let eff_inst = if let Some(reassigned) = reassign_to {
        if !reassigned.is_empty() {
            reassigned
        } else if !is_unscoped && !target_inst.is_empty() {
            target_inst
        } else {
            rec.instance_id.as_deref().unwrap_or(target_inst)
        }
    } else if !is_unscoped && !target_inst.is_empty() {
        target_inst
    } else {
        rec.instance_id.as_deref().unwrap_or(target_inst)
    };
    let restored_status = prompt_status_after_restore(&rec.status);
    tracing::info!(
        "[PROMPT_RESTORE] Restoring prompt {} for instance '{}' (original status: '{}', has_images: {}) -> restored status: '{}'",
        rec.prompt_id, eff_inst, rec.status, rec.has_images, restored_status
    );
    crate::modules::logger::log_info(&format!(
        "[PROMPT_RESTORE] Restoring prompt {} for instance '{}' (has_images: {}) -> status: {}",
        rec.prompt_id, eff_inst, rec.has_images, restored_status
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

    let live_status = live_conn
        .and_then(|c| live_prompt_status(c, &rec.prompt_id, &rec.project_path, &rec.prompt_text));
    let has_live_final_status = is_live_status_preserved(live_status.as_deref());
    if has_live_final_status {
        outcome.record_skipped_live();
        crate::modules::logger::log_info(&format!(
            "[BackupDB] Restore preserving live prompt {} on {} (live status: {})",
            rec.prompt_id,
            eff_inst,
            live_status.as_deref().unwrap_or("unknown")
        ));
        return;
    }

    match repo_db::save_or_requeue_prompt(&active_p) {
        Ok(()) => {
            outcome.record_restored();
            tracing::info!(
                "[PROMPT_RESTORE] Recovered prompt {} on instance '{}' with status '{}' (FIFO created_at: {})",
                rec.prompt_id, eff_inst, restored_status, active_p.created_at
            );
        }
        Err(e) => {
            outcome.record_failed(Some(e.clone()));
            tracing::error!(
                "[PROMPT_RESTORE] Failed to re-inject prompt {} on instance '{}': {}",
                rec.prompt_id,
                eff_inst,
                e
            );
            crate::modules::logger::log_error(&format!(
                "[BackupDB] Restore failed to re-inject prompt {} on {}: {}",
                rec.prompt_id, eff_inst, e
            ));
        }
    }
}

fn mark_backups_restored(
    conn: &Connection,
    update_query: &str,
    is_unscoped: bool,
    is_target_default: bool,
    target_inst: &str,
    reassign_to: Option<&str>,
    now: i64,
) {
    if is_unscoped || is_target_default {
        // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
        crate::error::record_ignored(conn.execute(update_query, params![now]), "db execute");
        if let Some(reassigned) = reassign_to {
            if !reassigned.is_empty() {
                crate::error::record_ignored(
                    conn.execute(
                        "UPDATE prompt_backups SET instance_id = ?1 WHERE restored_at = ?2",
                        params![reassigned, now],
                    ),
                    "reassign prompt_backups instance_id",
                );
            }
        }
    } else {
        // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
        crate::error::record_ignored(
            conn.execute(update_query, params![now, target_inst]),
            "db execute",
        );
    }
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(
        conn.execute(
            "UPDATE backup_batches SET is_fully_restored = 1 WHERE id IN (
                SELECT backup_batch_id FROM prompt_backups GROUP BY backup_batch_id HAVING min(is_restored) = 1
             )",
            [],
        ),
        "db execute",
    );
}

fn dispatch_restored_commands(
    is_unscoped: bool,
    target_inst: &str,
    reassign_to: Option<&str>,
    outcome: &mut RestoreOutcome,
) {
    let target_scope = if let Some(reassigned) = reassign_to {
        if !reassigned.is_empty() {
            Some(reassigned)
        } else if is_unscoped {
            None
        } else {
            Some(target_inst)
        }
    } else if is_unscoped {
        None
    } else {
        Some(target_inst)
    };
    match repo_db::resend_running_commands_for_instance(target_scope, 20) {
        Ok(sent) => outcome.record_dispatched(sent.len()),
        Err(e) => {
            outcome.record_failed(Some(e.clone()));
            crate::modules::logger::log_error(&format!(
                "[BackupDB] Restore dispatch pass failed for instance '{}': {}",
                target_scope.unwrap_or("all"),
                e
            ));
        }
    }
}

fn rearm_restored_goals(
    records: &[PromptBackupRecord],
    target_inst: &str,
    reassign_to: Option<&str>,
    is_unscoped: bool,
) {
    use std::collections::HashSet;
    let mut goal_instances: HashSet<&str> = HashSet::new();
    if let Some(reassigned) = reassign_to {
        if !reassigned.is_empty() {
            goal_instances.insert(reassigned);
        }
    }
    if goal_instances.is_empty() {
        for rec in records {
            let eff = if !is_unscoped && !target_inst.is_empty() {
                target_inst
            } else {
                rec.instance_id.as_deref().unwrap_or(target_inst)
            };
            goal_instances.insert(eff);
        }
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

/// Restore protocol: prompts that were running or in-progress are restored to 'in_progress'
/// so the execution engine can resume them; queued stays 'queued'; all other backed-up / interrupted
/// prompts are transitioned to 'pending' so they leave 'backed_up' and become immediately eligible for dispatch.
pub(crate) fn prompt_status_after_restore(saved: &str) -> &'static str {
    match saved {
        "running" | "in_progress" => "in_progress",
        "queued" => "queued",
        _ => "pending",
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
    match conn.query_row(
        "SELECT status FROM active_prompts WHERE id = ?1",
        params![prompt_id],
        |row| row.get::<_, String>(0),
    ) {
        Ok(st) => return Some(st),
        Err(rusqlite::Error::QueryReturnedNoRows) => {}
        Err(e) => {
            // Justification: probing live status by prompt id is best-effort; non-existence/read errors fall back to signature probe
            crate::error::record_ignored(Err::<(), _>(e), "query live_prompt_status by id");
        }
    }

    let norm_slash = repo_path.trim().replace('\\', "/");
    let norm_bslash = repo_path.trim().replace('/', "\\");
    match conn.query_row(
        "SELECT status FROM active_prompts WHERE (repo_path = ?1 OR repo_path = ?2 OR repo_path = ?3) AND prompt_content = ?4 LIMIT 1",
        params![repo_path, norm_slash, norm_bslash, prompt_text],
        |row| row.get::<_, String>(0),
    ) {
        Ok(st) => Some(st),
        Err(rusqlite::Error::QueryReturnedNoRows) => None,
        Err(e) => {
            // Justification: probing live status by signature is best-effort; fallback to None when row missing or query fails
            crate::error::record_ignored(Err::<(), _>(e), "query live_prompt_status by signature");
            None
        }
    }
}

/// RC-6: a live row that is already `completed` must never be
/// overwritten by a backup re-injection (`save_or_requeue_prompt` upserts
/// with INSERT OR REPLACE). Interrupted `running` prompts are restored
/// and requeued so they re-execute on the newly rotated account.
pub(crate) fn is_live_status_preserved(status: Option<&str>) -> bool {
    matches!(status, Some("completed"))
}

/// Re-attribute unrestored prompt backups to a new target instance.
/// If `from_instance` is Some, only backups belonging to that instance (including default aliases if from is default) are re-attributed.
/// If `from_instance` is None, all unrestored backups are re-attributed to `to_instance`.
pub fn reattribute_unrestored_prompt_backups(
    from_instance: Option<&str>,
    to_instance: &str,
    custom_file: Option<&str>,
) -> Result<usize, String> {
    let conn = connect_backup_db(custom_file)?;
    let count = match from_instance {
        Some(from) if from == "default" || from == "__default__" => conn.execute(
            "UPDATE prompt_backups SET instance_id = ?1 WHERE is_restored = 0 AND (instance_id = 'default' OR instance_id = '__default__' OR instance_id IS NULL OR instance_id = '')",
            params![to_instance],
        ),
        Some(from) => conn.execute(
            "UPDATE prompt_backups SET instance_id = ?1 WHERE is_restored = 0 AND instance_id = ?2",
            params![to_instance, from],
        ),
        None => conn.execute(
            "UPDATE prompt_backups SET instance_id = ?1 WHERE is_restored = 0",
            params![to_instance],
        ),
    }
    .map_err(|e| format!("Failed to reattribute prompt backups: {}", e))?;

    // Also sweep and reattribute any orphaned backed_up records to target instance
    crate::error::record_ignored(
        sweep_and_reattribute_orphaned_backups(&conn, to_instance).map_err(|e| e.to_string()),
        "sweep_and_reattribute_orphaned_backups in reattribute_unrestored_prompt_backups",
    );

    crate::modules::logger::log_info(&format!(
        "[BackupDB] Re-attributed {} unrestored prompt backups (from: {:?}) to target instance '{}'",
        count, from_instance, to_instance
    ));

    Ok(count)
}

/// Sweeps unrestored prompt backups that were orphaned or assigned to an inactive / different instance
/// and reattributes them to `current_instance_id` in strict FIFO order (`ORDER BY created_at ASC, id ASC`).
///
/// Preserves the original `created_at` timestamp so the true chronological queue sequence is maintained.
pub fn sweep_and_reattribute_orphaned_backups(
    pool: &SqlitePool,
    current_instance_id: &str,
) -> Result<usize, AppError> {
    let is_curr_default = current_instance_id == "default" || current_instance_id == "__default__";

    let select_sql = if is_curr_default {
        "SELECT id, prompt_id, created_at, instance_id, status FROM prompt_backups
         WHERE is_restored = 0
           AND (status = 'backed_up' OR status = 'pending' OR status = 'in_progress' OR status = 'queued')
           AND instance_id IS NOT NULL
           AND instance_id NOT IN ('', 'default', '__default__')
         ORDER BY created_at ASC, id ASC"
    } else {
        "SELECT id, prompt_id, created_at, instance_id, status FROM prompt_backups
         WHERE is_restored = 0
           AND (status = 'backed_up' OR status = 'pending' OR status = 'in_progress' OR status = 'queued')
           AND (instance_id IS NULL OR instance_id != ?1)
         ORDER BY created_at ASC, id ASC"
    };

    let mut stmt = pool.prepare(select_sql)?;

    struct OrphanRecord {
        id: String,
        prompt_id: String,
        created_at: i64,
        old_instance: Option<String>,
        status: String,
    }

    let orphans: Vec<OrphanRecord> = if is_curr_default {
        stmt.query_map([], |row| {
            Ok(OrphanRecord {
                id: row.get(0)?,
                prompt_id: row.get(1)?,
                created_at: row.get(2)?,
                old_instance: row.get(3)?,
                status: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?
    } else {
        stmt.query_map(params![current_instance_id], |row| {
            Ok(OrphanRecord {
                id: row.get(0)?,
                prompt_id: row.get(1)?,
                created_at: row.get(2)?,
                old_instance: row.get(3)?,
                status: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?
    };

    if orphans.is_empty() {
        tracing::info!(
            "[PROMPT_RESTORE_SWEEP] No orphaned backed-up prompts found for instance '{}'",
            current_instance_id
        );
        return Ok(0);
    }

    let count = orphans.len();
    for rec in &orphans {
        pool.execute(
            "UPDATE prompt_backups SET instance_id = ?1 WHERE id = ?2",
            params![current_instance_id, rec.id],
        )?;

        tracing::info!(
            "[PROMPT_RESTORE_SWEEP] Reattributed orphaned prompt backup '{}' (prompt_id: '{}', status: '{}', old_instance: '{:?}', created_at: {}) -> current_instance: '{}'",
            rec.id, rec.prompt_id, rec.status, rec.old_instance, rec.created_at, current_instance_id
        );
    }

    tracing::info!(
        "[PROMPT_RESTORE_SWEEP] Completed sweep: reattributed {} orphaned backup(s) in strict FIFO order to active instance '{}'",
        count, current_instance_id
    );

    Ok(count)
}

/// Convenience helper to run `sweep_and_reattribute_orphaned_backups` with a custom or default backup DB file.
pub fn sweep_and_reattribute_orphaned_backups_with_file(
    current_instance_id: &str,
    custom_file: Option<&str>,
) -> Result<usize, AppError> {
    let conn = connect_backup_db(custom_file).map_err(AppError::Unknown)?;
    sweep_and_reattribute_orphaned_backups(&conn, current_instance_id)
}

/// Count unrestored backup records for a given instance (or all unrestored if None).
pub fn count_unrestored_prompts_for_instance(instance_id: Option<&str>) -> Result<usize, String> {
    count_unrestored_prompts_for_instance_with_file(instance_id, None)
}

/// Count unrestored backup records for a given instance with optional custom database file.
pub fn count_unrestored_prompts_for_instance_with_file(
    instance_id: Option<&str>,
    custom_file: Option<&str>,
) -> Result<usize, String> {
    let conn = connect_backup_db(custom_file)?;
    let target_inst = instance_id.unwrap_or("default");
    let is_unscoped = instance_id.is_none();
    let is_target_default = target_inst == "default" || target_inst == "__default__";

    let count: i64 = if is_unscoped {
        conn.query_row(
            "SELECT COUNT(*) FROM prompt_backups WHERE is_restored = 0",
            [],
            |row| row.get(0),
        )
    } else if is_target_default {
        conn.query_row(
            "SELECT COUNT(*) FROM prompt_backups WHERE is_restored = 0 AND (instance_id = 'default' OR instance_id IS NULL OR instance_id = '')",
            [],
            |row| row.get(0),
        )
    } else {
        conn.query_row(
            "SELECT COUNT(*) FROM prompt_backups WHERE is_restored = 0 AND instance_id = ?1",
            params![target_inst],
            |row| row.get(0),
        )
    }
    .map_err(|e| format!("Failed to count unrestored prompts for instance: {}", e))?;

    Ok(count as usize)
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
    query_other_unrestored_count(&conn, instance_id, is_target_default)
}

fn query_other_unrestored_count(
    conn: &Connection,
    instance_id: &str,
    is_target_default: bool,
) -> Result<usize, String> {
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
    let has_records = !records.is_empty();
    if has_records {
        return Ok((records, outcome));
    }
    fallback_unscoped_if_orphaned(instance_id, keep_backup, custom_file, (records, outcome))
}

fn fallback_unscoped_if_orphaned(
    instance_id: &str,
    keep_backup: bool,
    custom_file: Option<&str>,
    empty_result: (Vec<PromptBackupRecord>, RestoreOutcome),
) -> Result<(Vec<PromptBackupRecord>, RestoreOutcome), String> {
    let other_count = count_unrestored_prompts_for_other_instances(instance_id, custom_file)?;
    let has_orphaned = other_count > 0;
    if !has_orphaned {
        return Ok(empty_result);
    }
    crate::modules::logger::log_info(&format!(
        "[BackupDB] Scoped restore for instance '{}' found nothing; {} unrestored record(s) exist under other instances — re-attributing to '{}' and restoring",
        instance_id, other_count, instance_id
    ));
    reattribute_unrestored_prompt_backups(None, instance_id, custom_file)?;
    restore_running_prompts_for_instance_with_outcome(Some(instance_id), keep_backup, custom_file)
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
    let size_bytes = get_db_file_size(&db_path);
    let size_kb = (size_bytes as f64) / 1024.0;

    let conn = connect_backup_db(custom_file)?;
    let (total_records, restored_records, batches_count) = query_storage_counts(&conn)?;

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

fn get_db_file_size(db_path: &Path) -> u64 {
    match fs::metadata(db_path) {
        Ok(m) => m.len(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => 0,
        Err(e) => {
            // Justification: metadata lookup for db path is non-fatal for storage stats; defaults to 0
            crate::error::record_ignored(Err::<(), _>(e), "fs::metadata backup db");
            0
        }
    }
}

fn query_storage_counts(conn: &Connection) -> Result<(usize, usize, usize), String> {
    let total_records: usize = conn
        .query_row("SELECT COUNT(*) FROM prompt_backups", [], |r| r.get(0))
        .map_err(|e| format!("Failed to count prompt_backups: {}", e))?;
    let restored_records: usize = conn
        .query_row(
            "SELECT COUNT(*) FROM prompt_backups WHERE is_restored = 1",
            [],
            |r| r.get(0),
        )
        .map_err(|e| format!("Failed to count restored prompt_backups: {}", e))?;
    let batches_count: usize = conn
        .query_row("SELECT COUNT(*) FROM backup_batches", [], |r| r.get(0))
        .map_err(|e| format!("Failed to count backup_batches: {}", e))?;
    Ok((total_records, restored_records, batches_count))
}
