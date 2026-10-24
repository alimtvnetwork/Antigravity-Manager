//! Post-switch prompt resume timer.
//!
//! After an auto-switch or account switch, the most recently active
//! running prompt is resumed on the new account once the IDE's prompt
//! channel is up (single-prompt resume; every other backed-up prompt
//! stays backed up). The delay
//! is configurable (`post_switch_resume_delay_seconds`, default 10s) and the
//! pending resume is cancellable: scheduling a newer resume for the same
//! instance supersedes the old one, and user/manual activity can cancel it via
//! [`cancel_post_switch_resume`] before it fires.
//!
//! RCA-15 single-point orchestration: the timer rides the EXISTING
//! STAGE_5_RESTORE restore unit (`push_backed_up_prompts_after_channel`) — it
//! does NOT add a parallel backup/restore pipeline, so backed-up prompts keep
//! their `running -> backed_up -> dispatched` status transitions, keep
//! bypassing the dispatch cache, and are never re-injected twice.
//!
//! RC-7: a scheduled task is also persisted to `post_switch_resume_state.json`
//! in the app data dir (the same atomic-write mechanism as the instance
//! registry), so an app restart cannot lose it: [`rearm_persisted_resumes`]
//! re-arms any persisted task whose restore has not completed yet.

use chrono::Utc;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;
use tauri::async_runtime::JoinHandle;
use tauri::Emitter;

use crate::modules::audit::{
    record_interaction, InteractionActor, InteractionTrigger, RecordInteractionInput,
};

/// Fallback resume delay when the config value cannot be read or is zero.
pub const DEFAULT_POST_SWITCH_RESUME_DELAY_SECONDS: u64 = 10;

/// Pause before the single retry when the first push landed zero prompts
/// (preserves the pre-existing STAGE_5_RESTORE behavior).
const RESTORE_RETRY_DELAY_SECONDS: u64 = 7;

/// Summary of a prompt pending post-switch re-injection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingPromptPreview {
    pub id: String,
    pub project_id: String,
    pub repo_name: String,
    pub repo_path: String,
    pub prompt_summary: String,
    pub full_prompt: String,
    pub created_at: i64,
    pub has_image: bool,
}

/// Information returned to frontend for a pending post-switch resume task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledResumeEntry {
    pub instance_id: String,
    pub scheduled_at_ms: u64,
    pub fire_at_ms: u64,
    pub remaining_seconds: i64,
    pub total_delay_seconds: u64,
    pub prompt_count: usize,
    pub prompts: Vec<PendingPromptPreview>,
}

struct InternalResumeTask {
    handle: JoinHandle<()>,
    entry: ScheduledResumeEntry,
}

/// Pending resume tasks keyed by instance id. At most one pending resume per
/// instance; scheduling a new one aborts the previous one.
static POST_SWITCH_RESUME_TASKS: Lazy<Mutex<HashMap<String, InternalResumeTask>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

// ---------------------------------------------------------------------------
// Persistence (RC-7): scheduled resumes survive an app restart.
//
// The state lives in a module-owned JSON file inside the app data dir and is
// written with the same atomic-write mechanism as the instance registry
// (`crate::utils::fs::write_atomic`). The in-memory timer above stays the
// runtime source of truth; the file is only the restart-recovery record.
// ---------------------------------------------------------------------------

/// File name (inside the app data dir) holding scheduled post-switch resumes.
const PERSISTED_RESUME_STATE_FILE: &str = "post_switch_resume_state.json";

/// One scheduled resume as persisted to disk: everything needed to re-arm
/// the timer after a restart (target instance, backed-up count, deadline).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedResumeTask {
    instance_id: String,
    backed_up_count: usize,
    scheduled_at_ms: u64,
    fire_at_ms: u64,
}

/// On-disk shape of [`PERSISTED_RESUME_STATE_FILE`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct PersistedResumeState {
    #[serde(default)]
    tasks: Vec<PersistedResumeTask>,
}

fn persisted_resume_state_path() -> Result<PathBuf, String> {
    Ok(crate::modules::account::get_data_dir()?.join(PERSISTED_RESUME_STATE_FILE))
}

/// Load the persisted resume state. A missing file is an empty state; a
/// corrupt file is logged and treated as empty so a bad state file can never
/// block the in-memory scheduler.
fn load_persisted_resume_state() -> PersistedResumeState {
    let path = match persisted_resume_state_path() {
        Ok(path) => path,
        Err(err) => {
            crate::modules::logger::log_warn(&format!(
                "[PostSwitchResume] Cannot resolve persisted resume state path: {err}"
            ));
            return PersistedResumeState::default();
        }
    };
    if !path.exists() {
        return PersistedResumeState::default();
    }
    let parsed = fs::read_to_string(&path)
        .map_err(|e| e.to_string())
        .and_then(|content| serde_json::from_str(&content).map_err(|e| e.to_string()));
    match parsed {
        Ok(state) => state,
        Err(err) => {
            crate::modules::logger::log_warn(&format!(
                "[PostSwitchResume] Failed to read persisted resume state at {path:?}: {err}; treating as empty"
            ));
            PersistedResumeState::default()
        }
    }
}

fn save_persisted_resume_state(state: &PersistedResumeState) -> Result<(), String> {
    let path = persisted_resume_state_path()?;
    let json = serde_json::to_string_pretty(state)
        .map_err(|e| format!("Failed to serialize post-switch resume state: {}", e))?;
    crate::utils::fs::write_atomic(&path, json.as_bytes())
        .map_err(|e| format!("Failed to write post-switch resume state: {}", e))
}

/// Persist (or replace) the persisted record for `entry`'s instance.
fn persist_resume_task(entry: &ScheduledResumeEntry) {
    let mut state = load_persisted_resume_state();
    state.tasks.retain(|t| t.instance_id != entry.instance_id);
    state.tasks.push(PersistedResumeTask {
        instance_id: entry.instance_id.clone(),
        backed_up_count: entry.prompt_count,
        scheduled_at_ms: entry.scheduled_at_ms,
        fire_at_ms: entry.fire_at_ms,
    });
    // Justification: persistence is best-effort restart insurance; the in-memory timer still fires this run even if the state file cannot be written
    crate::error::record_ignored(
        save_persisted_resume_state(&state),
        "persist post-switch resume task",
    );
}

/// Drop the persisted record for `instance_id`, if any.
fn clear_persisted_resume_task(instance_id: &str) {
    let mut state = load_persisted_resume_state();
    let count_before = state.tasks.len();
    state.tasks.retain(|t| t.instance_id != instance_id);
    if state.tasks.len() == count_before {
        return;
    }
    // Justification: best-effort cleanup; a stale record is re-validated against live backed-up prompts on re-arm and dropped when the restore already completed
    crate::error::record_ignored(
        save_persisted_resume_state(&state),
        "clear persisted post-switch resume task",
    );
}

/// Collect previews of prompts currently waiting to be restored for `instance_id`
pub fn collect_pending_prompt_previews(instance_id: &str) -> Vec<PendingPromptPreview> {
    let mut previews = Vec::new();
    if let Ok(conn) = crate::modules::repo_db::connect_db() {
        let is_default = instance_id == "default" || instance_id == "__default__";
        let query = if is_default {
            "SELECT id, project_id, repo_path, prompt_content, created_at, image_payload \
             FROM active_prompts \
             WHERE (instance_id = 'default' OR instance_id = '__default__' OR instance_id = '') \
               AND status = 'backed_up' \
             ORDER BY created_at ASC, id ASC"
        } else {
            "SELECT id, project_id, repo_path, prompt_content, created_at, image_payload \
             FROM active_prompts \
             WHERE instance_id = ?1 \
               AND status = 'backed_up' \
             ORDER BY created_at ASC, id ASC"
        };
        if let Ok(mut stmt) = conn.prepare(query) {
            let map_fn = |row: &rusqlite::Row| {
                let id: String = row.get(0)?;
                let project_id: String = row.get(1)?;
                let repo_path: String = row.get(2)?;
                let prompt_content: String = row.get(3)?;
                let created_at: i64 = row.get(4)?;
                let image_payload: Option<String> = row.get(5)?;
                Ok((
                    id,
                    project_id,
                    repo_path,
                    prompt_content,
                    created_at,
                    image_payload,
                ))
            };
            let query_res = if is_default {
                stmt.query_map([], map_fn)
            } else {
                stmt.query_map([instance_id], map_fn)
            };
            if let Ok(rows) = query_res {
                for item in rows.flatten() {
                    let (id, project_id, repo_path, prompt_content, created_at, image_payload) =
                        item;
                    let repo_name = std::path::Path::new(&repo_path)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "project".to_string());
                    let prompt_summary = if prompt_content.chars().count() > 120 {
                        format!(
                            "{}...",
                            prompt_content.chars().take(120).collect::<String>()
                        )
                    } else {
                        prompt_content.clone()
                    };
                    previews.push(PendingPromptPreview {
                        id,
                        project_id,
                        repo_name,
                        repo_path,
                        prompt_summary,
                        full_prompt: prompt_content,
                        created_at,
                        has_image: image_payload.is_some(),
                    });
                }
            }
        }
    }
    previews
}

/// Query all pending post-switch resume tasks with up-to-date remaining seconds
pub fn get_pending_resumes() -> Vec<ScheduledResumeEntry> {
    let timers = POST_SWITCH_RESUME_TASKS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let now_ms = Utc::now().timestamp_millis() as u64;
    timers
        .values()
        .map(|task| {
            let mut entry = task.entry.clone();
            entry.remaining_seconds = if entry.fire_at_ms > now_ms {
                ((entry.fire_at_ms - now_ms) + 999) / 1000
            } else {
                0
            } as i64;
            entry
        })
        .collect()
}

/// Query pending resume for a specific instance
pub fn get_instance_pending_resume(instance_id: &str) -> Option<ScheduledResumeEntry> {
    let timers = POST_SWITCH_RESUME_TASKS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let now_ms = Utc::now().timestamp_millis() as u64;
    timers.get(instance_id).map(|task| {
        let mut entry = task.entry.clone();
        entry.remaining_seconds = if entry.fire_at_ms > now_ms {
            ((entry.fire_at_ms - now_ms) + 999) / 1000
        } else {
            0
        } as i64;
        entry
    })
}

/// Schedule the post-switch resume for `instance_id` after the configured
/// delay. Any previously pending resume for the same instance is cancelled
/// first, so stacked switches can never double-restore.
pub fn schedule_post_switch_resume(instance_id: String) {
    cancel_post_switch_resume(&instance_id, "superseded by a newer schedule");

    let delay = post_switch_resume_delay_seconds();
    let previews = collect_pending_prompt_previews(&instance_id);
    let prompt_count = previews.len();
    let now_ms = Utc::now().timestamp_millis() as u64;
    let fire_at_ms = now_ms + delay * 1000;

    let entry = ScheduledResumeEntry {
        instance_id: instance_id.clone(),
        scheduled_at_ms: now_ms,
        fire_at_ms,
        remaining_seconds: delay as i64,
        total_delay_seconds: delay,
        prompt_count,
        prompts: previews,
    };

    crate::modules::logger::log_info(&format!(
        "[PostSwitchResume] Scheduling prompt resume for instance '{instance_id}' in {delay}s ({prompt_count} prompt(s))"
    ));
    audit_resume_lifecycle(
        "scheduled",
        &instance_id,
        &format!("resume scheduled in {delay}s ({prompt_count} prompt(s))"),
    );

    if let Some(app_handle) = crate::modules::log_bridge::get_app_handle() {
        crate::error::record_ignored(
            app_handle.emit("prompts://resume-scheduled", &entry),
            "emit prompts://resume-scheduled",
        );
    }

    persist_resume_task(&entry);
    arm_resume_timer(entry);
}

/// CLI/headless Stage-5: persist, wait the settle delay, restore inline.
/// The GUI timer is in-process and dies when `agm` exits, so a CLI switch
/// must block here or the running prompt never comes back on the next account.
pub async fn run_post_switch_resume_after_delay(instance_id: &str) -> usize {
    let delay = post_switch_resume_delay_seconds();
    let previews = collect_pending_prompt_previews(instance_id);
    let prompt_count = previews.len();
    let now_ms = Utc::now().timestamp_millis() as u64;
    let entry = ScheduledResumeEntry {
        instance_id: instance_id.to_string(),
        scheduled_at_ms: now_ms,
        fire_at_ms: now_ms + delay * 1000,
        remaining_seconds: delay as i64,
        total_delay_seconds: delay,
        prompt_count,
        prompts: previews,
    };
    persist_resume_task(&entry);
    audit_resume_lifecycle(
        "scheduled",
        instance_id,
        &format!("CLI/headless resume scheduled in {delay}s ({prompt_count} prompt(s))"),
    );
    crate::modules::logger::log_info(&format!(
        "[PostSwitchResume] CLI/headless: waiting {delay}s then restoring on '{instance_id}' ({prompt_count} prompt(s))"
    ));
    tokio::time::sleep(Duration::from_secs(delay)).await;
    let restored = run_resume_with_retry(instance_id).await;
    clear_persisted_resume_task(instance_id);
    audit_resume_lifecycle(
        "fired",
        instance_id,
        &format!("CLI/headless resume fired; {restored} prompt(s) restored"),
    );
    restored
}

/// Spawn the resume timer for `entry` and register it, aborting any displaced
/// task for the same instance. Lock scope is sync-only: no .await while the
/// guard is held. Aborting a displaced handle covers the tiny race between
/// cancel and insert.
fn arm_resume_timer(entry: ScheduledResumeEntry) {
    let delay = entry.total_delay_seconds;
    let timer_instance_id = entry.instance_id.clone();
    let handle = tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(delay)).await;
        let restored = run_resume_with_retry(&timer_instance_id).await;
        // Remove our own registry entry; a cancel that lost the race finds nothing.
        remove_timer_entry(&timer_instance_id);
        // The resume ran to completion; its persisted record must not re-arm it.
        clear_persisted_resume_task(&timer_instance_id);
        emit_prompts_restored(&timer_instance_id, restored);
        audit_resume_lifecycle(
            "fired",
            &timer_instance_id,
            &format!("resume fired; {restored} prompt(s) restored"),
        );
    });
    let mut timers = POST_SWITCH_RESUME_TASKS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(displaced) = timers.insert(
        entry.instance_id.clone(),
        InternalResumeTask { handle, entry },
    ) {
        displaced.handle.abort();
    }
}

/// Cancel a pending post-switch resume for `instance_id`, if any. No-op when
/// nothing is pending. Call this when the user (or another switch) acts first
/// so the later timer cannot re-inject prompts that were already handled.
pub fn cancel_post_switch_resume(instance_id: &str, reason: &str) -> bool {
    let removed = {
        let mut timers = POST_SWITCH_RESUME_TASKS
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        timers.remove(instance_id)
    };
    // A cancel also drops the persisted record, so an app restart cannot
    // resurrect a resume that was already cancelled or superseded.
    clear_persisted_resume_task(instance_id);
    if let Some(task) = removed {
        task.handle.abort();
        crate::modules::logger::log_info(&format!(
            "[PostSwitchResume] Cancelled pending resume for instance '{instance_id}': {reason}"
        ));
        audit_resume_lifecycle("cancelled", instance_id, reason);
        if let Some(app_handle) = crate::modules::log_bridge::get_app_handle() {
            crate::error::record_ignored(
                app_handle.emit(
                    "prompts://resume-cancelled",
                    serde_json::json!({ "instance_id": instance_id }),
                ),
                "emit prompts://resume-cancelled",
            );
        }
        true
    } else {
        false
    }
}

/// Public alias for user-driven cancellations
pub fn cancel_scheduled_resume(instance_id: &str) -> bool {
    cancel_post_switch_resume(instance_id, "User requested cancellation")
}

/// Re-arm persisted post-switch resumes after an app restart (RC-7). Call
/// once during app startup. A persisted task is re-armed only when its
/// restore has not completed — i.e. backed-up prompts for the instance still
/// exist; otherwise the stale record is cleared. A deadline that passed
/// while the app was closed fires immediately.
pub fn rearm_persisted_resumes() {
    for task in load_persisted_resume_state().tasks {
        rearm_persisted_task(&task);
    }
}

fn rearm_persisted_task(task: &PersistedResumeTask) {
    let instance_id = task.instance_id.as_str();
    let is_already_armed = get_instance_pending_resume(instance_id).is_some();
    if is_already_armed {
        return;
    }
    let previews = collect_pending_prompt_previews(instance_id);
    if previews.is_empty() {
        crate::modules::logger::log_info(&format!(
            "[PostSwitchResume] Persisted resume for '{instance_id}' has no backed-up prompts left; restore already completed, clearing record"
        ));
        clear_persisted_resume_task(instance_id);
        return;
    }
    let now_ms = Utc::now().timestamp_millis() as u64;
    let entry = build_rearm_entry(task, previews, now_ms);
    crate::modules::logger::log_info(&format!(
        "[PostSwitchResume] Re-arming persisted resume for instance '{instance_id}' after restart ({}s remaining, {} prompt(s), {} backed up at schedule time)",
        entry.total_delay_seconds, entry.prompt_count, task.backed_up_count
    ));
    audit_resume_lifecycle(
        "rearmed",
        instance_id,
        &format!(
            "resume re-armed after restart ({}s remaining)",
            entry.total_delay_seconds
        ),
    );
    if let Some(app_handle) = crate::modules::log_bridge::get_app_handle() {
        crate::error::record_ignored(
            app_handle.emit("prompts://resume-scheduled", &entry),
            "emit prompts://resume-scheduled",
        );
    }
    arm_resume_timer(entry);
}

/// Build the registry entry for a re-armed persisted task. The original
/// deadline is kept; the delay is the remaining time until it (0 when the
/// deadline passed while the app was closed, so the resume fires at once).
fn build_rearm_entry(
    task: &PersistedResumeTask,
    previews: Vec<PendingPromptPreview>,
    now_ms: u64,
) -> ScheduledResumeEntry {
    let remaining_seconds = if task.fire_at_ms > now_ms {
        ((task.fire_at_ms - now_ms) + 999) / 1000
    } else {
        0
    };
    ScheduledResumeEntry {
        instance_id: task.instance_id.clone(),
        scheduled_at_ms: task.scheduled_at_ms,
        fire_at_ms: task.fire_at_ms,
        remaining_seconds: remaining_seconds as i64,
        total_delay_seconds: remaining_seconds,
        prompt_count: previews.len(),
        prompts: previews,
    }
}

/// Immediately force-dispatch pending prompts for an instance without waiting for timer expiry
pub async fn force_dispatch_resume(
    _app: &tauri::AppHandle,
    instance_id: &str,
) -> Result<(), crate::error::AppError> {
    cancel_post_switch_resume(instance_id, "force-dispatched by user");
    let restored = run_resume_with_retry(instance_id).await;
    remove_timer_entry(instance_id);
    emit_prompts_restored(instance_id, restored);
    audit_resume_lifecycle(
        "force_dispatched",
        instance_id,
        &format!("force resume dispatched; {restored} prompt(s) restored"),
    );
    Ok(())
}

/// Configured post-switch resume delay in seconds (default 10). A missing or
/// zero value falls back to the default; the fallback is logged, never silent.
fn post_switch_resume_delay_seconds() -> u64 {
    let configured = match crate::modules::config::load_app_config() {
        Ok(cfg) => u64::from(cfg.auto_profile_switcher.post_switch_resume_delay_seconds),
        Err(err) => {
            crate::modules::logger::log_warn(&format!(
                "[PostSwitchResume] Failed to read post_switch_resume_delay_seconds: {err}; using {DEFAULT_POST_SWITCH_RESUME_DELAY_SECONDS}s default"
            ));
            DEFAULT_POST_SWITCH_RESUME_DELAY_SECONDS
        }
    };
    if configured == 0 {
        crate::modules::logger::log_warn(&format!(
            "[PostSwitchResume] post_switch_resume_delay_seconds is 0; using {DEFAULT_POST_SWITCH_RESUME_DELAY_SECONDS}s default"
        ));
        DEFAULT_POST_SWITCH_RESUME_DELAY_SECONDS
    } else {
        configured
    }
}

/// Run the existing STAGE_5_RESTORE restore unit once; when zero prompts
/// landed, wait and retry once. Returns the number of restored prompts.
async fn run_resume_with_retry(instance_id: &str) -> usize {
    let first = spawn_restore_once(instance_id).await;
    if first > 0 {
        return first;
    }
    crate::modules::logger::log_warn(&format!(
        "[PostSwitchResume] No prompt landed for '{instance_id}' on the first push; retrying once in {RESTORE_RETRY_DELAY_SECONDS}s"
    ));
    tokio::time::sleep(Duration::from_secs(RESTORE_RETRY_DELAY_SECONDS)).await;
    spawn_restore_once(instance_id).await
}

/// Run the single existing restore unit (wait for the prompt channel,
/// rehydrate the snapshot, dispatch once) on a blocking thread. This is the
/// same code path as the legacy post-launch restore — not a parallel pipeline.
async fn spawn_restore_once(instance_id: &str) -> usize {
    let owned = instance_id.to_string();
    match tokio::task::spawn_blocking(move || {
        super::workspace::push_backed_up_prompts_after_channel(&owned)
    })
    .await
    {
        Ok(count) => count,
        Err(err) => {
            crate::modules::logger::log_error(&format!(
                "[PostSwitchResume] Resume task failed for '{instance_id}': {err}"
            ));
            0
        }
    }
}

/// Emit `prompts://restored` so the frontend can show the resume indicator.
/// Best-effort: a dropped event only skips the UI refresh.
fn emit_prompts_restored(instance_id: &str, restored_count: usize) {
    let Some(handle) = crate::modules::log_bridge::get_app_handle() else {
        crate::modules::logger::log_warn(&format!(
            "[PostSwitchResume] No app handle available; skipping prompts://restored event for '{instance_id}'"
        ));
        return;
    };
    #[derive(serde::Serialize, Clone)]
    struct PromptsRestoredPayload {
        instance_id: String,
        restored_count: usize,
    }
    // Justification: best-effort frontend event; a dropped event only skips a UI refresh
    crate::error::record_ignored(
        handle.emit(
            "prompts://restored",
            PromptsRestoredPayload {
                instance_id: instance_id.to_string(),
                restored_count,
            },
        ),
        "emit prompts://restored",
    );
}

/// Record a resume-lifecycle step (scheduled / fired / cancelled) in the UI
/// interaction audit recorder. Actor `system` plus the `auto_switcher.*`
/// method name identifies the auto-switcher as the actor.
fn audit_resume_lifecycle(stage: &str, instance_id: &str, detail: &str) {
    let clean_method_label = match stage {
        "cancelled" => "Cancel Prompt Restore Settling Timer",
        "scheduled" => "Schedule Prompt Restore Timer (10s)",
        "fired" => "Execute Prompt Restore Rehydration",
        "rearmed" => "Re-Arm Persisted Prompt Restore",
        "force_dispatched" => "Force Dispatch Prompt Restore",
        _ => "Post-Switch Resume Lifecycle",
    };
    let step_summary = match stage {
        "cancelled" => format!(
            "AutoSwitcher cancelled 10s prompt restore settling timer on instance '{instance_id}': {detail}"
        ),
        "scheduled" => format!(
            "AutoSwitcher scheduled 10s settling window and prompt restore on instance '{instance_id}': {detail}"
        ),
        "fired" => format!(
            "AutoSwitcher executed prompt restore rehydration on instance '{instance_id}': {detail}"
        ),
        "rearmed" => format!(
            "AutoSwitcher re-armed persisted prompt restore on instance '{instance_id}' after restart: {detail}"
        ),
        "force_dispatched" => format!(
            "AutoSwitcher force-dispatched immediate prompt restore on instance '{instance_id}': {detail}"
        ),
        _ => format!(
            "AutoSwitcher post-switch lifecycle stage '{stage}' on '{instance_id}': {detail}"
        ),
    };
    let element_hierarchy = vec![
        format!("Instance: {instance_id}"),
        "AutoSwitcher Engine".to_string(),
        "Prompt Continuity Hub".to_string(),
        match stage {
            "cancelled" => "Settling Timer Cancellation Guard",
            "scheduled" => "10-Second Settling Window Timer",
            "fired" => "FIFO Prompt Restore Rehydration",
            "rearmed" => "Persisted Restore State Recovery",
            "force_dispatched" => "Manual Force Dispatch Sweep",
            _ => "Lifecycle State Transition",
        }
        .to_string(),
    ];
    let action_desc = match stage {
        "cancelled" => format!(
            "Antigravity Manager cancelled the scheduled 10-second prompt restore timer for instance '{instance_id}' because {detail}. Active and queued prompts remain safely preserved in repo_prompts.db and active_prompts FIFO queue."
        ),
        "scheduled" => format!(
            "Antigravity Manager scheduled a 10-second settling window for instance '{instance_id}' ({detail}) to allow IDE process and Chrome DevTools Protocol (CDP) session stabilization before rehydrating prompts."
        ),
        "fired" => format!(
            "Antigravity Manager executed prompt restore rehydration on instance '{instance_id}' ({detail}). Queued turns were submitted sequentially in strict FIFO order."
        ),
        "rearmed" => format!(
            "Antigravity Manager discovered an uncompleted prompt restore task for instance '{instance_id}' in post_switch_resume_state.json during startup and re-armed the timer ({detail})."
        ),
        "force_dispatched" => format!(
            "Antigravity Manager bypassed the remaining settling delay and immediately dispatched backed-up prompts to instance '{instance_id}' ({detail})."
        ),
        _ => format!(
            "AutoSwitcher lifecycle event '{stage}' on instance '{instance_id}': {detail}"
        ),
    };

    let input = RecordInteractionInput {
        actor: InteractionActor::System,
        method_name: format!("auto_switcher.post_switch_resume.{stage}"),
        route: format!("/instances?id={instance_id}"),
        component: Some("PostSwitchResume".to_string()),
        element_id: Some(format!("resume-{stage}-{instance_id}")),
        trigger: InteractionTrigger::Programmatic,
        params: Some(serde_json::json!({
            "instance_id": instance_id,
            "stage": stage,
            "reason": detail,
            "stepSummary": step_summary,
            "cleanMethodLabel": clean_method_label,
            "elementHierarchy": element_hierarchy,
            "container": format!("Instance Card: {instance_id}"),
            "textSnippet": format!("Stage: {stage} • {detail}"),
            "tag": "system-daemon",
            "action_description": action_desc,
            "executive_briefing": {
                "headline": format!("Prompt Restore Timer {}: {}", stage.to_uppercase(), instance_id),
                "operation": clean_method_label,
                "rationale": detail,
                "instance_id": instance_id,
                "safeguard": "Strict FIFO Queue Monotonicity (ORDER BY created_at ASC, id ASC)",
            }
        })),
        result: "ok".to_string(),
        message: Some(detail.to_string()),
    };
    // Justification: audit telemetry is best-effort; when the recorder is stopped this is a logged no-op
    crate::error::record_ignored(
        record_interaction(input),
        "audit post-switch resume lifecycle",
    );
}

fn remove_timer_entry(instance_id: &str) {
    let mut timers = POST_SWITCH_RESUME_TASKS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    timers.remove(instance_id);
}
