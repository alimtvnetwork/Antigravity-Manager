//! Miscellaneous instance utilities.
use super::*;
use std::path::Path;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Returns total instance count, active count, and running count.
pub fn count_instances() -> Result<serde_json::Value, String> {
    let instances = list_instances()?;
    let total = instances.len();
    let running = instances.iter().filter(|i| i.is_running).count();
    let active_id = get_active_instance_id().unwrap_or_else(|_| "default".to_string());
    let active_count = if instances.iter().any(|i| i.config.id == active_id) {
        1
    } else {
        0
    };

    Ok(serde_json::json!({
        "total": total,
        "active": active_count,
        "running": running,
        "active_instance_id": active_id
    }))
}

/// Resolve the executable binary name (e.g. "Antigravity.exe" or "Antigravity") for an instance
pub fn resolve_instance_exe_name(instance_id: &str, executable_path: Option<&str>) -> String {
    let is_default =
        instance_id == "default" || instance_id == "__default__" || instance_id.is_empty();
    if is_default {
        #[cfg(target_os = "windows")]
        let default_exe = "Antigravity.exe";
        #[cfg(not(target_os = "windows"))]
        let default_exe = "Antigravity";

        if let Ok(config) = crate::modules::config::load_app_config() {
            if let Some(ide_path) = config.antigravity_ide_executable {
                if let Some(file_name) = Path::new(&ide_path).file_name().and_then(|n| n.to_str()) {
                    if !file_name.trim().is_empty() {
                        return file_name.to_string();
                    }
                }
            }
        }
        default_exe.to_string()
    } else {
        if let Some(ep) = executable_path {
            if let Some(file_name) = Path::new(ep).file_name().and_then(|n| n.to_str()) {
                if !file_name.trim().is_empty() {
                    return file_name.to_string();
                }
            }
        }
        #[cfg(target_os = "windows")]
        return format!("Antigravity-{}.exe", instance_id);
        #[cfg(not(target_os = "windows"))]
        return format!("Antigravity-{}", instance_id);
    }
}
