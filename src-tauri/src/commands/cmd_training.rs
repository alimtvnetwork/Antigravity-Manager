use super::*;

#[tauri::command]
pub async fn get_training_api_status() -> Result<bool, String> {
    Ok(crate::modules::training_api::is_training_api_enabled())
}

/// Set the training REST API enabled status
#[tauri::command]
pub async fn set_training_api_status(enabled: bool) -> Result<(), String> {
    crate::modules::training_api::set_training_api_enabled(enabled)
}

/// Query machine training telemetry
#[tauri::command]
pub async fn get_training_telemetry() -> Result<serde_json::Value, String> {
    let t = crate::modules::training_api::gather_telemetry()?;
    serde_json::to_value(t).map_err(|e| e.to_string())
}

// --- OAuth 命令 ---
