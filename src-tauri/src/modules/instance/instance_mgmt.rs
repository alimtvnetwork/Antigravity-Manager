//! Instance deletion and session wipe.
use super::*;
use std::fs;
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Delete an instance profile
pub fn delete_instance(instance_id: &str) -> Result<(), String> {
    if instance_id == "default" {
        return Err("Cannot delete the default instance".to_string());
    }

    // Automatically close the instance if it is running so delete never fails
    // Justification: best-effort close; close is idempotent and re-attempted on the next action
    crate::error::record_ignored(close_instance(instance_id), "close instance");
    std::thread::sleep(std::time::Duration::from_millis(200));

    let mut registry = load_registry()?;
    let pos = match registry.instances.iter().position(|i| i.id == instance_id) {
        Some(p) => p,
        None => {
            // Idempotent: If instance is already deleted or not found in registry,
            // clean up any lingering directory and return Ok
            if let Ok(instances_root) = get_instances_dir() {
                let instance_folder = instances_root.join(instance_id);
                if instance_folder.exists() {
                    // Justification: cleanup of an optional directory tree; absence is the normal case
                    crate::error::record_ignored(
                        fs::remove_dir_all(&instance_folder),
                        "remove directory tree",
                    );
                }
            }
            return Ok(());
        }
    };

    if registry.instances[pos].is_default {
        return Err("Cannot delete the default instance".to_string());
    }

    let data_dir_to_remove = registry.instances[pos].data_dir.clone();
    let custom_exe_to_remove = registry.instances[pos].executable_path.clone();

    // Update and persist registry first so DB/JSON state is immediately consistent
    registry.instances.remove(pos);
    if registry.active_instance_id == instance_id {
        registry.active_instance_id = "default".to_string();
    }
    save_registry(&registry)?;

    // Remove instance folder and any legacy cloned executable resiliently
    if let Ok(instances_root) = get_instances_dir() {
        let instance_folder = instances_root.join(instance_id);
        if instance_folder.exists() {
            // Justification: cleanup of an optional directory tree; absence is the normal case
            crate::error::record_ignored(
                fs::remove_dir_all(&instance_folder),
                "remove directory tree",
            );
        }
    }
    if !data_dir_to_remove.is_empty() {
        let data_pb = PathBuf::from(&data_dir_to_remove);
        if data_pb.exists() && data_dir_to_remove.contains("instances") {
            // Justification: cleanup of an optional directory tree; absence is the normal case
            crate::error::record_ignored(fs::remove_dir_all(&data_pb), "remove directory tree");
        }
    }
    if let Some(custom_exe) = custom_exe_to_remove {
        if custom_exe.contains(&format!("Antigravity-{}", instance_id)) {
            // Justification: cleanup of an optional file; absence is the normal case
            crate::error::record_ignored(fs::remove_file(custom_exe), "remove file");
        }
    }

    Ok(())
}

/// Wipe authentication session tokens without deleting preferences or extensions
pub fn wipe_instance_session(instance_id: &str) -> Result<(), String> {
    let registry = load_registry()?;
    let config = registry
        .instances
        .iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance {} not found", instance_id))?;

    if is_instance_running(instance_id, &config.data_dir, config.pid) {
        return Err("Cannot wipe session while instance is running. Close it first.".to_string());
    }

    let target_data_path = PathBuf::from(&config.data_dir);
    let state_db = target_data_path
        .join("User")
        .join("globalStorage")
        .join("state.vscdb");

    if state_db.exists() {
        // Justification: cleanup of an optional file; absence is the normal case
        crate::error::record_ignored(fs::remove_file(&state_db), "remove file");
    }

    #[cfg(target_os = "windows")]
    {
        let appdata_db = target_data_path
            .join("AppData")
            .join("Roaming")
            .join("Antigravity")
            .join("User")
            .join("globalStorage")
            .join("state.vscdb");
        if appdata_db.exists() {
            // Justification: cleanup of an optional file; absence is the normal case
            crate::error::record_ignored(fs::remove_file(&appdata_db), "remove file");
        }

        if let Ok(inst_home) = get_instance_home_dir(instance_id) {
            let home_db = inst_home
                .join("AppData")
                .join("Roaming")
                .join("Antigravity")
                .join("User")
                .join("globalStorage")
                .join("state.vscdb");
            if home_db.exists() {
                // Justification: cleanup of an optional file; absence is the normal case
                crate::error::record_ignored(fs::remove_file(&home_db), "remove file");
            }
        }
    }

    purge_volatile_instance_sessions(&target_data_path);
    // Justification: best-effort storage sync; re-synced on every launch and account switch
    crate::error::record_ignored(
        update_instance_app_storage(&target_data_path, None, false),
        "sync app_storage.json",
    );

    Ok(())
}
