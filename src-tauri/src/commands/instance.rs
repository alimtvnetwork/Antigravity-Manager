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
pub fn copy_instance(source_id: String, target_name: String) -> Result<InstanceConfig, String> {
    instance::copy_instance(&source_id, target_name)
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
pub fn launch_instance(instance_id: String) -> Result<(), String> {
    instance::launch_instance(&instance_id)
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

