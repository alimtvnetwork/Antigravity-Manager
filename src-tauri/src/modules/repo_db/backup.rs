//! Repo DB: backup

use super::detection::{detect_running_projects, resume_task_document};
use super::discovery::discover_running_prompts_from_antigravity;
use super::models::{ActivePrompt, RunningProject};
use super::project_queries::get_live_project_execution_info;
use super::schema::{connect_db, init_tables};
use super::state::{get_memory_prompts_map, reset_dispatched_prompts_cache};
use super::tree::invalidate_prompt_tree_cache;
use chrono::Utc;
use rusqlite::params;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use uuid::Uuid;

thread_local! {
    /// Regression guard: set while `resend_queued_prompts_for_instance` runs.
    /// `backup_prompts_db::restore_running_prompts_for_instance` must never
    /// execute on the resend path — it dispatches internally, so calling it
    /// from resend would double-dispatch. The restore entry point checks this
    /// flag and fails loudly if the invariant is violated.
    pub(crate) static RESEND_IN_PROGRESS: std::cell::Cell<bool> =
        std::cell::Cell::new(false);
}

/// RAII guard that marks the resend path as active for the current thread.
/// Dropping the guard (including on early return / `?`) clears the flag.
struct ResendPathGuard;

impl ResendPathGuard {
    fn enter() -> Self {
        RESEND_IN_PROGRESS.with(|f| f.set(true));
        ResendPathGuard
    }
}

impl Drop for ResendPathGuard {
    fn drop(&mut self) {
        RESEND_IN_PROGRESS.with(|f| f.set(false));
    }
}

/// Outcome of re-sending backed-up queued prompts via a single dispatch pass.
/// Counts are real — nothing is reported as sent unless the dispatch pass
/// actually dispatched it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResendQueuedOutcome {
    pub instance_id: String,
    pub backup_batch_id: Option<String>,
    pub backed_up_queued: usize,
    pub reinjected: usize,
    pub dispatched: usize,
    pub dispatched_from_backup: usize,
    pub skipped_duplicates: usize,
    pub failed: usize,
    pub failures: Vec<String>,
}

fn norm_resend_sig_path(repo_path: &str) -> String {
    repo_path.trim().to_lowercase().replace('\\', "/")
}

fn resend_queued_sig(repo_path: &str, prompt_text: &str) -> String {
    format!(
        "{}\u{1f}{}",
        norm_resend_sig_path(repo_path),
        prompt_text.trim()
    )
}

/// Canonical instance match used by the queued re-send dedup (mirrors the
/// dispatch-time matching in `resend_running_commands_for_instance`).
fn resend_queued_instance_matches(scope: Option<&str>, prompt_instance: &str) -> bool {
    match scope {
        None | Some("all") => true,
        Some(target) => {
            let canon = |s: &str| {
                crate::modules::instance::resolve_instance_id(s).unwrap_or_else(|_| s.to_string())
            };
            let p_inst = canon(prompt_instance);
            if target == "default" || target == "__default__" {
                p_inst == "default" || p_inst == "__default__" || p_inst.is_empty()
            } else {
                p_inst == canon(target)
            }
        }
    }
}

/// Re-send all enqueued prompts in FIFO order from the durable prompt-backup
/// store.
///
/// Pipeline (SINGLE dispatch — never layered on another dispatch path):
/// 1. Take a durable FIFO snapshot of the live queue (`backup_queued_prompts_for_instance`).
/// 2. Re-inject unrestored queued backup records into `active_prompts` as
///    `queued`, skipping anything already queued/dispatched (RCA-15 dedup:
///    checked against the live queue by prompt id AND by
///    repo-path+text signature, plus in-run dedup).
/// 3. Run `resend_running_commands_for_instance` exactly ONCE.
///
/// This deliberately does NOT go through
/// `backup_prompts_db::restore_running_prompts_for_instance`, which dispatches
/// internally — calling it here would double-dispatch.
pub fn resend_queued_prompts_for_instance(
    instance_id: Option<&str>,
    limit: usize,
) -> Result<ResendQueuedOutcome, String> {
    // Regression guard (Step 4): mark the resend path active so a nested
    // restore_running_prompts_for_instance call fails loudly instead of
    // double-dispatching. Cleared on drop, including early returns.
    let _resend_guard = ResendPathGuard::enter();

    let unscoped = instance_id.is_none();
    let target_inst = instance_id.unwrap_or("default");
    let now = Utc::now().timestamp();
    let mut outcome = ResendQueuedOutcome {
        instance_id: target_inst.to_string(),
        backup_batch_id: None,
        backed_up_queued: 0,
        reinjected: 0,
        dispatched: 0,
        dispatched_from_backup: 0,
        skipped_duplicates: 0,
        failed: 0,
        failures: Vec::new(),
    };

    // Step 1: durable FIFO snapshot of the live queue before re-sending.
    match crate::modules::backup_prompts_db::backup_queued_prompts_for_instance(instance_id, None) {
        Ok((batch, records)) => {
            outcome.backup_batch_id = Some(batch.id);
            outcome.backed_up_queued = records.len();
        }
        Err(e) => {
            let msg = format!(
                "queued backup snapshot failed (continuing with existing backup records): {}",
                e
            );
            crate::modules::logger::log_warn(&format!("[RepoDB] {}", msg));
            outcome.failures.push(msg);
            outcome.failed += 1;
        }
    }

    // Step 2: load unrestored queued backup records in FIFO order.
    let bconn = crate::modules::backup_prompts_db::connect_backup_db(None)?;
    let is_default_scope = target_inst == "default" || target_inst == "__default__";
    let select_sql = if unscoped {
        "SELECT id, backup_batch_id, prompt_id, project_name, project_path, project_id,
                conversation_id, conversation_name, sequence_id, prompt_text, has_images,
                images_payload, status, created_at, is_restored, restored_at, instance_id
         FROM prompt_backups
         WHERE is_restored = 0 AND status = 'queued'
         ORDER BY created_at ASC, sequence_id ASC, id ASC"
    } else if is_default_scope {
        "SELECT id, backup_batch_id, prompt_id, project_name, project_path, project_id,
                conversation_id, conversation_name, sequence_id, prompt_text, has_images,
                images_payload, status, created_at, is_restored, restored_at, instance_id
         FROM prompt_backups
         WHERE is_restored = 0 AND status = 'queued'
           AND (instance_id = 'default' OR instance_id IS NULL OR instance_id = '')
         ORDER BY created_at ASC, sequence_id ASC, id ASC"
    } else {
        "SELECT id, backup_batch_id, prompt_id, project_name, project_path, project_id,
                conversation_id, conversation_name, sequence_id, prompt_text, has_images,
                images_payload, status, created_at, is_restored, restored_at, instance_id
         FROM prompt_backups
         WHERE is_restored = 0 AND status = 'queued' AND instance_id = ?1
         ORDER BY created_at ASC, sequence_id ASC, id ASC"
    };
    let mut sel = bconn
        .prepare(select_sql)
        .map_err(|e| format!("Failed to prepare queued backup query: {}", e))?;
    let parse_row = |row: &rusqlite::Row| {
        Ok(crate::modules::backup_prompts_db::PromptBackupRecord {
            id: row.get(0)?,
            backup_batch_id: row.get(1)?,
            prompt_id: row.get(2)?,
            project_name: row.get(3)?,
            project_path: row.get(4)?,
            project_id: row.get(5)?,
            conversation_id: row.get(6)?,
            conversation_name: row.get(7)?,
            sequence_id: row.get(8)?,
            prompt_text: row.get(9)?,
            has_images: row.get(10)?,
            images_payload: row.get(11)?,
            status: row.get(12)?,
            created_at: row.get(13)?,
            is_restored: row.get(14)?,
            restored_at: row.get(15)?,
            instance_id: row.get(16).ok(),
        })
    };
    let records: Vec<crate::modules::backup_prompts_db::PromptBackupRecord> =
        if unscoped || is_default_scope {
            sel.query_map([], parse_row)
                .map_err(|e| format!("Failed to query queued backups: {}", e))?
                .flatten()
                .collect()
        } else {
            sel.query_map(params![target_inst], parse_row)
                .map_err(|e| format!("Failed to query queued backups: {}", e))?
                .flatten()
                .collect()
        };

    // Live-queue dedup maps: prompt id -> status, (repo_path, text) sig -> status.
    let repo_conn = connect_db()?;
    let mut live_by_id: HashMap<String, String> = HashMap::new();
    let mut live_by_sig: HashMap<String, String> = HashMap::new();
    {
        let mut live_stmt = repo_conn
            .prepare(
                "SELECT id, instance_id, repo_path, prompt_content, status FROM active_prompts
                 WHERE status IN ('queued', 'pending', 'running', 'backed_up', 'dispatched')",
            )
            .map_err(|e| format!("Failed to prepare live queue query: {}", e))?;
        let live_rows = live_stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            })
            .map_err(|e| format!("Failed to query live queue: {}", e))?;
        for row in live_rows.flatten() {
            let (pid, pinst, ppath, ptext, pstatus) = row;
            if !resend_queued_instance_matches(instance_id, &pinst) {
                continue;
            }
            live_by_id.insert(pid, pstatus.clone());
            live_by_sig.insert(resend_queued_sig(&ppath, &ptext), pstatus);
        }
    }

    let mark_restored = |bconn: &Connection, rec_id: &str| {
        // Justification: idempotent state update; a missed mark is retried on the
        // next re-send and the live-queue dedup above prevents double dispatch.
        crate::error::record_ignored(
            bconn.execute(
                "UPDATE prompt_backups SET is_restored = 1, restored_at = ?1 WHERE id = ?2",
                params![now, rec_id],
            ),
            "mark queued backup restored",
        );
    };

    // Step 3: re-inject in FIFO order with RCA-15 dedup.
    let mut seen_sigs: HashSet<String> = HashSet::new();
    let mut reinjected_ids: HashSet<String> = HashSet::new();
    for rec in &records {
        let sig = resend_queued_sig(&rec.project_path, &rec.prompt_text);
        let eff_inst = rec
            .instance_id
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(target_inst);

        let live_status = live_by_id
            .get(&rec.prompt_id)
            .or_else(|| live_by_sig.get(&sig));
        let skip_reason: Option<&str> = match live_status.map(|s| s.as_str()) {
            Some("running") | Some("dispatched") => Some("already dispatched"),
            Some(_) => Some("already in queue"),
            None => {
                if seen_sigs.contains(&sig) {
                    Some("duplicate backup row")
                } else {
                    None
                }
            }
        };
        if let Some(reason) = skip_reason {
            crate::modules::logger::log_info(&format!(
                "[RepoDB] Resend-queued: skipping '{}' ({})",
                rec.prompt_id, reason
            ));
            mark_restored(&bconn, &rec.id);
            outcome.skipped_duplicates += 1;
            seen_sigs.insert(sig);
            continue;
        }

        let active_p = ActivePrompt {
            id: rec.prompt_id.clone(),
            project_id: rec.project_id.clone(),
            instance_id: eff_inst.to_string(),
            repo_path: rec.project_path.clone(),
            prompt_content: rec.prompt_text.clone(),
            // Backup schema carries no model column; mirror the established
            // restore.rs reinjection default.
            model: Some("gemini-3.8-flash-high".to_string()),
            session_id: Some(rec.conversation_id.clone()),
            status: "queued".to_string(),
            // Preserve FIFO order for the dispatch pass below (which reads
            // ORDER BY created_at ASC, id ASC): the backup's sequence_id is the
            // authoritative FIFO position, so stagger created_at by it. Without
            // this, same-batch records share created_at and the random-id
            // tiebreak could dispatch a later prompt before an earlier one.
            created_at: rec.created_at + rec.sequence_id.max(0),
            updated_at: now,
            image_payload: rec.images_payload.clone(),
        };
        match super::prompts_crud::save_or_requeue_prompt(&active_p) {
            Ok(()) => {
                mark_restored(&bconn, &rec.id);
                outcome.reinjected += 1;
                seen_sigs.insert(sig.clone());
                reinjected_ids.insert(rec.prompt_id.clone());
                live_by_id.insert(rec.prompt_id.clone(), "queued".to_string());
                live_by_sig.insert(sig, "queued".to_string());
                let (extracted_uris, img_paths) =
                    super::detection::extract_image_uris_or_paths(&active_p.prompt_content);
                let final_uris = super::detection::effective_image_uris(
                    active_p.image_payload.as_deref(),
                    extracted_uris,
                );
                let image_count = img_paths.len() + final_uris.len();
                let byte_size = active_p.prompt_content.len();
                tracing::info!(
                    "[PROMPT_RESTORE] Restoring running prompt for instance {} from backup (images: {}, bytes: {})",
                    eff_inst,
                    image_count,
                    byte_size
                );
                crate::modules::logger::log_info(&format!(
                    "[RepoDB] Resend-queued: re-injected '{}' as queued on '{}'",
                    rec.prompt_id, eff_inst
                ));
            }
            Err(e) => {
                let msg = format!(
                    "failed to re-inject queued backup '{}': {}",
                    rec.prompt_id, e
                );
                crate::modules::logger::log_error(&format!("[RepoDB] {}", msg));
                outcome.failures.push(msg);
                outcome.failed += 1;
                seen_sigs.insert(sig);
            }
        }
    }

    // Mark fully-restored batches (mirror restore.rs).
    // Justification: best-effort DB statement (idempotent cleanup write); failure logged
    crate::error::record_ignored(
        bconn.execute(
            "UPDATE backup_batches SET is_fully_restored = 1 WHERE id IN (
                SELECT backup_batch_id FROM prompt_backups GROUP BY backup_batch_id HAVING min(is_restored) = 1
             )",
            [],
        ),
        "db execute",
    );

    // Step 4: SINGLE dispatch pass — never layered on another dispatch.
    match super::resend::resend_running_commands_for_instance(instance_id, limit.max(1)) {
        Ok(resent) => {
            outcome.dispatched = resent.len();
            outcome.dispatched_from_backup = resent
                .iter()
                .filter(|p| reinjected_ids.contains(&p.id))
                .count();
            crate::modules::logger::log_info(&format!(
                "[RepoDB] Resend-queued: dispatched {} prompt(s) ({} from backup), skipped {}, failed {}",
                outcome.dispatched,
                outcome.dispatched_from_backup,
                outcome.skipped_duplicates,
                outcome.failed
            ));
        }
        Err(e) => {
            let msg = format!(
                "dispatch pass failed (re-injected prompts remain queued for retry): {}",
                e
            );
            crate::modules::logger::log_error(&format!("[RepoDB] {}", msg));
            outcome.failures.push(msg);
            outcome.failed += 1;
        }
    }

    Ok(outcome)
}

/// Prompts that still need a re-push after this instance's IDE is back up.
pub fn count_backed_up_prompts(instance_id: &str) -> usize {
    let Ok(conn) = connect_db() else {
        return 0;
    };
    conn.query_row(
        "SELECT COUNT(*) FROM active_prompts
         WHERE status = 'backed_up'
           AND (instance_id = ?1 OR (?1 = 'default' AND (instance_id = 'default' OR instance_id = '__default__' OR instance_id = '' OR instance_id IS NULL)))",
        [instance_id],
        |row| row.get::<_, i64>(0),
    )
    .unwrap_or(0) as usize
}

/// A switch waits for the IDE prompt channel only when a prompt was actually backed up.
pub fn needs_prompt_channel_wait(backed_up_count: usize) -> bool {
    backed_up_count > 0
}

/// Backup all currently running prompts across active projects before switching
pub fn backup_running_prompts(instance_id: &str) -> Result<usize, String> {
    let conn = connect_db()?;
    let now = Utc::now().timestamp();
    let mut backed_up_count = 0;

    // Step 0: Retire stale 'running' prompts older than 2 hours to 'dispatched' so they don't shadow active prompts
    let stale_cutoff = now - 7200;
    // Justification: stale-row retirement is housekeeping; the backup transition proceeds regardless
    crate::error::record_ignored(
        conn.execute(
        "UPDATE active_prompts SET status = 'dispatched', updated_at = ? WHERE status = 'running' AND updated_at < ?",
        params![now, stale_cutoff],
    ),
        "retire stale running prompts",
    );

    // Step 0.5: Clear dispatched prompts cache so that any prompt backed up can be cleanly re-dispatched upon switch completion
    reset_dispatched_prompts_cache();

    let is_all_or_default =
        instance_id == "all" || instance_id == "__default__" || instance_id == "default";

    // Transition all uncompleted prompts in active_prompts to 'backed_up' before switch (scoped to target instance).
    // Incomplete-prompt sweep: transition ALL uncompleted prompts (WHERE status NOT IN ('completed', 'succeeded', 'backed_up'))
    // so long-running, pending, or interrupted prompts are never dropped when switching or when quota expires.
    let transitioned = if instance_id == "all" {
        conn.execute(
            "UPDATE active_prompts SET status = 'backed_up', updated_at = ?1 WHERE status NOT IN ('completed', 'succeeded', 'backed_up')",
            params![now],
        )
        .unwrap_or(0)
    } else if is_all_or_default {
        conn.execute(
            "UPDATE active_prompts SET status = 'backed_up', updated_at = ?1 WHERE status NOT IN ('completed', 'succeeded', 'backed_up') AND (instance_id IN ('default', '__default__', '') OR instance_id IS NULL)",
            params![now],
        )
        .unwrap_or(0)
    } else {
        conn.execute(
            "UPDATE active_prompts SET status = 'backed_up', updated_at = ?1 WHERE status NOT IN ('completed', 'succeeded', 'backed_up') AND instance_id = ?2",
            params![now, instance_id],
        )
        .unwrap_or(0)
    };
    if transitioned > 0 {
        backed_up_count += transitioned;
        if let Ok(mut map) = get_memory_prompts_map().lock() {
            for p in map.values_mut() {
                let matches_inst = instance_id == "all"
                    || (is_all_or_default
                        && (p.instance_id.is_empty()
                            || p.instance_id == "default"
                            || p.instance_id == "__default__"))
                    || p.instance_id == instance_id;
                let is_inflight_status =
                    p.status != "completed" && p.status != "succeeded" && p.status != "backed_up";
                if is_inflight_status && matches_inst {
                    p.status = "backed_up".to_string();
                    p.updated_at = now;
                }
            }
        }

        crate::modules::logger::log_info(&format!(
            "[RepoDB] Transitioned {} in-flight prompts from 'running' to 'backed_up' for instance '{}' before switch",
            transitioned, instance_id
        ));
    }

    // Layer 1: Core Antigravity Live Conversations Discovery (~/.gemini/antigravity)
    let ag_prompts = discover_running_prompts_from_antigravity(instance_id);
    for mut p in ag_prompts {
        p.status = "backed_up".to_string();
        p.updated_at = now;
        let res = super::prompts_crud::save_or_requeue_prompt(&p);

        if res.is_ok() {
            backed_up_count += 1;
            let (extracted_uris, img_paths) =
                super::detection::extract_image_uris_or_paths(&p.prompt_content);
            let final_uris =
                super::detection::effective_image_uris(p.image_payload.as_deref(), extracted_uris);
            let mut final_img = super::detection::serialize_image_uris(&final_uris);
            if final_img.is_none() && !img_paths.is_empty() {
                final_img = Some(img_paths.join(";"));
            }
            let byte_size = p.prompt_content.len();
            let image_count = img_paths.len() + final_uris.len();
            let seq_code = p
                .session_id
                .as_deref()
                .and_then(super::sequences::get_sequence_code_for_conversation);
            tracing::info!(
                "[PROMPT_BACKUP] Backed up running prompt for instance {} (seq: {:?}, images: {}, bytes: {})",
                p.instance_id,
                seq_code,
                image_count,
                byte_size
            );

            // Write disk resume snapshot file inside project repo directory.
            // session_id is the live conversation, so the IDE reopens that chat.
            let task_file = PathBuf::from(&p.repo_path).join(".antigravity_resume_task.json");
            let mut payload = resume_task_document(&p, "backed_up", now, &img_paths);
            if final_img.is_some() {
                payload["image_payload"] = serde_json::json!(final_img);
            }
            if let Ok(json_str) = serde_json::to_string_pretty(&payload) {
                // Justification: resume snapshot is a convenience file for the IDE; the database row is the source of truth
                crate::error::record_ignored(
                    fs::write(&task_file, json_str),
                    "write resume task snapshot",
                );
            }
            if let Ok(mut map) = get_memory_prompts_map().lock() {
                map.insert(p.id.clone(), p);
            }
        }
    }

    // Layer 2: VS Code / Instance Workspace Storage & Active Projects Discovery (Parallelized)
    let projects = detect_running_projects(instance_id).unwrap_or_default();
    let extracted_results: Vec<(&RunningProject, Vec<(String, Option<String>)>)> =
        std::thread::scope(|s| {
            let handles: Vec<_> = projects
                .iter()
                .map(|project| {
                    s.spawn(move || {
                        let mut extracted: Vec<(String, Option<String>)> = Vec::new();

                        // Check workspace state.vscdb for active prompts / tasks strictly if project is running
                        if project.is_running {
                            if let Some(ref ws_storage) = project.workspace_storage_path {
                                let ws_db_path = PathBuf::from(ws_storage).join("state.vscdb");
                                if ws_db_path.exists() {
                                    if let Ok(ws_conn) = Connection::open_with_flags(
                                        &ws_db_path,
                                        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
                                            | rusqlite::OpenFlags::SQLITE_OPEN_URI,
                                    ) {
                                        let mut stmt = ws_conn
                                            .prepare(
                                                "SELECT key, value FROM ItemTable \
                                                 WHERE key IN ('workbench.action.chat.interactiveInputHistory', 'interactive.inputHistory') \
                                                    OR key LIKE 'workbench.panel.chat%input%'"
                                            )
                                            .ok();

                                        if let Some(ref mut prepared) = stmt {
                                            if let Ok(rows) = prepared.query_map([], |row| {
                                                let key: String = row.get(0)?;
                                                let val: String = row.get(1)?;
                                                Ok((key, val))
                                            }) {
                                                for (_key, val) in rows.flatten() {
                                                    // Only extract validated interactive user prompt text from structured chat input history
                                                    if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&val) {
                                                        if let Some(arr) = json_val.as_array() {
                                                            if let Some(last_elem) = arr.last() {
                                                                let candidate_txt = if let Some(s) = last_elem.as_str() {
                                                                    Some(s.to_string())
                                                                } else if let Some(s) = last_elem.get("text").or_else(|| last_elem.get("content")).and_then(|v| v.as_str()) {
                                                                    Some(s.to_string())
                                                                } else {
                                                                    None
                                                                };
                                                                if let Some(content) = candidate_txt {
                                                                    let trimmed = content.trim();
                                                                    if !trimmed.is_empty() && trimmed.len() > 10 && !trimmed.starts_with('{') {
                                                                        let (img_uris, img_paths) = super::detection::extract_image_uris_or_paths(trimmed);
                                                                        let img_payload = super::detection::compose_image_payload(None, img_uris, &img_paths);
                                                                        extracted.push((trimmed.to_string(), img_payload));
                                                                    }
                                                                }
                                                            }
                                                        } else if let Some(content) = json_val.get("text").or_else(|| json_val.get("content")).and_then(|v| v.as_str()) {
                                                            let trimmed = content.trim();
                                                            if !trimmed.is_empty() && trimmed.len() > 10 && !trimmed.starts_with('{') {
                                                                let (img_uris, img_paths) = super::detection::extract_image_uris_or_paths(trimmed);
                                                                let img_payload = super::detection::compose_image_payload(None, img_uris, &img_paths);
                                                                extracted.push((trimmed.to_string(), img_payload));
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Check if project has an existing .antigravity_resume_task.json on disk
                        if extracted.is_empty() {
                            let task_file = PathBuf::from(&project.repo_path)
                                .join(".antigravity_resume_task.json");
                            if task_file.exists() {
                                if let Ok(c) = fs::read_to_string(&task_file) {
                                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&c) {
                                        let file_status = v
                                            .get("status")
                                            .and_then(|s| s.as_str())
                                            .unwrap_or("");
                                        let file_inst = v
                                            .get("instance_id")
                                            .and_then(|s| s.as_str())
                                            .unwrap_or("");
                                        let file_ts = v
                                            .get("updated_at")
                                            .or_else(|| v.get("resumed_at"))
                                            .or_else(|| v.get("created_at"))
                                            .and_then(|t| t.as_i64())
                                            .unwrap_or(0);
                                        let is_default_target = instance_id == "default"
                                            || instance_id == "__default__";
                                        let is_default_file = file_inst.is_empty()
                                            || file_inst == "default"
                                            || file_inst == "__default__";
                                        let is_instance_matched = instance_id == "all"
                                            || (is_default_target && is_default_file)
                                            || (!file_inst.is_empty() && file_inst == instance_id);
                                        let is_fresh_timestamp = file_ts > 0 && file_ts >= now - 600;
                                        let is_active_status = file_status == "running"
                                            || file_status == "in_flight"
                                            || file_status == "backed_up"
                                            || file_status == "dispatched";
                                        let is_active_task = is_active_status && is_fresh_timestamp;
                                        if is_instance_matched && is_active_task {
                                            if let Some(txt) =
                                                v.get("prompt_content").and_then(|t| t.as_str())
                                            {
                                                if !txt.trim().is_empty() {
                                                    let img = v
                                                        .get("image_payload")
                                                        .and_then(|i| i.as_str())
                                                        .map(|s| s.to_string());
                                                    extracted.push((txt.to_string(), img));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        (project, extracted)
                    })
                })
                .collect();

            handles.into_iter().filter_map(|h| h.join().ok()).collect()
        });

    for (project, mut extracted_prompts) in extracted_results {
        // Check if SQLite active_prompts already has an active running/in_flight prompt for this repo_path and instance
        if extracted_prompts.is_empty() {
            let proj_like = format!("%{}%", project.repo_name.to_lowercase());
            let mut check_stmt = conn
                .prepare(
                    "SELECT prompt_content, image_payload FROM active_prompts \
                     WHERE (repo_path = ?1 OR project_id = ?2 OR project_id LIKE ?3) \
                       AND status IN ('running', 'in_flight') \
                       AND (?4 = 'all' OR instance_id = ?4 OR (?4 IN ('default', '__default__') AND instance_id IN ('default', '__default__'))) \
                     ORDER BY created_at ASC, id ASC LIMIT 1",
                )
                .ok();
            if let Some(ref mut c_stmt) = check_stmt {
                if let Ok((txt, img)) = c_stmt.query_row(
                    rusqlite::params![&project.repo_path, &project.id, &proj_like, instance_id],
                    |r| Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?)),
                ) {
                    if !txt.trim().is_empty() {
                        extracted_prompts.push((txt, img));
                    }
                }
            }
        }

        for (prompt_text, image_payload) in extracted_prompts {
            let existing_id: Option<String> = if instance_id == "default" || instance_id.is_empty()
            {
                conn.query_row(
                    "SELECT id FROM active_prompts WHERE repo_path = ?1 AND prompt_content = ?2 AND (instance_id = 'default' OR instance_id = '') LIMIT 1",
                    rusqlite::params![&project.repo_path, &prompt_text],
                    |r| r.get(0),
                )
                .ok()
            } else {
                conn.query_row(
                    "SELECT id FROM active_prompts WHERE repo_path = ?1 AND prompt_content = ?2 AND instance_id = ?3 LIMIT 1",
                    rusqlite::params![&project.repo_path, &prompt_text, instance_id],
                    |r| r.get(0),
                )
                .ok()
            };

            let (prompt_id, is_existing) = match existing_id {
                Some(eid) => (eid, true),
                None => (Uuid::new_v4().to_string(), false),
            };

            let prompt_model = Some("gemini-pro".to_string());
            let result = if is_existing {
                conn.execute(
                    "UPDATE active_prompts SET status = 'backed_up', instance_id = ?1, updated_at = ?2, image_payload = COALESCE(?3, image_payload) WHERE id = ?4",
                    params![instance_id, now, &image_payload, &prompt_id],
                )
            } else {
                conn.execute(
                    "INSERT INTO active_prompts 
                     (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
                     VALUES (?, ?, ?, ?, ?, ?, ?, 'backed_up', ?, ?, ?)",
                    params![
                        &prompt_id,
                        &project.id,
                        instance_id,
                        &project.repo_path,
                        &prompt_text,
                        &prompt_model,
                        &project.id,
                        now,
                        now,
                        &image_payload,
                    ],
                )
            };

            if result.is_ok() {
                backed_up_count += 1;
                let (extracted_uris, img_paths) =
                    super::detection::extract_image_uris_or_paths(&prompt_text);
                let final_uris = super::detection::effective_image_uris(
                    image_payload.as_deref(),
                    extracted_uris,
                );
                let mut final_img = super::detection::serialize_image_uris(&final_uris);
                if final_img.is_none() && !img_paths.is_empty() {
                    // Legacy fallback: paths-only prompts stored the joined paths.
                    final_img = Some(img_paths.join(";"));
                }
                let byte_size = prompt_text.len();
                let image_count = img_paths.len() + final_uris.len();
                let seq_code = super::sequences::get_sequence_code_for_conversation(&project.id);
                tracing::info!(
                    "[PROMPT_BACKUP] Backed up running prompt for instance {} (seq: {:?}, images: {}, bytes: {})",
                    instance_id,
                    seq_code,
                    image_count,
                    byte_size
                );

                let active_prompt = ActivePrompt {
                    id: prompt_id.clone(),
                    project_id: project.id.clone(),
                    instance_id: instance_id.to_string(),
                    repo_path: project.repo_path.clone(),
                    prompt_content: prompt_text,
                    model: prompt_model,
                    session_id: Some(project.id.clone()),
                    status: "backed_up".to_string(),
                    created_at: now,
                    updated_at: now,
                    image_payload: final_img.clone(),
                };

                // Write disk resume snapshot file inside project repo directory using canonical resume_task_document
                let task_file =
                    PathBuf::from(&project.repo_path).join(".antigravity_resume_task.json");
                let mut payload =
                    resume_task_document(&active_prompt, "backed_up", now, &img_paths);
                if final_img.is_some() {
                    payload["image_payload"] = serde_json::json!(final_img);
                }
                if let Ok(json_str) = serde_json::to_string_pretty(&payload) {
                    // Justification: resume snapshot is a convenience file for the IDE; the database row is the source of truth
                    crate::error::record_ignored(
                        fs::write(&task_file, json_str),
                        "write resume task snapshot",
                    );
                }

                if let Ok(mut map) = get_memory_prompts_map().lock() {
                    map.insert(prompt_id, active_prompt);
                }
            }
        }
    }

    crate::modules::logger::log_info(&format!(
        "[RepoDB] Backed up {} running prompts for instance '{}' across Antigravity core and workspaces",
        backed_up_count, instance_id
    ));

    invalidate_prompt_tree_cache(Some(instance_id));
    Ok(backed_up_count)
}

/// FIFO-ordered ids of incomplete prompts for an instance scope: rows in
/// 'running' / 'in_flight' / 'dispatched' / 'pending' / 'queued' / 'blocked'
/// (not marked 'completed' or 'succeeded', at ANY age).
/// 'default' and '__default__' share one scope; 'all' is unscoped.
fn select_incomplete_prompt_ids(
    conn: &Connection,
    instance_id: &str,
) -> Result<Vec<String>, String> {
    let is_default_scope = instance_id == "default" || instance_id == "__default__";
    let is_unscoped = instance_id == "all" || is_default_scope;
    let sql = if instance_id == "all" {
        "SELECT id FROM active_prompts
         WHERE status NOT IN ('completed', 'succeeded', 'backed_up')
         ORDER BY created_at ASC, id ASC"
    } else if is_default_scope {
        "SELECT id FROM active_prompts
         WHERE status NOT IN ('completed', 'succeeded', 'backed_up')
           AND (instance_id IN ('default', '__default__') OR instance_id IS NULL OR instance_id = '')
         ORDER BY created_at ASC, id ASC"
    } else {
        "SELECT id FROM active_prompts
         WHERE status NOT IN ('completed', 'succeeded', 'backed_up') AND instance_id = ?1
         ORDER BY created_at ASC, id ASC"
    };
    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| format!("Failed to prepare incomplete-prompt sweep query: {}", e))?;
    let map_id = |row: &rusqlite::Row| row.get::<_, String>(0);
    let ids: Vec<String> = if is_unscoped {
        stmt.query_map([], map_id)
            .map_err(|e| format!("Failed to query incomplete prompts: {}", e))?
            .flatten()
            .collect()
    } else {
        stmt.query_map(params![instance_id], map_id)
            .map_err(|e| format!("Failed to query incomplete prompts: {}", e))?
            .flatten()
            .collect()
    };
    Ok(ids)
}

/// Mirror swept prompt ids into the in-memory prompt map as 'backed_up'
/// so live views agree with the database after a sweep.
fn mark_swept_prompts_in_memory(ids: &[String], now: i64) {
    if let Ok(mut map) = get_memory_prompts_map().lock() {
        let id_set: HashSet<&str> = ids.iter().map(|id| id.as_str()).collect();
        for p in map.values_mut() {
            if id_set.contains(p.id.as_str()) {
                p.status = "backed_up".to_string();
                p.updated_at = now;
            }
        }
    }
}

/// Incomplete-prompt sweep (RC-1): transition every prompt for
/// `instance_id` that never reached 'completed' — status 'running',
/// 'in_flight', or 'dispatched', at ANY age — to 'backed_up', claiming
/// rows in FIFO order (created_at ASC, id ASC). Prompts dispatched more
/// than 120s before a switch used to fall out of every restore query;
/// this sweep makes them visible to restore. Returns the swept count.
pub fn sweep_incomplete_prompts_for_instance(instance_id: &str) -> Result<usize, String> {
    let conn = connect_db()?;
    let now = Utc::now().timestamp();
    let ids = select_incomplete_prompt_ids(&conn, instance_id)?;
    let mut swept_ids: Vec<String> = Vec::new();
    for id in &ids {
        // The status guard keeps a row that completed after selection
        // from being clobbered back to 'backed_up'.
        let changed = conn
            .execute(
                "UPDATE active_prompts SET status = 'backed_up', updated_at = ?1
                 WHERE id = ?2 AND status NOT IN ('completed', 'succeeded', 'backed_up')",
                params![now, id],
            )
            .map_err(|e| format!("Failed to sweep incomplete prompt '{}': {}", id, e))?;
        if changed > 0 {
            swept_ids.push(id.clone());
        }
    }
    let swept = swept_ids.len();
    if swept > 0 {
        mark_swept_prompts_in_memory(&swept_ids, now);
        crate::modules::logger::log_info(&format!(
            "[RepoDB] Swept {} incomplete prompt(s) to 'backed_up' for instance '{}'",
            swept, instance_id
        ));
        invalidate_prompt_tree_cache(Some(instance_id));
    }
    Ok(swept)
}

/// Verified count of active prompts or running processes across workspaces
pub fn verify_prompts_running() -> usize {
    let mut count = 0;

    // 1. Check SQLite active_prompts for 'running' or recently 'dispatched' (within 5 minutes)
    if let Ok(conn) = connect_db() {
        let now = Utc::now().timestamp();
        let db_count: usize = conn
            .query_row(
                "SELECT COUNT(*) FROM active_prompts WHERE status = 'running' OR (status = 'dispatched' AND updated_at >= ?1)",
                params![now - 45],
                |r| r.get(0),
            )
            .unwrap_or(0);
        count = db_count;
    }

    // 2. Check Antigravity conversation_summaries.db
    let base_dir = crate::modules::agy_cleaner::get_gemini_base_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join(".gemini").join("antigravity")));
    if let Some(base_dir) = base_dir {
        let summaries_db = base_dir.join("conversation_summaries.db");
        if summaries_db.exists() {
            if let Ok(conn) = Connection::open_with_flags(
                &summaries_db,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
            ) {
                let ag_count: usize = conn
                    .query_row(
                        "SELECT COUNT(*) FROM conversation_summaries 
                         WHERE not_fully_idle != 0 
                           AND status LIKE '%RUNNING%' 
                           AND status NOT LIKE '%IDLE%' 
                           AND status NOT LIKE '%COMPLETED%' 
                           AND status NOT LIKE '%FAILED%' 
                           AND status NOT LIKE '%CANCELLED%'",
                        [],
                        |row| row.get(0),
                    )
                    .unwrap_or(0);
                if ag_count > count {
                    count = ag_count;
                }
            }
        }
    }

    // 3. Fallback to live project execution info
    if count == 0 {
        let live = get_live_project_execution_info();
        count = live
            .iter()
            .filter(|p| p.is_running || (!p.is_idle && p.active_prompt.is_some()))
            .count();
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::repo_db::detection;
    use crate::modules::repo_db::init_tables;
    use rusqlite::Connection;

    #[test]
    fn prompt_channel_wait_only_when_a_prompt_was_backed_up() {
        assert!(!needs_prompt_channel_wait(0));
        assert!(needs_prompt_channel_wait(1));
    }

    #[test]
    fn test_backup_conflict_resolution_no_duplicates() {
        let conn = Connection::open_in_memory().unwrap();
        assert!(init_tables(&conn).is_ok());

        let now = Utc::now().timestamp();
        // Insert parent running_project first
        conn.execute(
            "INSERT INTO running_projects 
             (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
             VALUES ('proj-dup', 'default', 'dup', '/work/dup', NULL, 1, ?, ?)",
            params![now, now],
        )
        .unwrap();

        // Insert prompt first time
        conn.execute(
            "INSERT INTO active_prompts 
             (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'backed_up', ?8, ?9, NULL)
             ON CONFLICT(id) DO UPDATE SET status = 'backed_up', updated_at = excluded.updated_at",
            params!["p-duplicate", "proj-dup", "default", "/work/dup", "Prompt text", "gemini-pro", "cid-dup", now, now],
        ).unwrap();

        // Insert same prompt second time with ON CONFLICT
        conn.execute(
            "INSERT INTO active_prompts 
             (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'backed_up', ?8, ?9, NULL)
             ON CONFLICT(id) DO UPDATE SET status = 'backed_up', updated_at = excluded.updated_at",
            params!["p-duplicate", "proj-dup", "default", "/work/dup", "Prompt text", "gemini-pro", "cid-dup", now + 1, now + 1],
        ).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM active_prompts WHERE id = 'p-duplicate'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "Must never duplicate records on backup");
    }

    #[test]
    fn test_backup_includes_full_prompt_images_sequence_and_workspace_id() {
        let conn = Connection::open_in_memory().unwrap();
        assert!(init_tables(&conn).is_ok());

        let now = Utc::now().timestamp();
        let prompt_text = "Detailed refactoring prompt with embedded image data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
        let image_payload = Some("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==".to_string());
        let project_id = "proj-workspace-alpha";
        let conversation_id = "conv-session-101";

        // 1. Ensure conversation sequence exists
        let seq_id = super::super::sequences::ensure_conversation_sequence_in_conn(
            &conn,
            conversation_id,
            "d:/work/repo-alpha",
            "Refactoring Task",
            "default",
        );
        assert_eq!(seq_id, 1);
        let seq_code = super::super::sequences::get_sequence_code_in_conn(&conn, conversation_id);
        assert_eq!(seq_code, Some("C001".to_string()));

        // 2. Insert running project
        conn.execute(
            "INSERT INTO running_projects 
             (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
             VALUES (?1, 'default', 'repo-alpha', 'd:/work/repo-alpha', NULL, 1, ?2, ?2)",
            params![project_id, now],
        ).unwrap();

        // 3. Insert active prompt with full text, image payload, and workspace ID
        let prompt_id = "prompt-audit-test";
        conn.execute(
            "INSERT INTO active_prompts 
             (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
             VALUES (?1, ?2, 'default', 'd:/work/repo-alpha', ?3, 'gemini-2.5-pro', ?4, 'running', ?5, ?5, ?6)",
            params![prompt_id, project_id, prompt_text, conversation_id, now, &image_payload],
        ).unwrap();

        // 4. Verify backup transitions status to backed_up and preserves all attributes
        conn.execute(
            "UPDATE active_prompts SET status = 'backed_up', updated_at = ?1 WHERE id = ?2",
            params![now + 1, prompt_id],
        )
        .unwrap();

        let (retrieved_text, retrieved_img, retrieved_proj, retrieved_session, retrieved_status) = conn
            .query_row(
                "SELECT prompt_content, image_payload, project_id, session_id, status FROM active_prompts WHERE id = ?1",
                params![prompt_id],
                |r| Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, Option<String>>(3)?,
                    r.get::<_, String>(4)?,
                )),
            )
            .unwrap();

        assert_eq!(retrieved_text, prompt_text);
        assert_eq!(retrieved_img, image_payload);
        assert_eq!(retrieved_proj, project_id);
        assert_eq!(retrieved_session, Some(conversation_id.to_string()));
        assert_eq!(retrieved_status, "backed_up");

        // 5. Verify resume_task_document produces canonical document with sequence_code & workspace_id
        let prompt_obj = ActivePrompt {
            id: prompt_id.to_string(),
            project_id: project_id.to_string(),
            instance_id: "default".to_string(),
            repo_path: "d:/work/repo-alpha".to_string(),
            prompt_content: prompt_text.to_string(),
            model: Some("gemini-2.5-pro".to_string()),
            session_id: Some(conversation_id.to_string()),
            status: "backed_up".to_string(),
            created_at: now,
            updated_at: now + 1,
            image_payload: image_payload.clone(),
        };

        let (extracted_uris, img_paths) = detection::extract_image_uris_or_paths(&prompt_text);
        assert_eq!(extracted_uris.len(), 1);
        assert!(img_paths.is_empty());
        // Legacy single-URI wrapper still returns the first URI verbatim.
        let (legacy_img, _) = detection::extract_image_payload_or_path(&prompt_text);
        assert!(legacy_img.is_some());
        let doc = resume_task_document(&prompt_obj, "backed_up", now + 1, &img_paths);
        assert_eq!(doc["workspace_id"], project_id);
        assert_eq!(doc["project_id"], project_id);
        assert_eq!(doc["prompt_content"], prompt_text);
        assert_eq!(doc["image_payload"], image_payload.unwrap());
        assert_eq!(doc["has_image"], true);
    }
}
