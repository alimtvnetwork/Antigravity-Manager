use crate::models::instance::{InstanceConfig, InstanceStatus};
use crate::modules::instance;

#[tauri::command]
pub fn list_instances() -> Result<Vec<InstanceStatus>, String> {
    instance::list_instances()
}

#[tauri::command]
pub fn create_instance(name: String) -> Result<InstanceConfig, String> {
    instance::create_instance(name)
}

#[tauri::command]
pub fn copy_instance(
    source_id: String,
    target_name: String,
    clone_mode: Option<String>,
) -> Result<InstanceConfig, String> {
    instance::copy_instance(&source_id, target_name, clone_mode.as_deref())
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
pub async fn switch_account_to_instance(
    account_id: String,
    instance_id: Option<String>,
) -> Result<(), String> {
    instance::switch_account_to_instance(&account_id, instance_id.as_deref()).await
}

#[tauri::command]
pub fn get_auto_switcher_status(
) -> Result<crate::modules::auto_switcher::AutoSwitcherStatus, String> {
    Ok(crate::modules::auto_switcher::get_status())
}

#[tauri::command]
pub fn update_auto_switcher_config(
    config: crate::models::config::AutoProfileSwitcherConfig,
) -> Result<(), String> {
    let mut app_config = crate::modules::config::load_app_config()?;
    app_config.auto_profile_switcher = config;
    crate::modules::config::save_app_config(&app_config)
}

#[tauri::command]
pub async fn trigger_manual_profile_rotation() -> Result<String, String> {
    crate::modules::auto_switcher::trigger_manual_rotation().await
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
