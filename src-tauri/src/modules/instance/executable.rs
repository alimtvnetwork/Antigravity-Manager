//! Per-instance executable cloning and launch resolution.
use super::*;
use crate::error::AppError;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Export all instance configurations to a JSON string
pub fn export_instances_json() -> Result<String, String> {
    let registry = load_registry()?;
    serde_json::to_string_pretty(&registry)
        .map_err(|e| format!("Failed to export instances JSON: {}", e))
}

/// Import instance configurations from a JSON string, merging with existing
pub fn import_instances_json(json_str: &str) -> Result<Vec<InstanceConfig>, String> {
    let imported: InstanceRegistry = serde_json::from_str(json_str)
        .map_err(|e| format!("Invalid instances JSON payload: {}", e))?;

    let mut registry = load_registry()?;
    let instances_root = get_instances_dir()?;

    for mut inst in imported.instances {
        let is_default_inst = inst.id == "default";
        if is_default_inst {
            continue;
        }

        let inst_dir = instances_root.join(&inst.id).join("data");
        let has_dir = inst_dir.exists();
        if !has_dir {
            // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
            crate::error::record_ignored(fs::create_dir_all(&inst_dir), "create directory");
        }
        inst.data_dir = inst_dir.to_string_lossy().to_string();

        let existing_pos = registry.instances.iter().position(|i| i.id == inst.id);
        if let Some(pos) = existing_pos {
            registry.instances[pos] = inst;
        } else {
            registry.instances.push(inst);
        }
    }

    save_registry(&registry)?;
    Ok(registry.instances)
}

/// Clone or create an isolated executable for an instance regardless of OS
/// True when an existing Windows executable clone still matches the current base
/// executable (same size and not older). Instance executables on Windows are
/// hard links/copies that go stale when the Antigravity updater replaces the
/// base binary, so a stale clone must be dropped and re-linked.
#[cfg(target_os = "windows")]
fn exe_clone_is_fresh(base_exe: &Path, clone_path: &Path) -> bool {
    let base_meta = match std::fs::metadata(base_exe) {
        Ok(m) => m,
        Err(_) => return false,
    };
    let clone_meta = match std::fs::metadata(clone_path) {
        Ok(m) => m,
        Err(_) => return false,
    };
    if base_meta.len() != clone_meta.len() {
        return false;
    }
    match (base_meta.modified(), clone_meta.modified()) {
        (Ok(base_mtime), Ok(clone_mtime)) => clone_mtime >= base_mtime,
        _ => false,
    }
}

/// Determine which executable to launch for an instance, preferring an explicit
/// custom path, then a per-instance clone, then auto-detection. Resolved while
/// running processes are still alive so discovery can inspect them.
pub(crate) fn resolve_launch_executable(
    instance_id: &str,
    is_default: bool,
    custom_exe: Option<&str>,
) -> Result<PathBuf, crate::error::AppError> {
    if !is_default {
        if let Some(p) = custom_exe {
            let pb = PathBuf::from(p);
            let p_lower = p.to_lowercase();
            if pb.exists() && !p_lower.contains(".trash") {
                Ok(pb)
            } else if let Ok(cloned) = clone_instance_executable(instance_id) {
                Ok(PathBuf::from(cloned))
            } else {
                crate::modules::process::detect_antigravity_with_diagnostics(None)
            }
        } else if let Ok(cloned) = clone_instance_executable(instance_id) {
            Ok(PathBuf::from(cloned))
        } else {
            crate::modules::process::detect_antigravity_with_diagnostics(None)
        }
    } else {
        crate::modules::process::detect_antigravity_with_diagnostics(None)
    }
}

pub fn clone_instance_executable(instance_id: &str) -> Result<String, crate::error::AppError> {
    let mut registry = load_registry().map_err(crate::error::AppError::Config)?;
    let pos = registry
        .instances
        .iter()
        .position(|i| i.id == instance_id)
        .ok_or_else(|| {
            crate::error::AppError::Config(format!("Instance {} not found", instance_id))
        })?;

    // 1. Locate base executable
    let base_exe = crate::modules::process::detect_antigravity_with_diagnostics(None)?;

    // 2. Prepare instance bin directory
    let instances_dir = get_instances_dir().map_err(crate::error::AppError::Config)?;
    let instance_bin_dir = instances_dir.join(instance_id).join("bin");
    if !instance_bin_dir.exists() {
        fs::create_dir_all(&instance_bin_dir).map_err(|e| crate::error::AppError::Io(e))?;
    }

    // 3. Platform-specific cloning logic
    #[cfg(target_os = "windows")]
    let cloned_path = {
        let parent_dir = base_exe.parent().unwrap_or(&instance_bin_dir);
        let target_in_parent = parent_dir.join(format!("Antigravity-{}.exe", instance_id));
        // Refresh stale clones: if the base executable was updated (size/mtime
        // changed because the updater replaced the file), the existing hard
        // link/copy still points at the old binary — drop it so a fresh
        // link/copy is created below instead of silently reusing the stale one.
        if target_in_parent.exists() && !exe_clone_is_fresh(&base_exe, &target_in_parent) {
            crate::modules::logger::log_info(&format!(
                "[Instance] Refreshing stale executable clone for instance '{}'",
                instance_id
            ));
            // Justification: the stale clone is immediately replaced by a fresh link/copy below
            crate::error::record_ignored(
                std::fs::remove_file(&target_in_parent),
                "remove stale executable clone",
            );
        }
        let has_target = target_in_parent.exists();
        if has_target {
            target_in_parent
        } else if std::fs::hard_link(&base_exe, &target_in_parent).is_ok() {
            target_in_parent
        } else if std::fs::copy(&base_exe, &target_in_parent).is_ok() {
            target_in_parent
        } else {
            // If hardlink and copy fail (e.g. read-only Program Files), create a launcher cmd script in instance bin
            let launcher_cmd = instance_bin_dir.join(format!("launch-{}.cmd", instance_id));
            let script_content = format!(
                "@echo off\r\nstart \"\" \"{}\" %*\r\n",
                base_exe.to_string_lossy()
            );
            fs::write(&launcher_cmd, script_content).map_err(|e| crate::error::AppError::Io(e))?;
            launcher_cmd
        }
    };

    #[cfg(target_os = "linux")]
    let cloned_path = {
        let launcher_sh = instance_bin_dir.join(format!("antigravity-{}", instance_id));
        let base_str = base_exe.to_string_lossy();
        if base_str.ends_with(".AppImage") {
            let appimage_target =
                instance_bin_dir.join(format!("antigravity-{}.AppImage", instance_id));
            // Justification: cleanup of an optional file; absence is the normal case
            crate::error::record_ignored(std::fs::remove_file(&appimage_target), "remove file");
            // Justification: optional symlink; the direct executable path works without it
            crate::error::record_ignored(
                std::os::unix::fs::symlink(&base_exe, &appimage_target),
                "create symlink",
            );
            if appimage_target.exists() {
                appimage_target
            } else {
                let script_content = format!("#!/bin/sh\nexec \"{}\" \"$@\"\n", base_str);
                fs::write(&launcher_sh, script_content)
                    .map_err(|e| crate::error::AppError::Io(e))?;
                use std::os::unix::fs::PermissionsExt;
                // Justification: permission hardening; the file stays usable with its existing mode if this fails
                crate::error::record_ignored(
                    fs::set_permissions(&launcher_sh, fs::Permissions::from_mode(0o755)),
                    "set file permissions",
                );
                launcher_sh
            }
        } else {
            let script_content = format!("#!/bin/sh\nexec \"{}\" \"$@\"\n", base_str);
            fs::write(&launcher_sh, script_content).map_err(|e| crate::error::AppError::Io(e))?;
            use std::os::unix::fs::PermissionsExt;
            // Justification: permission hardening; the file stays usable with its existing mode if this fails
            crate::error::record_ignored(
                fs::set_permissions(&launcher_sh, fs::Permissions::from_mode(0o755)),
                "set file permissions",
            );
            launcher_sh
        }
    };

    #[cfg(target_os = "macos")]
    let cloned_path = {
        let launcher_sh = instance_bin_dir.join(format!("antigravity-{}", instance_id));
        let base_str = base_exe.to_string_lossy();
        let is_app_bundle = base_str.ends_with(".app") || base_exe.is_dir();
        let script_content = if is_app_bundle {
            format!(
                "#!/bin/sh\nexec open -n -a \"{}\" --args \"$@\"\n",
                base_str
            )
        } else {
            format!("#!/bin/sh\nexec \"{}\" \"$@\"\n", base_str)
        };
        fs::write(&launcher_sh, script_content).map_err(|e| {
            let trace = std::backtrace::Backtrace::capture();
            crate::modules::logger::log_error(&format!(
                "[Instance] Failed to write macOS launcher script {:?}: {}. Backtrace:\n{:?}",
                launcher_sh, e, trace
            ));
            crate::error::AppError::Io(e)
        })?;
        use std::os::unix::fs::PermissionsExt;
        // Justification: permission hardening; the file stays usable with its existing mode if this fails
        crate::error::record_ignored(
            fs::set_permissions(&launcher_sh, fs::Permissions::from_mode(0o755)),
            "set file permissions",
        );
        launcher_sh
    };

    let cloned_str = cloned_path.to_string_lossy().to_string();
    registry.instances[pos].executable_path = Some(cloned_str.clone());
    save_registry(&registry).map_err(crate::error::AppError::Config)?;

    crate::modules::logger::log_info(&format!(
        "[Instance] Cloned executable for instance '{}': {}",
        instance_id, cloned_str
    ));

    Ok(cloned_str)
}

/// Set or clear custom executable path for an instance
pub fn set_instance_executable(
    instance_id: &str,
    executable_path: Option<String>,
) -> Result<(), crate::error::AppError> {
    let mut registry = load_registry().map_err(crate::error::AppError::Config)?;
    let pos = registry
        .instances
        .iter()
        .position(|i| i.id == instance_id)
        .ok_or_else(|| {
            crate::error::AppError::Config(format!("Instance {} not found", instance_id))
        })?;

    registry.instances[pos].executable_path = executable_path;
    save_registry(&registry).map_err(crate::error::AppError::Config)?;
    Ok(())
}
