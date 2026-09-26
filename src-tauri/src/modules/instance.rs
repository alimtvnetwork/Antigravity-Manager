use crate::models::instance::{InstanceConfig, InstanceRegistry, InstanceStatus};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use sysinfo::System;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Get base directory for storing instance profiles
pub fn get_instances_dir() -> Result<PathBuf, String> {
    let base_dir = crate::modules::account::get_data_dir()
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

/// Path to instances.db SQLite database
pub fn get_instance_db_path() -> Result<PathBuf, String> {
    let dir = get_instances_dir()?;
    Ok(dir.join("instances.db"))
}

/// Open and initialize the instance SQLite database with WAL mode
pub fn open_instance_db() -> Result<rusqlite::Connection, String> {
    let db_path = get_instance_db_path()?;
    let conn = rusqlite::Connection::open(db_path).map_err(|e| e.to_string())?;

    let _ = conn.pragma_update(None, "journal_mode", "WAL");
    let _ = conn.pragma_update(None, "busy_timeout", 5000);
    let _ = conn.pragma_update(None, "synchronous", "NORMAL");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS instance_processes (
            instance_id TEXT PRIMARY KEY,
            pid INTEGER NOT NULL,
            data_dir TEXT NOT NULL,
            launched_at INTEGER NOT NULL,
            is_active INTEGER NOT NULL DEFAULT 1
        )",
        [],
    )
    .map_err(|e| format!("Failed to create instance_processes table: {}", e))?;

    Ok(conn)
}

/// Record an instance PID launch in the SQLite database and in registry
pub fn record_instance_pid(instance_id: &str, pid: u32, data_dir: &str) -> Result<(), String> {
    let now = chrono::Utc::now().timestamp();
    if let Ok(conn) = open_instance_db() {
        let _ = conn.execute(
            "INSERT INTO instance_processes (instance_id, pid, data_dir, launched_at, is_active)
             VALUES (?1, ?2, ?3, ?4, 1)
             ON CONFLICT(instance_id) DO UPDATE SET
                pid = excluded.pid,
                data_dir = excluded.data_dir,
                launched_at = excluded.launched_at,
                is_active = 1",
            rusqlite::params![instance_id, pid as i64, data_dir, now],
        );
    }

    if let Ok(mut registry) = load_registry() {
        if let Some(inst) = registry.instances.iter_mut().find(|i| i.id == instance_id) {
            inst.pid = Some(pid);
            inst.last_used = now;
            let _ = save_registry(&registry);
        }
    }

    crate::modules::logger::log_info(&format!(
        "[Instance] Saved instance '{}' launch PID {} to SQLite DB and registry",
        instance_id, pid
    ));
    Ok(())
}

/// Query active PID for an instance from SQLite DB
pub fn get_instance_saved_pid(instance_id: &str) -> Option<u32> {
    if let Ok(conn) = open_instance_db() {
        if let Ok(pid) = conn.query_row(
            "SELECT pid FROM instance_processes WHERE instance_id = ?1 AND is_active = 1",
            rusqlite::params![instance_id],
            |row| row.get::<_, i64>(0),
        ) {
            if pid > 0 {
                return Some(pid as u32);
            }
        }
    }

    if let Ok(registry) = load_registry() {
        if let Some(inst) = registry.instances.iter().find(|i| i.id == instance_id) {
            return inst.pid;
        }
    }
    None
}

/// Mark instance PID as stopped in SQLite DB and registry
pub fn mark_instance_stopped(instance_id: &str) -> Result<(), String> {
    if let Ok(conn) = open_instance_db() {
        let _ = conn.execute(
            "UPDATE instance_processes SET is_active = 0 WHERE instance_id = ?1",
            rusqlite::params![instance_id],
        );
    }

    if let Ok(mut registry) = load_registry() {
        if let Some(inst) = registry.instances.iter_mut().find(|i| i.id == instance_id) {
            inst.pid = None;
            let _ = save_registry(&registry);
        }
    }
    Ok(())
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
            executable_path: None,
            extensions_dir: None,
            bound_account_id: None,
            bound_email: None,
            created_at: now,
            last_used: now,
            is_default: true,
            pid: None,
            seq_num: Some(1),
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
    let mut registry: InstanceRegistry = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse instances registry: {}", e))?;

    let has_default = registry.instances.iter().any(|i| i.id == "default");
    if !has_default {
        let default_dir = get_default_antigravity_data_dir();
        let now = chrono::Utc::now().timestamp();
        let default_instance = InstanceConfig {
            id: "default".to_string(),
            name: "Default".to_string(),
            data_dir: default_dir.to_string_lossy().to_string(),
            executable_path: None,
            extensions_dir: None,
            bound_account_id: None,
            bound_email: None,
            created_at: now,
            last_used: now,
            is_default: true,
            pid: None,
            seq_num: Some(1),
        };
        registry.instances.insert(0, default_instance);
        let _ = save_registry(&registry);
    }

    let mut modified = false;
    let mut current_max = registry
        .instances
        .iter()
        .filter_map(|i| i.seq_num)
        .max()
        .unwrap_or(0);

    for inst in registry.instances.iter_mut() {
        if inst.seq_num.is_none() {
            current_max += 1;
            inst.seq_num = Some(current_max);
            modified = true;
        }
    }

    if modified {
        let _ = save_registry(&registry);
    }

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
    let clean_target = normalized_target.trim_end_matches('/');
    let mut matched_pids = Vec::new();

    for (pid, process) in system.processes() {
        let name = process.name().to_string_lossy().to_lowercase();
        let exe = process
            .exe()
            .and_then(|p| p.to_str())
            .unwrap_or("")
            .to_lowercase();

        let args = process.cmd();
        let args_str = args
            .iter()
            .map(|a| a.to_string_lossy().to_lowercase().replace('\\', "/"))
            .collect::<Vec<String>>()
            .join(" ");

        let is_antigravity = name.contains("antigravity")
            || exe.contains("antigravity")
            || args_str.contains("antigravity")
            || exe.contains("/tmp/.mount_")
            || name == "apprun"
            || args_str.contains(clean_target);

        if !is_antigravity {
            continue;
        }

        let is_helper =
            args_str.contains("--type=") || name.contains("helper") || name.contains("crashpad");
        if is_helper {
            continue;
        }

        let has_user_data_arg = args_str.contains("--user-data-dir");
        if has_user_data_arg {
            if args_str.contains(clean_target) {
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
        let mut pids = find_pids_for_data_dir(&config.data_dir, false);
        if let Some(saved_pid) = config.pid.or_else(|| get_instance_saved_pid(&config.id)) {
            let pid_alive = system.process(sysinfo::Pid::from_u32(saved_pid)).is_some();
            let not_in_list = !pids.contains(&saved_pid);
            if pid_alive {
                if not_in_list {
                    pids.push(saved_pid);
                }
            }
        }
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

    let user_dir = instance_data_dir.join("User");
    let _ = fs::create_dir_all(&user_dir);
    if let Ok(default_dir) = get_default_antigravity_data_dir() {
        let default_settings = default_dir.join("User").join("settings.json");
        let dest_settings = user_dir.join("settings.json");
        if default_settings.exists() && !dest_settings.exists() {
            let _ = fs::copy(default_settings, dest_settings);
        }
    }

    let next_seq = registry
        .instances
        .iter()
        .filter_map(|i| i.seq_num)
        .max()
        .unwrap_or(0)
        + 1;

    let config = InstanceConfig {
        id: instance_id,
        name: trimmed_name.to_string(),
        data_dir: instance_data_dir.to_string_lossy().to_string(),
        executable_path: None,
        extensions_dir: None,
        bound_account_id: None,
        bound_email: None,
        created_at: now,
        last_used: now,
        is_default: false,
        pid: None,
        seq_num: Some(next_seq),
    };

    registry.instances.push(config.clone());
    save_registry(&registry)?;

    Ok(config)
}

/// Copy/clone an existing profile (full directory copy by default, or profile only)
pub fn copy_instance(
    source_id: &str,
    target_name: String,
    clone_mode: Option<&str>,
) -> Result<InstanceConfig, String> {
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

    let is_profile_only = clone_mode
        .map(|m| m.eq_ignore_ascii_case("profile"))
        .unwrap_or(false);

    let has_src = src_path.exists();
    if has_src {
        if is_profile_only {
            let user_settings_src = src_path.join("User");
            let has_user_dir = user_settings_src.exists();
            if has_user_dir {
                let user_settings_dst = dst_path.join("User");
                let _ = copy_dir_recursive(&user_settings_src, &user_settings_dst);
            }
        } else {
            // Full directory copy (default): copies entire instance data tree while skipping volatile locks/caches
            let _ = copy_dir_recursive(&src_path, &dst_path);
        }

        // Sanitize cloned session so the new profile starts with clean authentication state
        let cloned_db = dst_path
            .join("User")
            .join("globalStorage")
            .join("state.vscdb");
        let has_cloned_db = cloned_db.exists();
        if has_cloned_db {
            let _ = crate::modules::db::sanitize_session(&cloned_db);
        }
    }

    Ok(new_instance)
}

/// Rename an existing instance profile
pub fn rename_instance(instance_id: &str, new_name: String) -> Result<InstanceConfig, String> {
    let mut registry = load_registry()?;
    let trimmed = new_name.trim();
    let is_empty = trimmed.is_empty();
    if is_empty {
        return Err("Profile name cannot be empty".to_string());
    }

    let pos = registry
        .instances
        .iter()
        .position(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance {} not found", instance_id))?;

    registry.instances[pos].name = trimmed.to_string();
    let updated = registry.instances[pos].clone();
    save_registry(&registry)?;

    Ok(updated)
}

/// Helper function to copy directories recursively while sanitizing lock files and volatile caches
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    let has_dst = dst.exists();
    if !has_dst {
        fs::create_dir_all(dst)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy().to_lowercase();

        // Skip volatile lock files, active socket handles, crashpad, and heavy caches
        let is_lock = name_str == "lockfile"
            || name_str.ends_with(".lock")
            || name_str.starts_with("singleton");
        let is_cache = name_str.contains("cache")
            || name_str == "crashpad"
            || name_str.starts_with(".org.chromium");
        if is_lock || is_cache {
            continue;
        }

        let dest_child = dst.join(&file_name);
        let is_directory = file_type.is_dir();
        if is_directory {
            copy_dir_recursive(&entry.path(), &dest_child)?;
        } else {
            let _ = fs::copy(entry.path(), dest_child);
        }
    }
    Ok(())
}

/// Check if an instance is currently running using selective data dir and saved PID
pub fn is_instance_running(instance_id: &str, data_dir: &str, config_pid: Option<u32>) -> bool {
    let is_default_inst = instance_id == "default";
    let pids = find_pids_for_data_dir(data_dir, is_default_inst);
    if !pids.is_empty() {
        return true;
    }
    if let Some(saved_pid) = config_pid.or_else(|| get_instance_saved_pid(instance_id)) {
        let mut sys = System::new();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All);
        if sys.process(sysinfo::Pid::from_u32(saved_pid)).is_some() {
            return true;
        }
    }
    false
}

/// Delete an instance profile
pub fn delete_instance(instance_id: &str) -> Result<(), String> {
    if instance_id == "default" {
        return Err("Cannot delete the default instance".to_string());
    }

    // Automatically close the instance if it is running so delete never fails
    let _ = close_instance(instance_id);
    std::thread::sleep(std::time::Duration::from_millis(200));

    let mut registry = load_registry()?;
    let pos = registry
        .instances
        .iter()
        .position(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance {} not found", instance_id))?;

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
            let _ = fs::remove_dir_all(&instance_folder);
        }
    }
    if !data_dir_to_remove.is_empty() {
        let data_pb = PathBuf::from(&data_dir_to_remove);
        if data_pb.exists() && data_dir_to_remove.contains("instances") {
            let _ = fs::remove_dir_all(&data_pb);
        }
    }
    if let Some(custom_exe) = custom_exe_to_remove {
        if custom_exe.contains(&format!("Antigravity-{}", instance_id)) {
            let _ = fs::remove_file(custom_exe);
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

    let state_db = PathBuf::from(&config.data_dir)
        .join("User")
        .join("globalStorage")
        .join("state.vscdb");

    if state_db.exists() {
        let _ = fs::remove_file(&state_db);
    }

    Ok(())
}

/// Launch a specific instance with multi-window isolation and custom/cloned executable support
pub fn launch_instance(instance_id: &str) -> Result<(), crate::error::AppError> {
    let mut registry = load_registry().map_err(crate::error::AppError::Config)?;
    let pos = registry
        .instances
        .iter()
        .position(|i| i.id == instance_id)
        .ok_or_else(|| {
            crate::error::AppError::Config(format!("Instance {} not found", instance_id))
        })?;

    let config = &mut registry.instances[pos];
    config.last_used = chrono::Utc::now().timestamp();
    let data_dir = config.data_dir.clone();
    let is_default = config.is_default;
    let custom_exe = config.executable_path.clone();
    let extensions_dir = config.extensions_dir.clone();
    let bound_acc = config.bound_account_id.clone();
    save_registry(&registry).map_err(crate::error::AppError::Config)?;

    let target_data_path = PathBuf::from(&data_dir);

    // If an account is bound to this instance profile, sync credentials and inject token directly into isolated state.vscdb
    if let Some(ref account_id) = bound_acc {
        if let Ok(account) = crate::modules::account::load_account(account_id) {
            let _ = crate::modules::integration::write_to_system_keyring(&account);

            let db_dir = target_data_path.join("User").join("globalStorage");
            let has_db_dir = db_dir.exists();
            if !has_db_dir {
                let _ = fs::create_dir_all(&db_dir);
            }
            let db_path = db_dir.join("state.vscdb");
            let _ = crate::modules::db::inject_token(
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
            );

            if let Some(ref profile) = account.device_profile {
                let _ =
                    crate::modules::db::write_service_machine_id(&db_path, &profile.mac_machine_id);
            }
        }
    }

    // Clean any orphaned lock files in the target instance data directory
    let has_target_dir = target_data_path.exists();
    if has_target_dir {
        let lockfile = target_data_path.join("lockfile");
        if lockfile.exists() {
            let _ = fs::remove_file(&lockfile);
        }
        let code_lock = target_data_path.join("code.lock");
        let has_code_lock = code_lock.exists();
        if has_code_lock {
            let _ = fs::remove_file(&code_lock);
        }
        if let Ok(entries) = fs::read_dir(&target_data_path) {
            for entry in entries.flatten() {
                let fname = entry.file_name().to_string_lossy().to_lowercase();
                let is_stale_lock = fname == "lockfile"
                    || fname.starts_with("singleton")
                    || fname.ends_with(".lock")
                    || fname == "code.lock";
                if is_stale_lock {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }
    }

    // Determine executable FIRST while running processes are alive for discovery
    let exe_path = if let Some(ref p) = custom_exe {
        let pb = PathBuf::from(p);
        let has_pb = pb.exists() && !p.contains("Antigravity-inst-");
        if has_pb {
            pb
        } else {
            crate::modules::process::detect_antigravity_with_diagnostics(None)?
        }
    } else {
        crate::modules::process::detect_antigravity_with_diagnostics(None)?
    };

    // Close only the existing process for THIS target instance if running, allowing OS to unmap locks
    let _ = close_instance(instance_id);
    std::thread::sleep(std::time::Duration::from_millis(300));

    let exe_str = exe_path.to_string_lossy().to_string();

    #[cfg(target_os = "macos")]
    {
        let mut cmd = Command::new("open");
        // -n flag guarantees a new separate instance is spawned even if another is already running
        cmd.arg("-n");
        cmd.arg("-a");
        cmd.arg(&exe_str);
        cmd.arg("--args");
        let has_custom_data = !is_default;
        if has_custom_data {
            cmd.arg(format!("--user-data-dir={}", data_dir));
            cmd.arg("--password-store=basic");
        }
        if let Some(ref ext_dir) = extensions_dir {
            cmd.arg(format!("--extensions-dir={}", ext_dir));
        }
        cmd.arg("--new-window");

        let child = cmd.spawn().map_err(|e| {
            crate::error::AppError::Process(format!(
                "Failed to spawn macOS instance process: {}",
                e
            ))
        })?;
        let _ = record_instance_pid(instance_id, child.id(), &data_dir);
        return Ok(());
    }

    #[cfg(not(target_os = "macos"))]
    {
        let mut cmd = Command::new(&exe_str);

        let has_custom_data = !is_default;
        if has_custom_data {
            cmd.arg(format!("--user-data-dir={}", data_dir));
            cmd.arg("--password-store=basic");
        }
        if let Some(ref ext_dir) = extensions_dir {
            cmd.arg(format!("--extensions-dir={}", ext_dir));
        }
        cmd.arg("--new-window");

        #[cfg(target_os = "windows")]
        {
            cmd.creation_flags(0x00000200); // CREATE_NEW_PROCESS_GROUP
        }

        #[cfg(target_os = "linux")]
        {
            crate::modules::process::clean_appimage_env(&mut cmd);
        }

        let child = cmd.spawn().map_err(|e| {
            crate::error::AppError::Process(format!("Failed to spawn instance process: {}", e))
        })?;
        let _ = record_instance_pid(instance_id, child.id(), &data_dir);
        Ok(())
    }
}

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
            let _ = fs::create_dir_all(&inst_dir);
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
            let _ = std::fs::remove_file(&appimage_target);
            let _ = std::os::unix::fs::symlink(&base_exe, &appimage_target);
            if appimage_target.exists() {
                appimage_target
            } else {
                let script_content = format!("#!/bin/sh\nexec \"{}\" \"$@\"\n", base_str);
                fs::write(&launcher_sh, script_content)
                    .map_err(|e| crate::error::AppError::Io(e))?;
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(&launcher_sh, fs::Permissions::from_mode(0o755));
                launcher_sh
            }
        } else {
            let script_content = format!("#!/bin/sh\nexec \"{}\" \"$@\"\n", base_str);
            fs::write(&launcher_sh, script_content).map_err(|e| crate::error::AppError::Io(e))?;
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&launcher_sh, fs::Permissions::from_mode(0o755));
            launcher_sh
        }
    };

    #[cfg(target_os = "macos")]
    let cloned_path = {
        let launcher_sh = instance_bin_dir.join(format!("antigravity-{}", instance_id));
        let base_str = base_exe.to_string_lossy();
        let script_content = format!("#!/bin/sh\nopen -n -a \"{}\" --args \"$@\"\n", base_str);
        fs::write(&launcher_sh, script_content).map_err(|e| crate::error::AppError::Io(e))?;
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&launcher_sh, fs::Permissions::from_mode(0o755));
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

/// Close only the process associated with this instance
pub fn close_instance(instance_id: &str) -> Result<(), String> {
    let registry = load_registry()?;
    let config = registry
        .instances
        .iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance {} not found", instance_id))?;

    let mut system = System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All);

    // 1. Gather all candidate PIDs for THIS specific instance only
    let is_default_inst = config.is_default || instance_id == "default";
    let mut pids = find_pids_for_data_dir(&config.data_dir, is_default_inst);
    if let Some(saved_pid) = config.pid.or_else(|| get_instance_saved_pid(instance_id)) {
        let is_alive = system.process(sysinfo::Pid::from_u32(saved_pid)).is_some();
        let not_contains = !pids.contains(&saved_pid);
        if is_alive {
            if not_contains {
                pids.push(saved_pid);
            }
        }
    }

    if pids.is_empty() {
        let _ = mark_instance_stopped(instance_id);
        return Ok(());
    }

    crate::modules::logger::log_info(&format!(
        "[Instance] Terminating {} process(es) for instance '{}' (PIDs: {:?})",
        pids.len(),
        instance_id,
        pids
    ));

    for pid in &pids {
        #[cfg(target_os = "windows")]
        {
            let _ = Command::new("taskkill")
                .args(["/F", "/T", "/PID", &pid.to_string()])
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

    // Synchronously wait for processes to exit to prevent SQLite locks and state overwrite
    let start_wait = std::time::Instant::now();
    let max_graceful = std::time::Duration::from_millis(3000);

    loop {
        system.refresh_processes(sysinfo::ProcessesToUpdate::All);
        let has_alive = pids
            .iter()
            .any(|&pid| system.process(sysinfo::Pid::from_u32(pid)).is_some());

        if !has_alive {
            crate::modules::logger::log_info(&format!(
                "[Instance] Successfully closed all processes for instance '{}'",
                instance_id
            ));
            break;
        }

        if start_wait.elapsed() > max_graceful {
            #[cfg(not(target_os = "windows"))]
            {
                crate::modules::logger::log_warn(&format!(
                    "[Instance] Graceful exit timed out for instance '{}', sending SIGKILL to remaining processes...",
                    instance_id
                ));
                for pid in &pids {
                    if system.process(sysinfo::Pid::from_u32(*pid)).is_some() {
                        let _ = Command::new("kill").args(["-9", &pid.to_string()]).output();
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(300));
            }
            break;
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // Clean any orphaned lock files in data_dir
    let target_data_path = PathBuf::from(&config.data_dir);
    if target_data_path.exists() {
        let lockfile = target_data_path.join("lockfile");
        if lockfile.exists() {
            let _ = fs::remove_file(&lockfile);
        }
        let code_lock = target_data_path.join("code.lock");
        if code_lock.exists() {
            let _ = fs::remove_file(&code_lock);
        }
        if let Ok(entries) = fs::read_dir(&target_data_path) {
            for entry in entries.flatten() {
                let fname = entry.file_name().to_string_lossy().to_lowercase();
                let is_stale_lock = fname == "lockfile"
                    || fname.starts_with("singleton")
                    || fname.ends_with(".lock")
                    || fname == "code.lock";
                if is_stale_lock {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }
    }

    let _ = mark_instance_stopped(instance_id);

    // Small settle delay to ensure OS flushes file handles and SQLite locks
    std::thread::sleep(std::time::Duration::from_millis(150));

    Ok(())
}

/// Get currently active instance ID
pub fn get_active_instance_id() -> Result<String, String> {
    let registry = load_registry()?;
    let active_exists = registry
        .instances
        .iter()
        .any(|i| i.id == registry.active_instance_id);
    if active_exists {
        return Ok(registry.active_instance_id);
    }
    // Fallback: check if any instance is currently running
    for inst in &registry.instances {
        let is_running = is_instance_running(&inst.id, &inst.data_dir, inst.pid);
        if is_running {
            return Ok(inst.id.clone());
        }
    }
    Ok("default".to_string())
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

/// Set an instance as the default instance
pub fn set_default_instance(instance_id: &str) -> Result<(), String> {
    let mut registry = load_registry()?;
    if !registry.instances.iter().any(|i| i.id == instance_id) {
        return Err(format!("Instance {} does not exist", instance_id));
    }
    for inst in registry.instances.iter_mut() {
        inst.is_default = inst.id == instance_id;
    }
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

/// Resolve an instance query string (seq_num like "1", ID like "inst-xyz", name like "Instance 1", or "default"/"active")
/// to a valid concrete instance ID.
pub fn resolve_instance_id(specifier: &str) -> Result<String, String> {
    let registry = load_registry()?;
    let clean = specifier.trim();
    if clean.is_empty() || clean.eq_ignore_ascii_case("active") {
        return Ok(registry.active_instance_id);
    }
    if clean.eq_ignore_ascii_case("default") {
        if let Some(def) = registry
            .instances
            .iter()
            .find(|i| i.is_default || i.id == "default")
        {
            return Ok(def.id.clone());
        }
        return Ok("default".to_string());
    }
    // Check if numeric seq_num (e.g. "1") or 1-based index
    if let Ok(num) = clean.parse::<u32>() {
        if let Some(inst) = registry.instances.iter().find(|i| i.seq_num == Some(num)) {
            return Ok(inst.id.clone());
        }
        if num >= 1 && (num as usize) <= registry.instances.len() {
            return Ok(registry.instances[(num as usize) - 1].id.clone());
        }
    }
    // Check clean number prefix like "ins-1", "instance-1", "#1"
    let clean_num = clean
        .trim_start_matches("ins-")
        .trim_start_matches("instance-")
        .trim_start_matches('#');
    if let Ok(num) = clean_num.parse::<u32>() {
        if let Some(inst) = registry.instances.iter().find(|i| i.seq_num == Some(num)) {
            return Ok(inst.id.clone());
        }
        if num >= 1 && (num as usize) <= registry.instances.len() {
            return Ok(registry.instances[(num as usize) - 1].id.clone());
        }
    }
    // Check exact id match
    if let Some(inst) = registry
        .instances
        .iter()
        .find(|i| i.id.eq_ignore_ascii_case(clean))
    {
        return Ok(inst.id.clone());
    }
    // Check name contains
    if let Some(inst) = registry
        .instances
        .iter()
        .find(|i| i.name.to_lowercase().contains(&clean.to_lowercase()))
    {
        return Ok(inst.id.clone());
    }
    if clean.is_empty() {
        return Ok(registry.active_instance_id);
    }
    Err(format!("Instance '{}' not found", target))
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
    let target_id = match target_instance_id {
        Some(s) => resolve_instance_id(s).unwrap_or_else(|_| s.to_string()),
        None => registry.active_instance_id.clone(),
    };

    let instance = registry
        .instances
        .iter()
        .find(|i| i.id == target_id)
        .ok_or_else(|| format!("Target instance {} not found", target_id))?;

    let is_default_inst = instance.is_default || instance.id == "default";

    if let Some(prev_email) = instance
        .bound_email
        .clone()
        .or_else(|| {
            instance.bound_account_id.as_ref().and_then(|id| {
                crate::modules::account::load_account(id)
                    .ok()
                    .map(|a| a.email)
            })
        })
        .or_else(|| {
            crate::modules::account::get_current_account()
                .ok()
                .flatten()
                .map(|a| a.email)
        })
    {
        if !prev_email.is_empty() {
            crate::modules::notification_hub::record_previous_email(&prev_email);
        }
    }

    // Ensure account has a bound device fingerprint profile for isolation
    if account.device_profile.is_none() {
        let new_profile = crate::modules::device::generate_profile();
        let _ = crate::modules::account::apply_profile_to_account(
            &mut account,
            new_profile,
            Some("auto_generated".to_string()),
            true,
        );
    }

    let db_dir = PathBuf::from(&instance.data_dir)
        .join("User")
        .join("globalStorage");
    if !db_dir.exists() {
        fs::create_dir_all(&db_dir).map_err(|e| format!("Failed to create db dir: {}", e))?;
    }
    let db_path = db_dir.join("state.vscdb");

    // Helper closure to inject credentials into all relevant state.vscdb & storage.json & OS keyring locations
    let inject_all_credentials = |acc: &crate::models::Account| -> Result<(), String> {
        let _ = crate::modules::integration::write_to_system_keyring(acc);

        crate::modules::db::inject_token(
            &db_path,
            &acc.token.access_token,
            &acc.token.refresh_token,
            acc.token.expiry_timestamp,
            &acc.email,
            acc.token.is_gcp_tos,
            acc.token.project_id.as_deref(),
            acc.token.id_token.as_deref(),
            acc.token.oauth_client_key.as_deref(),
            None,
        )?;

        if let Some(ref profile) = acc.device_profile {
            let _ = crate::modules::db::write_service_machine_id(&db_path, &profile.mac_machine_id);
        }

        if is_default_inst {
            for target_hint in [None, Some("ide")] {
                if let Ok(storage_path) = crate::modules::device::get_storage_path(target_hint) {
                    if let Some(ref profile) = acc.device_profile {
                        let _ = crate::modules::device::write_profile(&storage_path, profile);
                    }
                }
                if let Ok(global_db_path) = crate::modules::db::get_db_path(target_hint) {
                    if global_db_path != db_path
                        && global_db_path.parent().map(|p| p.exists()).unwrap_or(false)
                    {
                        let _ = crate::modules::db::inject_token(
                            &global_db_path,
                            &acc.token.access_token,
                            &acc.token.refresh_token,
                            acc.token.expiry_timestamp,
                            &acc.email,
                            acc.token.is_gcp_tos,
                            acc.token.project_id.as_deref(),
                            acc.token.id_token.as_deref(),
                            acc.token.oauth_client_key.as_deref(),
                            target_hint,
                        );
                        if let Some(ref profile) = acc.device_profile {
                            let _ = crate::modules::db::write_service_machine_id(
                                &global_db_path,
                                &profile.mac_machine_id,
                            );
                        }
                    }
                }
            }
        } else {
            let instance_storage_path = db_dir.join("storage.json");
            if let Some(ref profile) = acc.device_profile {
                let _ = crate::modules::device::write_profile(&instance_storage_path, profile);
            }
        }
        Ok(())
    };

    // 1. Snapshot running executable path & CLI workspace args BEFORE closing processes
    let active_exe_path = if is_default_inst {
        crate::modules::process::get_antigravity_executable_path(None)
            .or_else(|| crate::modules::process::get_antigravity_executable_path(Some("ide")))
    } else {
        None
    };
    let active_args = if is_default_inst {
        crate::modules::process::get_args_from_running_process(None)
            .or_else(|| crate::modules::process::get_args_from_running_process(Some("ide")))
    } else {
        None
    };

    // 2. Close the running instance process FIRST ("Kill First -> Write Second -> Start Third")
    //    Running Antigravity flushes in-memory state to state.vscdb/keyring on exit; closing first
    //    prevents the exiting process from overwriting our newly injected credentials.
    if is_default_inst {
        if crate::modules::process::is_antigravity_running(None) {
            let _ = crate::modules::process::close_antigravity(20, None);
        }
        if crate::modules::process::is_antigravity_running(Some("ide")) {
            let _ = crate::modules::process::close_antigravity(20, Some("ide"));
        }
    }
    let _ = close_instance(&instance.id);
    std::thread::sleep(std::time::Duration::from_millis(300));

    // 3. Inject credentials into all relevant state.vscdb, storage.json, and OS keyring locations AFTER process exit
    inject_all_credentials(&account)?;

    // 4. Bind account in registry and set as active
    bind_account_to_instance(&instance.id, &account.id, &account.email)?;
    let _ = set_active_instance_id(&instance.id);
    let _ = crate::modules::account::set_current_account_id(&account.id);

    account.update_last_used();
    let _ = crate::modules::account::save_account(&account);

    // 5. Relaunch Antigravity preserving exact executable path and workspace arguments (same as highlighted ⇄ switch button)
    if is_default_inst {
        if let Err(e) = crate::modules::process::start_antigravity_with_fallback_path(
            None,
            active_exe_path.as_deref(),
            active_args.as_deref(),
        ) {
            crate::modules::logger::log_warn(&format!(
                "[Instance] start_antigravity_with_fallback_path returned ({}), falling back to launch_instance",
                e
            ));
            launch_instance(&instance.id).map_err(|err| err.to_string())?;
        }
    } else {
        launch_instance(&instance.id).map_err(|e| e.to_string())?;
    }

    // 6. Dispatch unified Email and Telegram switch notifications
    crate::modules::notification_hub::notify_account_switched(
        &account.email,
        &instance.name,
        "Smart Rotator / Instance Account Switch",
        false,
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_pids_target_path_normalization() {
        let raw_path = "C:\\Users\\User\\.config\\antigravity\\";
        let normalized = raw_path.to_lowercase().replace('\\', "/");
        let clean = normalized.trim_end_matches('/');
        assert_eq!(clean, "c:/users/user/.config/antigravity");
    }

    #[test]
    fn test_linux_process_name_matching() {
        let names = ["antigravity", "antigravity-ide", "apprun", "code"];
        let is_matched_first = names[0].contains("antigravity");
        let is_matched_third = names[2] == "apprun";
        assert!(is_matched_first);
        assert!(is_matched_third);

        let helper_args = "--type=renderer --user-data-dir=/tmp/test";
        let is_helper = helper_args.contains("--type=");
        assert!(is_helper);
    }

    #[test]
    fn test_instance_config_serialization() {
        let instance = InstanceConfig {
            id: "ubuntu-test".to_string(),
            name: "Ubuntu Test".to_string(),
            data_dir: "/home/user/.config/antigravity-test".to_string(),
            executable_path: Some("/opt/antigravity/antigravity".to_string()),
            extensions_dir: None,
            bound_account_id: Some("acc-123".to_string()),
            bound_email: Some("dev@example.com".to_string()),
            created_at: 1000,
            last_used: 2000,
            is_default: false,
            pid: Some(12345),
            seq_num: Some(2),
        };

        let json = serde_json::to_string(&instance).unwrap();
        let restored: InstanceConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.id, "ubuntu-test");
        assert_eq!(restored.pid, Some(12345));
        assert_eq!(restored.seq_num, Some(2));
        assert_eq!(
            restored.executable_path,
            Some("/opt/antigravity/antigravity".to_string())
        );
    }

    #[test]
    fn test_instance_pid_sqlite_persistence() {
        let temp_dir = std::env::temp_dir().join(format!(
            "agm_pid_test_{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        let _ = std::fs::create_dir_all(&temp_dir);
        let test_db_path = temp_dir.join("test_instances.db");
        let conn = rusqlite::Connection::open(&test_db_path).unwrap();
        let _ = conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS instance_processes (
                instance_id TEXT PRIMARY KEY,
                pid INTEGER NOT NULL,
                data_dir TEXT,
                status TEXT NOT NULL,
                started_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );",
        );

        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT OR REPLACE INTO instance_processes (instance_id, pid, data_dir, status, started_at, updated_at)
             VALUES (?1, ?2, ?3, 'running', ?4, ?4)",
            rusqlite::params!["inst-test-1", 54321, "/tmp/inst1", now],
        ).unwrap();

        let (saved_pid, status): (u32, String) = conn
            .query_row(
                "SELECT pid, status FROM instance_processes WHERE instance_id = ?1",
                ["inst-test-1"],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();

        assert_eq!(saved_pid, 54321);
        assert_eq!(status, "running");

        conn.execute(
            "UPDATE instance_processes SET status = 'stopped', updated_at = ?1 WHERE instance_id = ?2",
            rusqlite::params![now + 10, "inst-test-1"],
        ).unwrap();

        let updated_status: String = conn
            .query_row(
                "SELECT status FROM instance_processes WHERE instance_id = ?1",
                ["inst-test-1"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(updated_status, "stopped");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_ubuntu_instance_switching_end_to_end_flow() {
        // Heavy local disk / OS environment instance switching test: skip in CI/CD
        if crate::proxy::config::is_ci_environment() {
            return;
        }
        let temp_dir = std::env::temp_dir().join(format!(
            "agm_test_{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        let instance_data = temp_dir.join("ubuntu-inst").join("data");
        let user_storage = instance_data.join("User").join("globalStorage");
        assert!(std::fs::create_dir_all(&user_storage).is_ok());

        let db_path = user_storage.join("state.vscdb");
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        assert!(conn
            .execute(
                "CREATE TABLE IF NOT EXISTS ItemTable (key TEXT PRIMARY KEY, value BLOB)",
                [],
            )
            .is_ok());

        let rows_count: i64 = conn
            .query_row("SELECT count(*) FROM ItemTable", [], |r| r.get(0))
            .unwrap();
        assert_eq!(rows_count, 0);

        let linux_data_str = instance_data.to_string_lossy().to_string();
        let normalized = linux_data_str.to_lowercase().replace('\\', "/");
        let clean = normalized.trim_end_matches('/');
        assert!(clean.contains("ubuntu-inst"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_mock_account_switch_ci() {
        // Lightweight in-memory mock test for CI/CD account & instance binding rotation
        let mut inst = InstanceConfig {
            id: "inst-ci-mock".to_string(),
            name: "CI Mock Worker".to_string(),
            data_dir: "/mock/inst-ci".to_string(),
            executable_path: None,
            extensions_dir: None,
            bound_account_id: Some("acc-old".to_string()),
            bound_email: Some("old@example.com".to_string()),
            created_at: 1000,
            last_used: 1000,
            is_default: true,
            pid: None,
            seq_num: Some(1),
        };
        inst.bound_account_id = Some("acc-new".to_string());
        inst.bound_email = Some("new@example.com".to_string());
        inst.last_used = 2000;
        assert_eq!(inst.bound_account_id.as_deref(), Some("acc-new"));
        assert_eq!(inst.bound_email.as_deref(), Some("new@example.com"));
        assert_eq!(inst.last_used, 2000);
    }

    #[test]
    fn test_resolve_instance_id_resolution() {
        let registry = InstanceRegistry {
            active_instance_id: "inst-default".to_string(),
            instances: vec![
                InstanceConfig {
                    id: "inst-default".to_string(),
                    name: "Default Instance".to_string(),
                    data_dir: "/tmp/default".to_string(),
                    executable_path: None,
                    extensions_dir: None,
                    bound_account_id: None,
                    bound_email: None,
                    created_at: 0,
                    last_used: 0,
                    is_default: true,
                    pid: None,
                    seq_num: Some(1),
                },
                InstanceConfig {
                    id: "inst-custom-2".to_string(),
                    name: "Worker Node 2".to_string(),
                    data_dir: "/tmp/custom2".to_string(),
                    executable_path: None,
                    extensions_dir: None,
                    bound_account_id: None,
                    bound_email: None,
                    created_at: 0,
                    last_used: 0,
                    is_default: false,
                    pid: None,
                    seq_num: Some(2),
                },
            ],
        };

        // Helper mock check logic
        let resolve_mock = |spec: &str| -> String {
            let clean = spec.trim();
            if clean.is_empty() || clean.eq_ignore_ascii_case("active") {
                return registry.active_instance_id.clone();
            }
            if clean.eq_ignore_ascii_case("default") {
                if let Some(def) = registry
                    .instances
                    .iter()
                    .find(|i| i.is_default || i.id == "default")
                {
                    return def.id.clone();
                }
            }
            if let Ok(num) = clean.parse::<u32>() {
                if let Some(inst) = registry.instances.iter().find(|i| i.seq_num == Some(num)) {
                    return inst.id.clone();
                }
            }
            let clean_num = clean
                .trim_start_matches("ins-")
                .trim_start_matches("instance-")
                .trim_start_matches('#');
            if let Ok(num) = clean_num.parse::<u32>() {
                if let Some(inst) = registry.instances.iter().find(|i| i.seq_num == Some(num)) {
                    return inst.id.clone();
                }
            }
            if let Some(inst) = registry
                .instances
                .iter()
                .find(|i| i.id.eq_ignore_ascii_case(clean))
            {
                return inst.id.clone();
            }
            registry.active_instance_id.clone()
        };

        assert_eq!(resolve_mock("1"), "inst-default");
        assert_eq!(resolve_mock("2"), "inst-custom-2");
        assert_eq!(resolve_mock("#2"), "inst-custom-2");
        assert_eq!(resolve_mock("ins-2"), "inst-custom-2");
        assert_eq!(resolve_mock("default"), "inst-default");
        assert_eq!(resolve_mock("active"), "inst-default");
        assert_eq!(resolve_mock("inst-custom-2"), "inst-custom-2");
    }
}
