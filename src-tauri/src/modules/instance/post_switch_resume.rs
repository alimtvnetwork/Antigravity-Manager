//! Post-switch prompt resume timer.
//!
//! After an auto-switch or account switch, the last running prompts are
//! resumed on the new account once the IDE's prompt channel is up. The delay
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

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

use once_cell::sync::Lazy;
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

/// Pending resume tasks keyed by instance id. At most one pending resume per
/// instance; scheduling a new one aborts the previous one.
static POST_SWITCH_RESUME_TASKS: Lazy<Mutex<HashMap<String, JoinHandle<()>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Schedule the post-switch resume for `instance_id` after the configured
/// delay. Any previously pending resume for the same instance is cancelled
/// first, so stacked switches can never double-restore.
pub fn schedule_post_switch_resume(instance_id: String) {
    cancel_post_switch_resume(&instance_id, "superseded by a newer schedule");

    let delay = post_switch_resume_delay_seconds();
    crate::modules::logger::log_info(&format!(
        "[PostSwitchResume] Scheduling prompt resume for instance '{instance_id}' in {delay}s"
    ));
    audit_resume_lifecycle(
        "scheduled",
        &instance_id,
        &format!("resume scheduled in {delay}s"),
    );

    let timer_instance_id = instance_id.clone();
    let handle = tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(delay)).await;
        let restored = run_resume_with_retry(&timer_instance_id).await;
        // Remove our own registry entry; a cancel that lost the race finds nothing.
        remove_timer_entry(&timer_instance_id);
        emit_prompts_restored(&timer_instance_id, restored);
        audit_resume_lifecycle(
            "fired",
            &timer_instance_id,
            &format!("resume fired; {restored} prompt(s) restored"),
        );
    });

    // Lock scope is sync-only: no .await while the guard is held. Aborting a
    // displaced handle covers the tiny race between cancel and insert.
    let mut timers = POST_SWITCH_RESUME_TASKS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(displaced) = timers.insert(instance_id, handle) {
        displaced.abort();
    }
}

/// Cancel a pending post-switch resume for `instance_id`, if any. No-op when
/// nothing is pending. Call this when the user (or another switch) acts first
/// so the later timer cannot re-inject prompts that were already handled.
pub fn cancel_post_switch_resume(instance_id: &str, reason: &str) {
    let removed = {
        let mut timers = POST_SWITCH_RESUME_TASKS
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        timers.remove(instance_id)
    };
    if let Some(handle) = removed {
        handle.abort();
        crate::modules::logger::log_info(&format!(
            "[PostSwitchResume] Cancelled pending resume for instance '{instance_id}': {reason}"
        ));
        audit_resume_lifecycle("cancelled", instance_id, reason);
    }
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
    let input = RecordInteractionInput {
        actor: InteractionActor::System,
        method_name: format!("auto_switcher.post_switch_resume.{stage}"),
        route: format!("instance://{instance_id}"),
        component: Some("PostSwitchResume".to_string()),
        element_id: None,
        trigger: InteractionTrigger::Programmatic,
        params: Some(serde_json::json!({
            "instance_id": instance_id,
            "stage": stage,
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
