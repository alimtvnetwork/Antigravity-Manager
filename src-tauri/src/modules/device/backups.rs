use crate::models::DeviceProfile;
use crate::modules::{account, logger, process};
use rand::{distributions::Alphanumeric, Rng};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use super::*;

/// Load/Save global original profile (shared across all accounts)
pub fn load_global_original() -> Option<DeviceProfile> {
    if let Ok(dir) = get_data_dir() {
        let path = dir.join(GLOBAL_BASELINE);
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(profile) = serde_json::from_str::<DeviceProfile>(&content) {
                    return Some(profile);
                }
            }
        }
    }
    None
}

pub fn save_global_original(profile: &DeviceProfile) -> Result<(), String> {
    let dir = get_data_dir()?;
    let path = dir.join(GLOBAL_BASELINE);
    if path.exists() {
        return Ok(()); // already exists, don't overwrite
    }
    let content =
        serde_json::to_string_pretty(profile).map_err(|e| format!("serialize_failed: {}", e))?;
    fs::write(&path, content).map_err(|e| format!("write_failed: {}", e))
}

/// List storage.json backups in current directory (descending by time)
#[allow(dead_code)]
pub fn list_backups(storage_path: &Path) -> Result<Vec<PathBuf>, String> {
    let dir = storage_path
        .parent()
        .ok_or_else(|| "failed_to_get_storage_parent_dir".to_string())?;
    let mut backups = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("storage.json.backup_") {
                    backups.push(path);
                }
            }
        }
    }
    // Sort by modification time (new to old)
    backups.sort_by(|a, b| {
        let ma = fs::metadata(a).and_then(|m| m.modified()).ok();
        let mb = fs::metadata(b).and_then(|m| m.modified()).ok();
        mb.cmp(&ma)
    });
    Ok(backups)
}

/// Restore backup to storage.json. If use_oldest=true, use oldest backup, else use latest.
#[allow(dead_code)]
pub fn restore_backup(storage_path: &Path, use_oldest: bool) -> Result<PathBuf, String> {
    let backups = list_backups(storage_path)?;
    if backups.is_empty() {
        return Err("no_backups_found".to_string());
    }
    let target = if use_oldest {
        backups.last().unwrap().clone()
    } else {
        backups.first().unwrap().clone()
    };
    // backup current first
    let _ = backup_storage(storage_path)?;
    fs::copy(&target, storage_path).map_err(|e| format!("restore_failed: {}", e))?;
    logger::log_info(&format!("storage_json_restored: {:?}", target));
    Ok(target)
}

/// Generate a new set of device fingerprints (Cursor/VSCode style)
pub fn generate_profile() -> DeviceProfile {
    DeviceProfile {
        machine_id: format!("auth0|user_{}", random_hex(32)),
        mac_machine_id: new_standard_machine_id(),
        dev_device_id: Uuid::new_v4().to_string(),
        sqm_id: format!("{{{}}}", Uuid::new_v4().to_string().to_uppercase()),
    }
}

pub(crate) fn random_hex(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect::<String>()
        .to_lowercase()
}

pub(crate) fn new_standard_machine_id() -> String {
    // xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx (y in 8..b)
    let mut rng = rand::thread_rng();
    let mut id = String::with_capacity(36);
    for ch in "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx".chars() {
        if ch == '-' || ch == '4' {
            id.push(ch);
        } else if ch == 'x' {
            id.push_str(&format!("{:x}", rng.gen_range(0..16)));
        } else if ch == 'y' {
            id.push_str(&format!("{:x}", rng.gen_range(8..12)));
        }
    }
    id
}
