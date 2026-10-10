use crate::modules::{account, logger, process};
use std::fs;
use std::path::{Path, PathBuf};

use super::*;

pub(crate) const GLOBAL_BASELINE: &str = "device_original.json";

pub(crate) fn get_data_dir() -> Result<PathBuf, String> {
    account::get_data_dir()
}

/// Find storage.json path (prefer custom/portable paths)
pub fn get_storage_path(target_ide: Option<&str>) -> Result<PathBuf, String> {
    // 1) --user-data-dir flag
    if let Some(user_data_dir) = process::get_user_data_dir_from_process(target_ide) {
        let path = user_data_dir
            .join("User")
            .join("globalStorage")
            .join("storage.json");
        if path.exists() {
            return Ok(path);
        }
    }

    // 2) Portable mode (based on executable data/user-data)
    if let Some(exe_path) = process::get_antigravity_executable_path(target_ide) {
        if let Some(parent) = exe_path.parent() {
            let portable = parent
                .join("data")
                .join("user-data")
                .join("User")
                .join("globalStorage")
                .join("storage.json");
            if portable.exists() {
                return Ok(portable);
            }
        }
    }

    let folder_names: &[&str] = if target_ide == Some("ide") {
        &[
            "Antigravity IDE",
            "antigravity-ide",
            "Antigravity",
            "antigravity",
        ]
    } else if target_ide == Some("code") || target_ide == Some("cursor") {
        &[
            "Antigravity",
            "antigravity",
            "Antigravity IDE",
            "antigravity-ide",
        ]
    } else if target_ide == Some("classic") {
        &["Antigravity", "antigravity"]
    } else {
        // target_ide = None: try classic and IDE folder variations
        &[
            "Antigravity",
            "Antigravity IDE",
            "antigravity",
            "antigravity-ide",
        ]
    };

    // 3) Standard installation location
    #[cfg(target_os = "macos")]
    {
        let home = dirs::home_dir().ok_or("failed_to_get_home_dir")?;
        for folder_name in folder_names {
            let path = home.join(format!(
                "Library/Application Support/{}/User/globalStorage/storage.json",
                folder_name
            ));
            if path.exists() {
                return Ok(path);
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let appdata =
            std::env::var("APPDATA").map_err(|_| "failed_to_get_appdata_env".to_string())?;
        for folder_name in folder_names {
            let path = PathBuf::from(&appdata)
                .join(folder_name)
                .join("User\\globalStorage\\storage.json");
            if path.exists() {
                return Ok(path);
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let home = dirs::home_dir().ok_or("failed_to_get_home_dir")?;
        for folder_name in folder_names {
            let path = home.join(format!(
                ".config/{}/User/globalStorage/storage.json",
                folder_name
            ));
            if path.exists() {
                return Ok(path);
            }
        }

        let snap_paths = vec![
            home.join("snap/antigravity/current/.config/Antigravity/User/globalStorage/storage.json"),
            home.join("snap/antigravity/current/.config/antigravity/User/globalStorage/storage.json"),
            home.join("snap/antigravity-ide/current/.config/Antigravity IDE/User/globalStorage/storage.json"),
            home.join("snap/code/current/.config/Code/User/globalStorage/storage.json"),
        ];
        for path in snap_paths {
            if path.exists() {
                return Ok(path);
            }
        }

        let flatpak_paths = vec![
            home.join(".var/app/com.antigravity.ide/config/Antigravity IDE/User/globalStorage/storage.json"),
            home.join(".var/app/com.antigravity.ide/config/antigravity/User/globalStorage/storage.json"),
        ];
        for path in flatpak_paths {
            if path.exists() {
                return Ok(path);
            }
        }
    }

    // 4) Auto-healing fallback if not found anywhere
    auto_heal_storage_json(target_ide)
}

pub(crate) fn get_default_storage_candidate_path(
    target_ide: Option<&str>,
) -> Result<PathBuf, String> {
    let folder_name = if target_ide == Some("ide") {
        "Antigravity IDE"
    } else {
        "Antigravity"
    };

    #[cfg(target_os = "macos")]
    {
        let home = dirs::home_dir().ok_or("failed_to_get_home_dir")?;
        return Ok(home.join(format!(
            "Library/Application Support/{}/User/globalStorage/storage.json",
            folder_name
        )));
    }

    #[cfg(target_os = "windows")]
    {
        let appdata =
            std::env::var("APPDATA").map_err(|_| "failed_to_get_appdata_env".to_string())?;
        return Ok(PathBuf::from(&appdata)
            .join(folder_name)
            .join("User\\globalStorage\\storage.json"));
    }

    #[cfg(target_os = "linux")]
    {
        let home = dirs::home_dir().ok_or("failed_to_get_home_dir")?;
        return Ok(home.join(format!(
            ".config/{}/User/globalStorage/storage.json",
            folder_name
        )));
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        let home = dirs::home_dir().ok_or("failed_to_get_home_dir")?;
        Ok(home.join(".config/Antigravity/User/globalStorage/storage.json"))
    }
}

pub(crate) fn auto_heal_storage_json(target_ide: Option<&str>) -> Result<PathBuf, String> {
    let fallback_path = get_default_storage_candidate_path(target_ide)?;
    if let Some(parent) = fallback_path.parent() {
        // Justification: best-effort directory creation; later file ops fail loudly if the directory is actually needed
        crate::error::record_ignored(fs::create_dir_all(parent), "create_dir_all");
    }

    let default_profile = generate_profile();
    let initial_content = serde_json::json!({
        "telemetry": {
            "machineId": default_profile.machine_id,
            "macMachineId": default_profile.mac_machine_id,
            "devDeviceId": default_profile.dev_device_id,
            "sqmId": default_profile.sqm_id,
        },
        "telemetry.machineId": default_profile.machine_id,
        "telemetry.macMachineId": default_profile.mac_machine_id,
        "telemetry.devDeviceId": default_profile.dev_device_id,
        "telemetry.sqmId": default_profile.sqm_id,
        "storage.serviceMachineId": default_profile.dev_device_id,
    });

    if let Ok(serialized) = serde_json::to_string_pretty(&initial_content) {
        if fs::write(&fallback_path, serialized).is_ok() {
            logger::log_info(&format!("auto_healed_storage_json_at: {:?}", fallback_path));
            return Ok(fallback_path);
        }
    }

    Ok(fallback_path)
}

/// Get directory of storage.json
pub fn get_storage_dir() -> Result<PathBuf, String> {
    let path = get_storage_path(None)?;
    path.parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| "failed_to_get_storage_parent_dir".to_string())
}

/// Get state.vscdb path (same directory as storage.json)
pub fn get_state_db_path() -> Result<PathBuf, String> {
    let dir = get_storage_dir()?;
    Ok(dir.join("state.vscdb"))
}
