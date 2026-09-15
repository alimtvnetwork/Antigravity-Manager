use crate::models::instance::{InstanceConfig, InstanceRegistry, InstanceStatus};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use sysinfo::System;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Get base directory for storing instance profiles
pub fn get_instances_dir() -> Result<PathBuf, String> {
    let base_dir = crate::modules::config::get_config_dir()
        .map_err(|e| format!("Failed to get config dir: {}", e))?;
    let instances_dir = base_dir.join("instances");
    if !instances_dir.exists() {
        fs::create_dir_all(&instances_dir)
            .map_err(|e| format!("Failed to create instances directory: {}", e))?;
    }
    Ok(instances_dir)
}

/// Path to instances.json registry
pub fn get_registry_path() -> Result<PathBuf, String> {
    let dir = get_instances_dir()?;
    Ok(dir.join("instances.json"))
}

/// Fallback default user data directory
pub fn get_default_antigravity_data_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            let path = PathBuf::from(appdata).join("Antigravity");
            if path.exists() {
                return path;
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            let path = PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("Antigravity");
            if path.exists() {
                return path;
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(config_home) = std::env::var("XDG_CONFIG_HOME") {
            let path = PathBuf::from(config_home).join("Antigravity");
            if path.exists() {
                return path;
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            let path = PathBuf::from(home).join(".config").join("Antigravity");
            if path.exists() {
                return path;
            }
        }
    }

    get_instances_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("default")
        .join("data")
}

/// Load instance registry or initialize with default instance
pub fn load_registry() -> Result<InstanceRegistry, String> {
    let registry_path = get_registry_path()?;
    if !registry_path.exists() {
        let default_dir = get_default_antigravity_data_dir();
        let now = chrono::Utc::now().timestamp();
        let default_instance = InstanceConfig {
            id: "default".to_string(),
            name: "Default".to_string(),
            data_dir: default_dir.to_string_lossy().to_string(),
            extensions_dir: None,
            bound_account_id: None,
            bound_email: None,
            created_at: now,
            last_used: now,
            is_default: true,
        };

        let registry = InstanceRegistry {
            active_instance_id: "default".to_string(),
            instances: vec![default_instance],
        };
        save_registry(&registry)?;
        return Ok(registry);
    }

    let content = fs::read_to_string(&registry_path)
        .map_err(|e| format!("Failed to read instances registry: {}", e))?;
    let registry: InstanceRegistry = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse instances registry: {}", e))?;

    Ok(registry)
}

/// Save instance registry
pub fn save_registry(registry: &InstanceRegistry) -> Result<(), String> {
    let registry_path = get_registry_path()?;
    let json = serde_json::to_string_pretty(registry)
        .map_err(|e| format!("Failed to serialize instances registry: {}", e))?;
    fs::write(&registry_path, json)
        .map_err(|e| format!("Failed to write instances registry: {}", e))?;
    Ok(())
}

/// Inspect running Antigravity processes matching an instance data_dir
pub fn find_pids_for_data_dir(data_dir: &str, is_default: bool) -> Vec<u32> {
    let mut system = System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All);

    let normalized_target = data_dir.to_lowercase().replace('\\', "/");
    let mut matched_pids = Vec::new();

    for (pid, process) in system.processes() {
        let name = process.name().to_string_lossy().to_lowercase();
        let exe = process
            .exe()
            .and_then(|p| p.to_str())
            .unwrap_or("")
            .to_lowercase();

        let is_antigravity = name.contains("antigravity") || exe.contains("antigravity");
        if !is_antigravity {
            continue;
        }

        let args = process.cmd();
        let args_str = args
            .iter()
            .map(|a| a.to_string_lossy().to_lowercase().replace('\\', "/"))
            .collect::<Vec<String>>()
            .join(" ");

        let is_helper =
            args_str.contains("--type=") || name.contains("helper") || name.contains("crashpad");
        if is_helper {
            continue;
        }

        let has_user_data_arg = args_str.contains("--user-data-dir");
        if has_user_data_arg {
            if args_str.contains(&normalized_target) {
                matched_pids.push(pid.as_u32());
            }
        } else if is_default {
            matched_pids.push(pid.as_u32());
        }
    }

    matched_pids
}

/// List all instances with live running status
pub fn list_instances() -> Result<Vec<InstanceStatus>, String> {
    let registry = load_registry()?;
    let mut statuses = Vec::new();

    let mut system = System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All);

    for config in registry.instances {
        let pids = find_pids_for_data_dir(&config.data_dir, config.is_default);
        let is_running = !pids.is_empty();
        let first_pid = pids.first().copied();

        let memory_mb = first_pid.and_then(|p| {
            system
                .process(sysinfo::Pid::from_u32(p))
                .map(|proc| proc.memory() as f64 / (1024.0 * 1024.0))
        });

        statuses.push(InstanceStatus {
            config,
            is_running,
            pid: first_pid,
            memory_mb,
        });
    }

    Ok(statuses)
}

/// Create a new isolated profile
pub fn create_instance(name: String) -> Result<InstanceConfig, String> {
    let mut registry = load_registry()?;
    let trimmed_name = name.trim();
    if trimmed_name.is_empty() {
        return Err("Profile name cannot be empty".to_string());
    }

    let slug: String = trimmed_name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    let sanitized_slug = slug.trim_matches('-').to_string();

    let now = chrono::Utc::now().timestamp();
    let instance_id = format!("{}-{}", sanitized_slug, now % 10000);

    let instances_root = get_instances_dir()?;
    let instance_data_dir = instances_root.join(&instance_id).join("data");

    fs::create_dir_all(&instance_data_dir)
        .map_err(|e| format!("Failed to create instance directory: {}", e))?;

    let config = InstanceConfig {
        id: instance_id,
        name: trimmed_name.to_string(),
        data_dir: instance_data_dir.to_string_lossy().to_string(),
        extensions_dir: None,
        bound_account_id: None,
        bound_email: None,
        created_at: now,
        last_used: now,
        is_default: false,
    };

    registry.instances.push(config.clone());
    save_registry(&registry)?;

    Ok(config)
}

/// Copy/clone an existing profile
pub fn copy_instance(source_id: &str, target_name: String) -> Result<InstanceConfig, String> {
    let registry = load_registry()?;
    let source = registry
        .instances
        .iter()
        .find(|i| i.id == source_id)
        .ok_or_else(|| format!("Source instance {} not found", source_id))?
        .clone();

    let new_instance = create_instance(target_name)?;

    let src_path = PathBuf::from(&source.data_dir);
    let dst_path = PathBuf::from(&new_instance.data_dir);

    // Recursively copy configuration if source exists
    if src_path.exists() {
        let user_settings_src = src_path.join("User");
        if user_settings_src.exists() {
            let user_settings_dst = dst_path.join("User");
            let _ = copy_dir_recursive(&user_settings_src, &user_settings_dst);
        }
    }

    Ok(new_instance)
}

/// Helper function to copy directories recursively
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_child = dst.join(entry.file_name());
        if file_type.is_dir() {
            // Skip large cache directories
            let dir_name = entry.file_name().to_string_lossy().to_lowercase();
            if dir_name.contains("cache") || dir_name == "crashpad" {
                continue;
            }
            copy_dir_recursive(&entry.path(), &dest_child)?;
        } else {
            let _ = fs::copy(entry.path(), dest_child);
        }
    }
    Ok(())
}

/// Delete an instance profile
pub fn delete_instance(instance_id: &str) -> Result<(), String> {
    let mut registry = load_registry()?;
    if instance_id == "default" {
        return Err("Cannot delete the default instance".to_string());
    }

    let pos = registry
        .instances
        .iter()
        .position(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance {} not found", instance_id))?;

    let config = &registry.instances[pos];
    let pids = find_pids_for_data_dir(&config.data_dir, config.is_default);
    if !pids.is_empty() {
        return Err("Cannot delete instance while it is running. Close it first.".to_string());
    }

    // Remove directory
    let instances_root = get_instances_dir()?;
    let instance_folder = instances_root.join(instance_id);
    if instance_folder.exists() {
        let _ = fs::remove_dir_all(instance_folder);
    }

    registry.instances.remove(pos);
    if registry.active_instance_id == instance_id {
        registry.active_instance_id = "default".to_string();
    }
    save_registry(&registry)?;

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

    let pids = find_pids_for_data_dir(&config.data_dir, config.is_default);
    if !pids.is_empty() {
        return Err("Cannot wipe session while instance is running. Close it first.".to_string());
    }

    let state_db = PathBuf::from(&config.data_dir)
        .join("User")
        .join("globalStorage")
        .join("state.vscdb");

    if state_db.exists() {
        let _ = fs::remove_file(&state_db);
    }

    Ok(())
}

/// Launch a specific instance
pub fn launch_instance(instance_id: &str) -> Result<(), String> {
    let mut registry = load_registry()?;
    let pos = registry
        .instances
        .iter()
        .position(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance {} not found", instance_id))?;

    let config = &mut registry.instances[pos];
    config.last_used = chrono::Utc::now().timestamp();
    let data_dir = config.data_dir.clone();
    let is_default = config.is_default;
    save_registry(&registry)?;

    let exe_path = crate::modules::process::get_antigravity_executable_path(None)
        .ok_or_else(|| "Could not locate Antigravity executable on this system".to_string())?;

    let exe_str = exe_path.to_string_lossy().to_string();
    let mut cmd = Command::new(&exe_str);

    if !is_default {
        cmd.arg(format!("--user-data-dir={}", data_dir));
    }

    #[cfg(target_os = "linux")]
    {
        // Bypass GNOME Keyring by isolating tokens in local state.vscdb
        cmd.arg("--password-store=basic");
        crate::modules::process::clean_appimage_env(&mut cmd);
    }

    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(0x08000000);
    }

    cmd.spawn()
        .map_err(|e| format!("Failed to spawn instance process: {}", e))?;

    Ok(())
}

/// Close only the process associated with this instance
pub fn close_instance(instance_id: &str) -> Result<(), String> {
    let registry = load_registry()?;
    let config = registry
        .instances
        .iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance {} not found", instance_id))?;

    let pids = find_pids_for_data_dir(&config.data_dir, config.is_default);
    if pids.is_empty() {
        return Ok(());
    }

    for pid in pids {
        #[cfg(target_os = "windows")]
        {
            let _ = Command::new("taskkill")
                .args(["/F", "/PID", &pid.to_string()])
                .creation_flags(0x08000000)
                .output();
        }

        #[cfg(not(target_os = "windows"))]
        {
            let _ = Command::new("kill")
                .args(["-15", &pid.to_string()])
                .output();
        }
    }

    Ok(())
}

/// Get currently active instance ID
pub fn get_active_instance_id() -> Result<String, String> {
    let registry = load_registry()?;
    Ok(registry.active_instance_id)
}

/// Set currently active instance ID
pub fn set_active_instance_id(instance_id: &str) -> Result<(), String> {
    let mut registry = load_registry()?;
    if !registry.instances.iter().any(|i| i.id == instance_id) {
        return Err(format!("Instance {} does not exist", instance_id));
    }
    registry.active_instance_id = instance_id.to_string();
    save_registry(&registry)?;
    Ok(())
}

/// Bind an account to an instance
pub fn bind_account_to_instance(
    instance_id: &str,
    account_id: &str,
    email: &str,
) -> Result<(), String> {
    let mut registry = load_registry()?;
    if let Some(inst) = registry.instances.iter_mut().find(|i| i.id == instance_id) {
        inst.bound_account_id = Some(account_id.to_string());
        inst.bound_email = Some(email.to_string());
        inst.last_used = chrono::Utc::now().timestamp();
        save_registry(&registry)?;
        Ok(())
    } else {
        Err(format!("Instance {} not found", instance_id))
    }
}

/// Switch account and inject into a specific target instance without terminating siblings
pub async fn switch_account_to_instance(
    account_id: &str,
    target_instance_id: Option<&str>,
) -> Result<(), String> {
    let mut account = crate::modules::account::load_account(account_id)?;
    let fresh_token = crate::modules::oauth::ensure_fresh_token(&account.token, Some(&account.id))
        .await
        .map_err(|e| format!("Failed to refresh token: {}", e))?;
    if fresh_token.access_token != account.token.access_token {
        account.token = fresh_token;
        crate::modules::account::save_account(&account)?;
    }

    let registry = load_registry()?;
    let target_id = target_instance_id
        .map(|s| s.to_string())
        .unwrap_or_else(|| registry.active_instance_id.clone());

    let instance = registry
        .instances
        .iter()
        .find(|i| i.id == target_id)
        .ok_or_else(|| format!("Target instance {} not found", target_id))?;

    let db_dir = PathBuf::from(&instance.data_dir)
        .join("User")
        .join("globalStorage");
    if !db_dir.exists() {
        fs::create_dir_all(&db_dir).map_err(|e| format!("Failed to create db dir: {}", e))?;
    }
    let db_path = db_dir.join("state.vscdb");

    // Close only this specific instance window before database injection
    let _ = close_instance(&instance.id);

    // Inject token directly into instance's isolated state.vscdb
    crate::modules::db::inject_token(
        &db_path,
        &account.token.access_token,
        &account.token.refresh_token,
        account.token.expiry_timestamp,
        &account.email,
        account.token.is_gcp_tos,
        account.token.project_id.as_deref(),
        account.token.id_token.as_deref(),
        account.token.oauth_client_key.as_deref(),
        None,
    )?;

    if let Some(ref profile) = account.device_profile {
        let _ = crate::modules::db::write_service_machine_id(&db_path, &profile.mac_machine_id);
    }

    // Bind account in registry and set as active
    bind_account_to_instance(&instance.id, &account.id, &account.email)?;
    let _ = set_active_instance_id(&instance.id);

    account.update_last_used();
    let _ = crate::modules::account::save_account(&account);

    // Launch instance
    launch_instance(&instance.id)?;

    Ok(())
}
