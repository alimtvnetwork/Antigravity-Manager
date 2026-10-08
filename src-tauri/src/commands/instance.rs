use crate::models::instance::{InstanceConfig, InstanceStatus};
use crate::modules::instance;

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
            let _ = tauri::async_runtime::block_on(instance::switch_account_to_instance(
                acc_id,
                Some(&cfg.id),
            ));
            if let Ok(acc) = crate::modules::account::load_account(acc_id) {
                cfg.bound_account_id = Some(acc.id);
                cfg.bound_email = Some(acc.email);
            }
        }
    }

    Ok(cfg)
}

#[tauri::command]
pub fn stop_instance(instance_id: String) -> Result<(), String> {
    instance::stop_instance(&instance_id)
}

#[tauri::command]
pub async fn restart_instance(instance_id: String) -> Result<InstanceStatus, String> {
    let resolved_id = instance::resolve_instance_id(&instance_id).unwrap_or(instance_id);
    instance::restart_instance(&resolved_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn fast_forward_instance(instance_id: String) -> Result<String, String> {
    crate::modules::auto_switcher::trigger_manual_rotation_for_instance(Some(&instance_id)).await
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
    instance::launch_instance(&instance_id)
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
    instance::close_instance(&instance_id)
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
    instance::switch_account_to_instance(&account_id, instance_id.as_deref())
        .await
        .map_err(crate::error::AppError::Process)
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
            let _ = crate::modules::auto_switcher::check_and_rotate_if_needed().await;
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
            let _ = crate::modules::auto_switcher::check_and_rotate_if_needed().await;
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

#[tauri::command]
pub async fn send_prompt_now(
    instance_id: String,
    repo_path: String,
    prompt_content: String,
    conversation_id: Option<String>,
) -> Result<crate::modules::repo_db::ActivePrompt, String> {
    tokio::task::spawn_blocking(move || {
        crate::modules::repo_db::send_prompt_now_for_instance(
            &instance_id,
            &repo_path,
            &prompt_content,
            conversation_id.as_deref(),
        )
    })
    .await
    .map_err(|e| format!("Task execution failed: {}", e))?
}

#[tauri::command]
pub async fn enqueue_prompt(
    instance_id: String,
    repo_path: String,
    prompt_content: String,
    conversation_id: Option<String>,
) -> Result<crate::modules::repo_db::ActivePrompt, String> {
    tokio::task::spawn_blocking(move || {
        crate::modules::repo_db::enqueue_prompt_for_instance(
            &instance_id,
            &repo_path,
            &prompt_content,
            conversation_id.as_deref(),
        )
    })
    .await
    .map_err(|e| format!("Task execution failed: {}", e))?
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
pub fn get_project_conversation_tree(
    instance_id: Option<String>,
    max_words: Option<usize>,
    only_running: Option<bool>,
    force: Option<bool>,
) -> Result<Vec<crate::modules::repo_db::AgmProjectTreeNode>, String> {
    Ok(
        crate::modules::repo_db::get_project_conversation_tree_cached(
            instance_id.as_deref(),
            max_words.unwrap_or(200),
            only_running.unwrap_or(false),
            force.unwrap_or(false),
        ),
    )
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
pub async fn enqueue_prompt(
    instance_id: String,
    prompt_text: Option<String>,
    prompt_content: Option<String>,
    workspace_path: Option<String>,
    repo_path: Option<String>,
    conversation_id: Option<String>,
    project_id: Option<String>,
) -> crate::error::AppResult<serde_json::Value> {
    let resolved_id = instance::resolve_instance_id(&instance_id).unwrap_or(instance_id);
    let prompt = prompt_text.or(prompt_content).unwrap_or_default();
    let workspace = workspace_path.or(repo_path);

    let row_id = crate::modules::repo_db::enqueue_prompt_for_instance_full(
        &resolved_id,
        &prompt,
        workspace.as_deref(),
        conversation_id.as_deref(),
        project_id.as_deref(),
    )?;

    Ok(serde_json::json!({
        "success": true,
        "prompt_id": row_id,
        "instance_id": resolved_id,
        "status": "queued"
    }))
}
