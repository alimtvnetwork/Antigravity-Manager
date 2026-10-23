use crate::error::AppError;
use crate::models::instance::{InstanceConfig, InstanceStatus};
use crate::modules::instance;
use crate::modules::instance::post_switch_resume::{
    cancel_post_switch_resume as cancel_resume_internal, force_dispatch_resume,
    get_instance_pending_resume as get_inst_resume_internal, get_pending_resumes,
    ScheduledResumeEntry,
};

#[tauri::command]
pub fn list_instances() -> Result<Vec<InstanceStatus>, String> {
    instance::list_instances()
}

#[tauri::command]
pub fn create_instance(
    name: String,
    bound_account_id: Option<String>,
    from_instance_id: Option<String>,
) -> Result<InstanceConfig, String> {
    let mut cfg = if let Some(ref source) = from_instance_id {
        let resolved_src = instance::resolve_instance_id(source).unwrap_or_else(|_| source.clone());
        instance::copy_instance(&resolved_src, name.clone(), Some("full"))?
    } else {
        instance::create_instance_with_account(name, bound_account_id.as_deref())?
    };

    if let Some(ref acc_id) = bound_account_id {
        if from_instance_id.is_some() {
            let switch_res = tauri::async_runtime::block_on(instance::switch_account_to_instance(
                acc_id,
                Some(&cfg.id),
            ));
            if let Err(ref e) = switch_res {
                tracing::warn!(
                    "[Instance] Failed to switch account '{}' to new instance '{}': {}",
                    acc_id,
                    cfg.id,
                    e
                );
            }
            crate::error::record_ignored(switch_res, "create_instance: switch_account_to_instance"); // Justification: best-effort account binding after cloning; instance was created successfully
            if let Ok(acc) = crate::modules::account::load_account(acc_id) {
                cfg.bound_account_id = Some(acc.id);
                cfg.bound_email = Some(acc.email);
            }
        }
    }

    Ok(cfg)
}

#[tauri::command]
pub fn stop_instance(instance_id: String, target_pid: Option<u32>) -> Result<(), String> {
    // Explicit GUI action: allow stopping instance even if AGM was spawned from an IDE terminal
    std::env::set_var("AGM_FORCE_CALLER_OVERRIDE", "1");

    let mut audit = crate::modules::task_history_db::AuditTask::start(
        crate::modules::audit_action::AuditAction::StopInstance,
        &format!("Stop instance {}", instance_id),
        Some(&instance_id),
    );

    if let Some(pid) = target_pid {
        if pid > 0 {
            if let Ok(reg) = instance::load_registry() {
                if let Some(inst) = reg.instances.iter().find(|i| i.id == instance_id) {
                    // Justification: best effort recording of target pid before stopping
                    crate::error::record_ignored(
                        instance::record_instance_pid(&instance_id, pid, &inst.data_dir),
                        "record target pid before stop",
                    );
                }
            }
            #[cfg(target_os = "windows")]
            {
                use std::os::windows::process::CommandExt;
                if instance::is_pid_alive_targeted(pid) {
                    crate::error::record_ignored(
                        std::process::Command::new("taskkill")
                            .args(["/F", "/T", "/PID", &pid.to_string()])
                            .creation_flags(0x08000000)
                            .output(),
                        "taskkill target pid directly",
                    );
                }
            }
        }
    }
    let res = instance::stop_instance(&instance_id);
    if let Err(ref e) = res {
        audit.fail(e);
        return Err(e.clone());
    }
    audit.succeed(&format!("Instance {} stopped successfully", instance_id));

    if let Some(handle) = crate::modules::log_bridge::get_app_handle() {
        use tauri::Emitter;
        // Justification: best-effort UI event emission; UI state is re-derived on poll
        crate::error::record_ignored(
            handle.emit("instances://refreshed", ()),
            "emit instances://refreshed",
        );
        // Justification: best-effort UI event emission; UI state is re-derived on poll
        crate::error::record_ignored(
            handle.emit("accounts://refreshed", ()),
            "emit accounts://refreshed",
        );
    }
    Ok(())
}

#[tauri::command]
pub async fn restart_instance(instance_id: String) -> Result<InstanceStatus, String> {
    let resolved_id = instance::resolve_instance_id(&instance_id).unwrap_or(instance_id);

    // Refresh the bound/selected account's token BEFORE the sync restart runs,
    // so the reopened IDE comes up with the same account on fresh credentials.
    // (The sync `instance::restart_instance` cannot block_on inside the runtime.)
    if let Ok(registry) = instance::load_registry() {
        if let Some(cfg) = registry.instances.iter().find(|i| i.id == resolved_id) {
            if let Some(ref account_id) = cfg.bound_account_id {
                if let Ok(mut account) = crate::modules::account::load_account(account_id) {
                    match crate::modules::oauth::ensure_fresh_token(
                        &account.token,
                        Some(&account.id),
                    )
                    .await
                    {
                        Ok(tok) => {
                            if tok.access_token != account.token.access_token {
                                account.token = tok;
                                // Justification: token persistence is write-through; re-persisted on the next account touch
                                crate::error::record_ignored(
                                    crate::modules::account::save_account(&account),
                                    "save refreshed account token",
                                );
                            }
                        }
                        Err(e) => {
                            crate::modules::logger::log_warn(&format!(
                                "[Instance] Token refresh failed for bound account '{}' during restart: {}. Proceeding with stored token.",
                                account.email, e
                            ));
                        }
                    }
                }
            }
        }
    }

    instance::restart_instance(&resolved_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn fast_forward_instance(instance_id: String) -> Result<String, String> {
    let res =
        crate::modules::auto_switcher::trigger_manual_rotation_for_instance(Some(&instance_id))
            .await;
    // Justification: UI audit recording is observability; never block fast-forward on it.
    let ff_res_clone = res.clone();
    crate::error::record_ignored(
        crate::modules::audit::record_interaction(crate::modules::audit::RecordInteractionInput {
            actor: crate::modules::audit::InteractionActor::User,
            method_name: "fast_forward_instance".to_string(),
            route: "/instances".to_string(),
            component: Some("InstanceCardActions".to_string()),
            element_id: Some(format!("fast-forward-{instance_id}")),
            trigger: crate::modules::audit::InteractionTrigger::Click,
            params: Some(serde_json::json!({
                "instance_id": instance_id,
                "stepSummary": format!(
                    "User executed Fast-Forward profile rotation on instance '{instance_id}'"
                ),
                "cleanMethodLabel": "Fast-Forward Profile Rotation",
                "elementHierarchy": [
                    format!("Instance: {instance_id}"),
                    "Instances Grid".to_string(),
                    "Instance Card Actions".to_string(),
                    "Fast-Forward Rotation Trigger".to_string(),
                ],
                "container": format!("Instance Card: {instance_id}"),
                "textSnippet": "Fast Forward Shortcut",
                "tag": "action-button",
                "action_description": format!(
                    "Initiated manual 4-tier fast-forward account evaluation for instance '{}'. \
                     Candidate selection enforces 15% weekly quota floor, 100% 4-hour headroom, \
                     zero cross-instance fleet usage, and earliest reset timestamp prioritization.",
                    instance_id
                ),
                "executive_briefing": {
                    "headline": format!("Fast-Forward Profile Rotation: {}", instance_id),
                    "operation": "Manual Fast-Forward Account Switch",
                    "rationale": "User triggered instant profile fast-forward shortcut",
                    "instance_id": instance_id,
                    "safeguard": "4-Tier Candidate Resolution Ladder & Quota Headroom Invariants",
                }
            })),
            result: if res.is_ok() {
                "ok".to_string()
            } else {
                "error".to_string()
            },
            message: Some(match &ff_res_clone {
                Ok(msg) => msg.clone(),
                Err(err) => err.clone(),
            }),
        }),
        "audit fast_forward_instance",
    );

    let status = if res.is_ok() {
        crate::modules::task_history_db::AuditTaskStatus::Succeeded
    } else {
        crate::modules::task_history_db::AuditTaskStatus::Failed
    };
    let detail = match &res {
        Ok(msg) => msg.clone(),
        Err(err) => format!("Fast-forward rotation failed: {err}"),
    };
    crate::modules::task_history_db::record_instance_lifecycle_event_with_status(
        crate::modules::audit_action::AuditAction::SwitchAccount,
        Some(&instance_id),
        &format!("Fast-forward rotation for {}", instance_id),
        status,
        &detail,
    );

    res
}

#[tauri::command]
pub fn copy_instance(
    source_id: String,
    target_name: String,
    clone_mode: Option<String>,
    copy_projects: Option<bool>,
) -> Result<InstanceConfig, String> {
    let resolved_src = instance::resolve_instance_id(&source_id).unwrap_or(source_id);
    let should_copy_projs = copy_projects.unwrap_or(true);
    instance::copy_instance_with_options(
        &resolved_src,
        target_name,
        clone_mode.as_deref(),
        should_copy_projs,
    )
}

#[tauri::command]
pub fn export_instances_json() -> Result<String, String> {
    instance::export_instances_json()
}

#[tauri::command]
pub fn import_instances_json(json_content: String) -> Result<Vec<InstanceConfig>, String> {
    instance::import_instances_json(&json_content)
}

#[tauri::command]
pub fn rename_instance(instance_id: String, new_name: String) -> Result<InstanceConfig, String> {
    instance::rename_instance(&instance_id, new_name)
}

#[tauri::command]
pub fn delete_instance(instance_id: String) -> Result<(), String> {
    instance::delete_instance(&instance_id)
}

#[tauri::command]
pub fn wipe_instance_session(instance_id: String) -> Result<(), String> {
    instance::wipe_instance_session(&instance_id)
}

#[tauri::command]
pub fn launch_instance(instance_id: String) -> Result<(), crate::error::AppError> {
    let mut audit = crate::modules::task_history_db::AuditTask::start(
        crate::modules::audit_action::AuditAction::StartInstance,
        &format!("Launch instance {}", instance_id),
        Some(&instance_id),
    );
    let res = instance::launch_instance(&instance_id);
    if res.is_ok() {
        audit.succeed(&format!("Instance {} launched", instance_id));
    } else if let Err(ref e) = res {
        audit.fail(&e.to_string());
    }
    res
}

#[tauri::command]
pub fn launch_all_instances() -> Result<Vec<String>, crate::error::AppError> {
    let mut audit = crate::modules::task_history_db::AuditTask::start(
        crate::modules::audit_action::AuditAction::StartInstance,
        "Launch all stopped instances",
        None,
    );

    let instances = instance::list_instances().map_err(crate::error::AppError::Process)?;
    let mut launched = Vec::new();
    let mut errors = Vec::new();

    for inst in instances {
        if !inst.is_running {
            match instance::launch_instance(&inst.config.id) {
                Ok(_) => {
                    launched.push(inst.config.id.clone());
                }
                Err(e) => {
                    tracing::warn!(
                        "[Instance] Failed to launch instance '{}' during launch_all: {}",
                        inst.config.id,
                        e
                    );
                    errors.push(format!("{}: {}", inst.config.id, e));
                }
            }
        }
    }

    if let Some(handle) = crate::modules::log_bridge::get_app_handle() {
        use tauri::Emitter;
        // Justification: best-effort UI event emission; UI state is re-derived on poll
        crate::error::record_ignored(
            handle.emit("instances://refreshed", ()),
            "emit instances://refreshed",
        );
    }

    if errors.is_empty() {
        audit.succeed(&format!("Launched {} instance(s)", launched.len()));
    } else if launched.is_empty() {
        audit.fail(&format!(
            "Failed to launch stopped instance(s): {}",
            errors.join("; ")
        ));
    } else {
        audit.succeed(&format!(
            "Launched {} instance(s) with {} failure(s): {}",
            launched.len(),
            errors.len(),
            errors.join("; ")
        ));
    }

    Ok(launched)
}

#[tauri::command]
pub fn focus_or_launch_instance(
    instance_id: String,
    workspace_path: Option<String>,
) -> Result<bool, crate::error::AppError> {
    instance::focus_or_launch_instance_with_workspace(&instance_id, workspace_path.as_deref())
}

#[tauri::command]
pub fn focus_instance_workspace(
    instance_id: String,
    repo_path: String,
    repo_name: String,
) -> Result<bool, crate::error::AppError> {
    instance::focus_or_launch_workspace(&instance_id, &repo_path, &repo_name)
}

#[tauri::command]
pub fn clone_instance_executable(instance_id: String) -> Result<String, crate::error::AppError> {
    instance::clone_instance_executable(&instance_id)
}

#[tauri::command]
pub fn set_instance_executable(
    instance_id: String,
    executable_path: Option<String>,
) -> Result<(), crate::error::AppError> {
    instance::set_instance_executable(&instance_id, executable_path)
}

#[tauri::command]
pub fn close_instance(instance_id: String) -> Result<(), String> {
    instance::close_instance(&instance_id)?;
    if let Some(handle) = crate::modules::log_bridge::get_app_handle() {
        use tauri::Emitter;
        // Justification: best-effort UI event emission; UI state is re-derived on poll
        crate::error::record_ignored(
            handle.emit("instances://refreshed", ()),
            "emit instances://refreshed",
        );
        // Justification: best-effort UI event emission; UI state is re-derived on poll
        crate::error::record_ignored(
            handle.emit("accounts://refreshed", ()),
            "emit accounts://refreshed",
        );
    }
    Ok(())
}

#[tauri::command]
pub fn get_active_instance() -> Result<String, String> {
    instance::get_active_instance_id()
}

#[tauri::command]
pub fn set_active_instance(instance_id: String) -> Result<(), String> {
    instance::set_active_instance_id(&instance_id)
}

#[tauri::command]
pub fn set_default_instance(instance_id: String) -> Result<(), String> {
    instance::set_default_instance(&instance_id)
}

#[tauri::command]
pub async fn switch_account_to_instance(
    account_id: String,
    instance_id: Option<String>,
) -> Result<(), crate::error::AppError> {
    // Explicit GUI action: allow switching even if AGM was spawned from an IDE terminal
    std::env::set_var("AGM_FORCE_CALLER_OVERRIDE", "1");
    let res = instance::switch_account_to_instance(&account_id, instance_id.as_deref())
        .await
        .map_err(crate::error::AppError::Process);
    if res.is_ok() {
        if let Some(handle) = crate::modules::log_bridge::get_app_handle() {
            use tauri::Emitter;
            // Justification: best-effort UI event emission; UI state is re-derived on poll
            crate::error::record_ignored(
                handle.emit("instances://refreshed", ()),
                "emit instances://refreshed",
            );
            // Justification: best-effort UI event emission; UI state is re-derived on poll
            crate::error::record_ignored(
                handle.emit("accounts://refreshed", ()),
                "emit accounts://refreshed",
            );
        }
    }
    // Justification: UI audit recording is observability; never block account switch on it.
    let target_label = instance_id
        .as_deref()
        .unwrap_or("auto-detected active instance");
    crate::error::record_ignored(
        crate::modules::audit::record_interaction(crate::modules::audit::RecordInteractionInput {
            actor: crate::modules::audit::InteractionActor::User,
            method_name: "switch_account_to_instance".to_string(),
            route: "/accounts".to_string(),
            component: Some("AccountRow".to_string()),
            element_id: Some(format!("switch-account-{account_id}")),
            trigger: crate::modules::audit::InteractionTrigger::Click,
            params: Some(serde_json::json!({
                "account_id": account_id,
                "instance_id": target_label,
                "stepSummary": format!(
                    "User switched account '{account_id}' to instance '{target_label}'"
                ),
                "cleanMethodLabel": "Switch Profile to Instance",
                "elementHierarchy": [
                    "/accounts".to_string(),
                    "Accounts Table".to_string(),
                    format!("Account Row: {account_id}"),
                    format!("Target Instance: {target_label}"),
                ],
                "container": format!("Account Row: {account_id}"),
                "textSnippet": format!("Switch to {target_label}"),
                "tag": "action-button",
                "action_description": format!(
                    "Explicit user profile switch binding account '{}' to instance '{}'. \
                     Running prompts and active state snapshotted, post-switch 10s settling window armed.",
                    account_id, target_label
                ),
                "executive_briefing": {
                    "headline": format!("Account Switch: {} → {}", account_id, target_label),
                    "operation": "Manual Account Profile Switch",
                    "rationale": "Direct user selection from accounts management table",
                    "instance_id": target_label,
                    "safeguard": "Prompt Continuity Preservation & Settling Delay",
                }
            })),
            result: if res.is_ok() {
                "ok".to_string()
            } else {
                "error".to_string()
            },
            message: res.as_ref().err().map(|e| e.to_string()),
        }),
        "audit switch_account_to_instance",
    );
    res
}

#[tauri::command]
pub fn get_auto_switcher_config() -> Result<crate::models::config::AutoProfileSwitcherConfig, String>
{
    let app_config = crate::modules::config::load_app_config()?;
    Ok(app_config.auto_profile_switcher)
}

#[tauri::command]
pub fn toggle_auto_switcher() -> Result<bool, String> {
    let mut app_config = crate::modules::config::load_app_config()?;
    let new_val = !app_config.auto_profile_switcher.is_enabled;
    app_config.auto_profile_switcher.is_enabled = new_val;
    crate::modules::config::save_app_config(&app_config)?;
    if new_val {
        tokio::spawn(async move {
            // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
            crate::error::record_ignored(
                crate::modules::auto_switcher::check_and_rotate_if_needed().await,
                "check_and_rotate_if_needed",
            );
        });
    }
    Ok(new_val)
}

#[tauri::command]
pub fn get_auto_switcher_status(
) -> Result<crate::modules::auto_switcher::AutoSwitcherStatus, String> {
    Ok(crate::modules::auto_switcher::get_status())
}

#[tauri::command]
pub async fn get_auto_switcher_daemon_status(
) -> Result<crate::modules::auto_switcher::AutoSwitcherDaemonStatus, String> {
    Ok(crate::modules::auto_switcher::get_daemon_status())
}

#[tauri::command]
pub fn update_auto_switcher_config(
    config: crate::models::config::AutoProfileSwitcherConfig,
) -> Result<(), String> {
    let is_enabled = config.is_enabled;
    let mut app_config = crate::modules::config::load_app_config()?;
    app_config.auto_profile_switcher = config;
    crate::modules::config::save_app_config(&app_config)?;
    if is_enabled {
        tauri::async_runtime::spawn(async move {
            // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
            crate::error::record_ignored(
                crate::modules::auto_switcher::check_and_rotate_if_needed().await,
                "check_and_rotate_if_needed",
            );
        });
    }
    Ok(())
}

#[tauri::command]
pub async fn trigger_manual_profile_rotation(
    instance_id: Option<String>,
) -> Result<String, String> {
    crate::modules::auto_switcher::trigger_manual_rotation_for_instance(instance_id.as_deref())
        .await
}

#[tauri::command]
pub fn list_running_projects() -> Result<Vec<crate::modules::repo_db::RunningProject>, String> {
    crate::modules::repo_db::list_running_projects()
}

#[tauri::command]
pub fn list_backed_up_prompts() -> Result<Vec<crate::modules::repo_db::ActivePrompt>, String> {
    crate::modules::repo_db::list_backed_up_prompts()
}

#[tauri::command]
pub fn clean_and_restart_workspace() -> Result<String, String> {
    crate::modules::process::clean_and_restart_workspace(None)
}

#[tauri::command]
pub fn resume_recent_project_prompts(
    instance_id: Option<String>,
    max_age_seconds: Option<i64>,
) -> Result<crate::modules::repo_db::AutoResumeResult, String> {
    let inst_id = instance_id.unwrap_or_else(|| {
        crate::modules::instance::get_active_instance_id().unwrap_or_else(|_| "default".to_string())
    });
    let max_age = max_age_seconds.unwrap_or(3600);
    crate::modules::repo_db::auto_resume_recent_prompts(&inst_id, max_age)
}

fn panic_payload_text(payload: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(text) = payload.downcast_ref::<String>() {
        return text.clone();
    }
    if let Some(text) = payload.downcast_ref::<&str>() {
        return (*text).to_string();
    }
    "unknown panic payload".to_string()
}

fn prompt_task_error(label: &str, payload: Box<dyn std::any::Any + Send>) -> String {
    let detail = panic_payload_text(&payload);
    let trace = std::backtrace::Backtrace::force_capture();
    let message = format!("{label} panicked: {detail}\n{trace}");
    tracing::error!(target: "instance", "{message}");
    message
}

fn run_caught_prompt_task<F>(
    label: &str,
    task: F,
) -> Result<crate::modules::repo_db::ActivePrompt, String>
where
    F: FnOnce() -> Result<crate::modules::repo_db::ActivePrompt, String> + Send,
{
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(task));
    match caught {
        Ok(result) => result,
        Err(payload) => Err(prompt_task_error(label, payload)),
    }
}

#[tauri::command]
pub async fn send_prompt_now(
    instance_id: String,
    repo_path: String,
    prompt_content: String,
    conversation_id: Option<String>,
    media_paths: Option<Vec<String>>,
) -> Result<crate::modules::repo_db::ActivePrompt, String> {
    let resolved_id = instance::resolve_instance_id(&instance_id).unwrap_or(instance_id);
    let audit_inst = resolved_id.clone();
    let audit_repo = repo_path.clone();
    let audit_conv = conversation_id.clone();
    let media_paths_clone = media_paths.clone();
    let media_paths_audit = media_paths.clone();
    let res = tokio::task::spawn_blocking(move || {
        run_caught_prompt_task("send_prompt_now", move || {
            let mut prompt = crate::modules::repo_db::send_prompt_now_for_instance(
                &resolved_id,
                &repo_path,
                &prompt_content,
                conversation_id.as_deref(),
            )?;
            if let Some(ref paths) = media_paths_clone {
                if !paths.is_empty() {
                    prompt.image_payload = Some(paths.join(";"));
                    // Justification: updating image_payload in SQLite cache after successful dispatch is best-effort
                    crate::error::record_ignored(
                        crate::modules::repo_db::save_or_requeue_prompt(&prompt),
                        "update prompt image payload in database",
                    );
                }
            }
            Ok(prompt)
        })
    })
    .await
    .map_err(|e| format!("send_prompt_now task failed: {e}"))?;
    // Justification: UI audit recording is observability; never block prompt dispatch on it.
    let send_inst_for_audit = audit_inst.clone();
    let send_repo_for_audit = audit_repo.clone();
    let media_count = media_paths_audit.as_ref().map(|m| m.len()).unwrap_or(0);
    crate::error::record_ignored(
        crate::modules::audit::record_interaction(crate::modules::audit::RecordInteractionInput {
            actor: crate::modules::audit::InteractionActor::User,
            method_name: "send_prompt_now".to_string(),
            route: "/instances".to_string(),
            component: Some("PromptTreeView".to_string()),
            element_id: Some("send-prompt-btn".to_string()),
            trigger: crate::modules::audit::InteractionTrigger::Click,
            params: Some(serde_json::json!({
                "instance_id": send_inst_for_audit,
                "repo_path": send_repo_for_audit,
                "conversation_id": audit_conv,
                "prompt_id": res.as_ref().ok().map(|p| p.id.clone()),
                "media_count": media_count,
                "stepSummary": format!(
                    "User dispatched immediate prompt to instance '{send_inst_for_audit}' ({send_repo_for_audit})"
                ),
                "cleanMethodLabel": "Send Prompt Now (Immediate Dispatch)",
                "elementHierarchy": [
                    "/instances".to_string(),
                    format!("Instance: {send_inst_for_audit}"),
                    "Prompt Tree View".to_string(),
                    "Immediate Dispatch Action".to_string(),
                ],
                "container": format!("Instance Prompt Tree: {send_inst_for_audit}"),
                "textSnippet": "Send Now",
                "tag": "action-button",
                "action_description": format!(
                    "Direct interactive dispatch bypassing FIFO backlog queue for immediate \
                     execution in instance '{}' (Repository: {}, Attached Media: {}).",
                    send_inst_for_audit, send_repo_for_audit, media_count
                ),
                "executive_briefing": {
                    "headline": format!("Prompt Dispatched: {}", send_inst_for_audit),
                    "operation": "Immediate Prompt Execution",
                    "rationale": "Interactive user 'Send Prompt Now' action",
                    "instance_id": send_inst_for_audit,
                    "safeguard": "Active CDP Channel Injection",
                }
            })),
            result: if res.is_ok() {
                "ok".to_string()
            } else {
                "error".to_string()
            },
            message: res.as_ref().err().cloned(),
        }),
        "audit send_prompt_now",
    );
    res
}

#[tauri::command]
pub async fn enqueue_prompt(
    instance_id: String,
    repo_path: String,
    prompt_content: String,
    conversation_id: Option<String>,
    _project_id: Option<String>,
    media_paths: Option<Vec<String>>,
) -> Result<crate::modules::repo_db::ActivePrompt, String> {
    let resolved_id = instance::resolve_instance_id(&instance_id).unwrap_or(instance_id);
    let audit_inst = resolved_id.clone();
    let audit_repo = repo_path.clone();
    let audit_conv = conversation_id.clone();
    let media_paths_clone = media_paths.clone();
    let media_paths_audit = media_paths.clone();
    let res = tokio::task::spawn_blocking(move || {
        run_caught_prompt_task("enqueue_prompt", move || {
            let mut prompt = crate::modules::repo_db::enqueue_prompt_for_instance(
                &resolved_id,
                &repo_path,
                &prompt_content,
                conversation_id.as_deref(),
            )?;
            if let Some(ref paths) = media_paths_clone {
                if !paths.is_empty() {
                    prompt.image_payload = Some(paths.join(";"));
                    // Justification: updating image_payload in SQLite cache after successful enqueue is best-effort
                    crate::error::record_ignored(
                        crate::modules::repo_db::save_or_requeue_prompt(&prompt),
                        "update queued prompt image payload in database",
                    );
                }
            }
            Ok(prompt)
        })
    })
    .await
    .map_err(|e| format!("enqueue_prompt task failed: {e}"))?;
    // Justification: UI audit recording is observability; never block prompt enqueue on it.
    let enq_inst_for_audit = audit_inst.clone();
    let enq_repo_for_audit = audit_repo.clone();
    let enq_media_count = media_paths_audit.as_ref().map(|m| m.len()).unwrap_or(0);
    crate::error::record_ignored(
        crate::modules::audit::record_interaction(crate::modules::audit::RecordInteractionInput {
            actor: crate::modules::audit::InteractionActor::User,
            method_name: "enqueue_prompt".to_string(),
            route: "/instances".to_string(),
            component: Some("PromptTreeView".to_string()),
            element_id: Some("enqueue-prompt-btn".to_string()),
            trigger: crate::modules::audit::InteractionTrigger::Click,
            params: Some(serde_json::json!({
                "instance_id": enq_inst_for_audit,
                "repo_path": enq_repo_for_audit,
                "conversation_id": audit_conv,
                "prompt_id": res.as_ref().ok().map(|p| p.id.clone()),
                "media_count": enq_media_count,
                "stepSummary": format!(
                    "User enqueued prompt for sequential FIFO processing on instance '{enq_inst_for_audit}' ({enq_repo_for_audit})"
                ),
                "cleanMethodLabel": "Enqueue Prompt (FIFO Queue)",
                "elementHierarchy": [
                    "/instances".to_string(),
                    format!("Instance: {enq_inst_for_audit}"),
                    "Prompt Tree View".to_string(),
                    "FIFO Enqueue Action".to_string(),
                ],
                "container": format!("Instance Prompt Tree: {enq_inst_for_audit}"),
                "textSnippet": "Enqueue for Later",
                "tag": "action-button",
                "action_description": format!(
                    "Appended prompt turn to persistent SQLite FIFO queue on instance '{}' \
                     (Repository: {}, Attached Media: {}). Execution will occur sequentially via \
                     background queue dispatcher.",
                    enq_inst_for_audit, enq_repo_for_audit, enq_media_count
                ),
                "executive_briefing": {
                    "headline": format!("Prompt Enqueued: {}", enq_inst_for_audit),
                    "operation": "FIFO Background Queue Insertion",
                    "rationale": "Interactive user 'Enqueue Prompt' action",
                    "instance_id": enq_inst_for_audit,
                    "safeguard": "Strict FIFO Monotonicity (created_at ASC, id ASC) in active_prompts",
                }
            })),
            result: if res.is_ok() {
                "ok".to_string()
            } else {
                "error".to_string()
            },
            message: res.as_ref().err().cloned(),
        }),
        "audit enqueue_prompt",
    );
    res
}

#[tauri::command]
pub async fn cancel_running_prompt(
    instance_id: String,
    repo_path: Option<String>,
) -> Result<bool, String> {
    let resolved_id = instance::resolve_instance_id(&instance_id).unwrap_or(instance_id);
    tokio::task::spawn_blocking(move || {
        crate::modules::repo_db::cancel_running_prompt_for_instance(
            &resolved_id,
            repo_path.as_deref(),
        )
    })
    .await
    .map_err(|e| format!("cancel_running_prompt task failed: {e}"))?
}

#[tauri::command]
pub fn get_running_instances_process_count() -> Result<usize, String> {
    Ok(crate::modules::instance::scan_and_cache_all_running_instances())
}

#[tauri::command]
pub fn restore_prompts_backup(backup_json: String) -> Result<usize, String> {
    crate::modules::repo_db::restore_prompts_from_backup_json(&backup_json)
}

#[tauri::command]
pub fn dispatch_backed_up_prompts(instance_id: Option<String>) -> Result<usize, String> {
    let id = instance_id.unwrap_or_else(|| "default".to_string());
    crate::modules::repo_db::dispatch_running_prompts(&id)
}

#[tauri::command]
pub fn assign_project_to_instance(
    instance_id: String,
    repo_paths: Vec<String>,
) -> Result<Vec<String>, String> {
    let mut assigned = Vec::new();
    for path in &repo_paths {
        let res = instance::assign_project_to_instance(&instance_id, path)?;
        assigned.push(res);
    }
    Ok(assigned)
}

#[tauri::command]
pub fn get_instance_workspace_folders(instance_id: String) -> Result<Vec<String>, String> {
    let resolved_id = instance::resolve_instance_id(&instance_id)?;
    let registry = instance::load_registry()?;
    let data_dir = registry
        .instances
        .iter()
        .find(|i| i.id == resolved_id)
        .map(|i| i.data_dir.clone())
        .unwrap_or_default();
    Ok(instance::get_instance_workspace_folders(
        &resolved_id,
        &data_dir,
    ))
}

#[tauri::command]
pub fn copy_instance_projects(from_id: String, to_id: String) -> Result<usize, String> {
    instance::copy_instance_projects(&from_id, &to_id)
}

#[tauri::command]
pub fn copy_instance_settings(from_id: String, to_id: String) -> Result<(), String> {
    instance::copy_instance_settings(&from_id, &to_id)
}

#[tauri::command]
pub fn enforce_default_settings(target_instance: Option<String>) -> Result<usize, String> {
    instance::enforce_default_settings(target_instance.as_deref())
}

#[tauri::command]
pub fn set_instance_turbo_mode(
    target_instance: Option<String>,
    enabled: bool,
) -> Result<usize, String> {
    instance::set_instance_turbo_mode(target_instance.as_deref(), enabled)
}

#[tauri::command]
pub fn set_instance_plan_review(
    target_instance: Option<String>,
    always_proceed: bool,
) -> Result<usize, String> {
    instance::set_instance_plan_review(target_instance.as_deref(), always_proceed)
}

#[tauri::command]
pub fn export_instance_settings(instance_id: String) -> Result<String, String> {
    instance::export_instance_settings(&instance_id)
}

#[tauri::command]
pub fn import_instance_settings(
    target_instance: Option<String>,
    json_str: String,
) -> Result<usize, String> {
    instance::import_instance_settings(target_instance.as_deref(), &json_str)
}

#[tauri::command]
pub fn count_instances() -> Result<serde_json::Value, String> {
    instance::count_instances()
}

#[tauri::command]
pub async fn get_project_conversation_tree(
    instance_id: Option<String>,
    max_words: Option<usize>,
    only_running: Option<bool>,
    force: Option<bool>,
) -> Result<Vec<crate::modules::repo_db::AgmProjectTreeNode>, String> {
    tokio::task::spawn_blocking(move || {
        let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            crate::modules::repo_db::get_project_conversation_tree_cached(
                instance_id.as_deref(),
                max_words.unwrap_or(200),
                only_running.unwrap_or(false),
                force.unwrap_or(false),
            )
        }));
        match caught {
            Ok(nodes) => Ok(nodes),
            Err(payload) => Err(prompt_task_error("get_project_conversation_tree", payload)),
        }
    })
    .await
    .map_err(|e| format!("get_project_conversation_tree task failed: {e}"))?
}

#[tauri::command]
pub async fn reset_prompt_tree_cache(instance_id: Option<String>) -> Result<String, String> {
    let id_clone = instance_id.clone();
    tokio::task::spawn_blocking(move || {
        let count = crate::modules::repo_db::wipe_and_rescan_repo_db(id_clone.as_deref())?;
        tracing::info!(
            target: "prompt_tree",
            instance_id = id_clone.as_deref().unwrap_or("all"),
            count = count,
            "wiped database and regenerated prompt tree from disk transcripts"
        );
        Ok(format!(
            "Prompt tree wiped and regenerated ({} projects)",
            count
        ))
    })
    .await
    .map_err(|e| format!("wipe_and_rescan_repo_db task failed: {e}"))?
}

#[tauri::command]
pub async fn wipe_and_rescan_repo_db(instance_id: Option<String>) -> Result<usize, String> {
    let id_clone = instance_id.clone();
    tokio::task::spawn_blocking(move || {
        crate::modules::repo_db::wipe_and_rescan_repo_db(id_clone.as_deref())
    })
    .await
    .map_err(|e| format!("wipe_and_rescan_repo_db task failed: {e}"))?
}

#[tauri::command]
pub fn get_instance_switch_history(
    instance_id: String,
    limit: Option<u32>,
) -> Result<crate::modules::task_history_db::InstanceSwitchHistoryResponse, String> {
    let resolved_id = instance::resolve_instance_id(&instance_id).unwrap_or(instance_id);
    let l = limit.unwrap_or(50) as usize;
    crate::modules::task_history_db::get_instance_switch_history(&resolved_id, l)
}

#[tauri::command]
pub fn get_instance_audit_trail(
    instance_id: String,
    limit: Option<u32>,
) -> Result<Vec<crate::modules::task_history_db::TaskRecord>, String> {
    let resolved_id = instance::resolve_instance_id(&instance_id).unwrap_or(instance_id);
    let l = limit.unwrap_or(50) as usize;
    crate::modules::task_history_db::get_instance_audit_trail(&resolved_id, l)
}

#[tauri::command]
pub async fn sync_instance_pid_and_quota(instance_id: String) -> Result<InstanceStatus, String> {
    let resolved_id = instance::resolve_instance_id(&instance_id).unwrap_or(instance_id);
    instance::sync_instance_pid_and_quota_logic(&resolved_id).await
}

#[tauri::command]
pub async fn sync_all_instances_and_quotas() -> Result<Vec<InstanceStatus>, String> {
    instance::sync_all_instances_and_quotas_logic().await
}

#[tauri::command]
pub async fn get_pending_post_switch_resumes() -> Result<Vec<ScheduledResumeEntry>, AppError> {
    Ok(get_pending_resumes())
}

#[tauri::command]
pub async fn get_instance_pending_resume(
    instance_id: String,
) -> Result<Option<ScheduledResumeEntry>, AppError> {
    let resolved_id = instance::resolve_instance_id(&instance_id).unwrap_or(instance_id);
    Ok(get_inst_resume_internal(&resolved_id))
}

#[tauri::command]
pub async fn cancel_post_switch_resume(instance_id: String) -> Result<bool, AppError> {
    let resolved_id = instance::resolve_instance_id(&instance_id).unwrap_or(instance_id);
    Ok(cancel_resume_internal(
        &resolved_id,
        "User requested cancellation",
    ))
}

#[tauri::command]
pub async fn force_dispatch_pending_resume(
    app: tauri::AppHandle,
    instance_id: String,
) -> Result<(), AppError> {
    let resolved_id = instance::resolve_instance_id(&instance_id).unwrap_or(instance_id);
    force_dispatch_resume(&app, &resolved_id).await
}

#[tauri::command]
pub async fn rename_conversation(
    instance_id: Option<String>,
    conversation_id: String,
    new_title: String,
) -> Result<(), AppError> {
    let clean_title = new_title.trim();
    if clean_title.is_empty() {
        return Err(AppError::Config(
            "Conversation title cannot be empty".to_string(),
        ));
    }
    if conversation_id.trim().is_empty() {
        return Err(AppError::Config(
            "Conversation ID cannot be empty".to_string(),
        ));
    }

    let resolved_inst = instance_id
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "default".to_string());
    let title_owned = clean_title.to_string();

    tokio::task::spawn_blocking(move || {
        crate::modules::repo_db::sequences::rename_conversation_dual_store(
            &resolved_inst,
            &conversation_id,
            &title_owned,
        )
    })
    .await
    .map_err(|e| AppError::Process(format!("Task execution failed: {e}")))?
}
