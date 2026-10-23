use super::*;

#[tauri::command]
pub async fn get_antigravity_path(
    bypass_config: Option<bool>,
) -> Result<String, crate::error::AppError> {
    // 1. Query from configuration first (unless explicitly required to bypass)
    if bypass_config != Some(true) {
        if let Ok(config) = crate::modules::config::load_app_config() {
            if let Some(path) = config.antigravity_executable {
                if std::path::Path::new(&path).exists() {
                    return Ok(path);
                }
            }
        }
    }

    // 2. Perform real-time probing (with full diagnostic trace and call stack)
    let path = crate::modules::process::detect_antigravity_with_diagnostics(None)?;
    Ok(path.to_string_lossy().to_string())
}

/// Get Antigravity CLI (agy) executable path
#[tauri::command]
pub async fn get_antigravity_cli_path(bypass_config: Option<bool>) -> Result<String, String> {
    // 1. Query from configuration first (unless explicitly required to bypass)
    if bypass_config != Some(true) {
        if let Ok(config) = crate::modules::config::load_app_config() {
            if let Some(path) = config.antigravity_cli_executable {
                if std::path::Path::new(&path).exists() {
                    return Ok(path);
                }
            }
        }
    }

    // 2. Perform real-time detection
    match crate::modules::process::get_antigravity_cli_executable_path() {
        Some(path) => Ok(path.to_string_lossy().to_string()),
        None => Err("Antigravity CLI (agy) installation path not found".to_string()),
    }
}

/// Get Antigravity startup parameters
#[tauri::command]
pub async fn get_antigravity_args() -> Result<Vec<String>, String> {
    match crate::modules::process::get_args_from_running_process(None) {
        Some(args) => Ok(args),
        None => Err("No running Antigravity process found".to_string()),
    }
}
