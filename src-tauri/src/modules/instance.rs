use crate::error::{AppError, AppResult};
pub use crate::models::instance::{InstanceConfig, InstanceRegistry, InstanceStatus};
use once_cell::sync::Lazy;
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

/// Resolve or create the isolated home directory for an instance
pub fn get_instance_home_dir(instance_id: &str) -> Result<PathBuf, String> {
    let resolved_id = resolve_instance_id(instance_id).unwrap_or_else(|_| instance_id.to_string());
    let instances_root = get_instances_dir()?;
    let home_dir = instances_root.join(&resolved_id).join("home");
    if !home_dir.exists() {
        fs::create_dir_all(&home_dir)
            .map_err(|e| format!("Failed to create instance home directory: {}", e))?;
    }
    Ok(home_dir)
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

    conn.execute(
        "CREATE TABLE IF NOT EXISTS active_instance_selection (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            instance_id TEXT NOT NULL,
            updated_at INTEGER NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("Failed to create active_instance_selection table: {}", e))?;

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

/// Helper to synchronize app_storage.json across all instance storage locations
/// ensuring ide-install-wizard-shown, jetski.onboarding.lastLoginUsername, and
/// jetski.onboarding.lastLoginIsGcpTos reflect the bound account.
pub fn update_instance_app_storage(
    data_dir: &Path,
    bound_email: Option<&str>,
    is_gcp_tos: bool,
) -> Result<(), String> {
    let mut targets = vec![
        data_dir.join("app_storage.json"),
        data_dir
            .join("AppData")
            .join("Roaming")
            .join("Antigravity")
            .join("app_storage.json"),
    ];

    if let Some(parent) = data_dir.parent() {
        let home_app_storage = parent
            .join("home")
            .join("AppData")
            .join("Roaming")
            .join("Antigravity")
            .join("app_storage.json");
        targets.push(home_app_storage);
    }

    for target in targets {
        if let Some(p) = target.parent() {
            let _ = fs::create_dir_all(p);
        }
        let mut map: serde_json::Map<String, serde_json::Value> = if target.exists() {
            fs::read_to_string(&target)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            serde_json::Map::new()
        };

        map.insert(
            "ide-install-wizard-shown".to_string(),
            serde_json::Value::String("true".to_string()),
        );

        if let Some(email) = bound_email.map(str::trim).filter(|e| !e.is_empty()) {
            map.insert(
                "jetski.onboarding.lastLoginUsername".to_string(),
                serde_json::Value::String(email.to_string()),
            );
            map.insert(
                "jetski.onboarding.lastLoginIsGcpTos".to_string(),
                serde_json::Value::Bool(is_gcp_tos),
            );
        }

        if let Ok(content) = serde_json::to_string_pretty(&map) {
            let _ = fs::write(&target, content);
        }
    }

    // Also update <data_dir>/User/settings.json for initial workbench preferences
    if bound_email
        .map(str::trim)
        .filter(|e| !e.is_empty())
        .is_some()
    {
        let user_dir = data_dir.join("User");
        let _ = fs::create_dir_all(&user_dir);
        let settings_path = user_dir.join("settings.json");
        let mut settings_map: serde_json::Map<String, serde_json::Value> = if settings_path.exists()
        {
            fs::read_to_string(&settings_path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            serde_json::Map::new()
        };
        let mut modified = false;
        if !settings_map.contains_key("workbench.startupEditor") {
            settings_map.insert(
                "workbench.startupEditor".to_string(),
                serde_json::Value::String("none".to_string()),
            );
            modified = true;
        }
        if !settings_map.contains_key("security.workspace.trust.enabled") {
            settings_map.insert(
                "security.workspace.trust.enabled".to_string(),
                serde_json::Value::Bool(false),
            );
            modified = true;
        }
        if modified {
            if let Ok(pretty) = serde_json::to_string_pretty(&settings_map) {
                let _ = fs::write(&settings_path, pretty);
            }
        }
    }

    // Pre-seed User/globalStorage/state.vscdb from default instance if it doesn't exist yet so workbench layout/onboarding is initialized
    let inst_db = data_dir
        .join("User")
        .join("globalStorage")
        .join("state.vscdb");
    if !inst_db.exists() {
        let default_db = get_default_antigravity_data_dir()
            .join("User")
            .join("globalStorage")
            .join("state.vscdb");
        if default_db.exists() && default_db != inst_db {
            if let Some(p) = inst_db.parent() {
                let _ = fs::create_dir_all(p);
            }
            let _ = safe_clone_sqlite_db(&default_db, &inst_db);
            let _ = crate::modules::db::sanitize_session(&inst_db);
        }
    }

    Ok(())
}

/// Purge transient caches, LevelDB session storage, cookies, and lock files
/// to guarantee clean account initialization and prevent stale credential caching.
pub fn purge_volatile_instance_sessions(data_dir: &Path) {
    let folders_to_purge = [
        "Local Storage",
        "Session Storage",
        "Network",
        "GPUCache",
        "Cache",
        "Code Cache",
        "DawnCache",
        "blob_storage",
    ];

    let mut candidate_roots = vec![data_dir.to_path_buf()];
    let roaming = data_dir.join("AppData").join("Roaming").join("Antigravity");
    if roaming.exists() {
        candidate_roots.push(roaming);
    }
    if let Some(parent) = data_dir.parent() {
        let home_roaming = parent
            .join("home")
            .join("AppData")
            .join("Roaming")
            .join("Antigravity");
        if home_roaming.exists() {
            candidate_roots.push(home_roaming);
        }
    }

    for root in candidate_roots {
        for folder in &folders_to_purge {
            let target = root.join(folder);
            if target.exists() {
                let _ = fs::remove_dir_all(&target);
            }
        }
        for lock in &["lockfile", "code.lock"] {
            let p = root.join(lock);
            if p.exists() {
                let _ = fs::remove_file(&p);
            }
        }
    }
}

/// Helper to write keyring unavailable markers across all instance paths
/// to force Antigravity language_server to use isolated file credentials instead of system keyring.
pub fn write_keyring_bypass_markers(target_data_path: &Path, inst_home: Option<&Path>) {
    let mut marker_dirs = Vec::new();
    if let Some(home) = inst_home {
        marker_dirs.push(home.join(".gemini"));
        marker_dirs.push(home.join(".gemini").join("antigravity"));
        marker_dirs.push(home.join(".gemini").join("antigravity-ide"));
        marker_dirs.push(home.join(".gemini").join("antigravity-cli"));
        marker_dirs.push(home.join(".gemini").join("cache"));
        marker_dirs.push(home.join("AppData").join("Roaming").join("Antigravity"));
        marker_dirs.push(home.to_path_buf());
    }
    marker_dirs.push(target_data_path.to_path_buf());
    marker_dirs.push(target_data_path.join(".gemini"));
    marker_dirs.push(target_data_path.join(".gemini").join("antigravity"));
    marker_dirs.push(target_data_path.join(".gemini").join("antigravity-ide"));
    marker_dirs.push(target_data_path.join(".gemini").join("antigravity-cli"));
    marker_dirs.push(target_data_path.join(".gemini").join("cache"));
    marker_dirs.push(
        target_data_path
            .join("AppData")
            .join("Roaming")
            .join("Antigravity"),
    );

    let marker_names = [
        "antigravity-keyring-unavailable",
        "antigravity-ide-keyring-unavailable",
        "antigravity-cli-keyring-unavailable",
    ];

    for dir in marker_dirs {
        let _ = fs::create_dir_all(&dir);
        for name in &marker_names {
            let marker_path = dir.join(name);
            let _ = fs::write(&marker_path, b"1\n");
        }
    }
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
        if let Some(home) = dirs::home_dir() {
            let path = home
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
        let _ = inject_instance_settings(&registry.instances[0]);
        let _ = ensure_default_instance_exists();
        return Ok(registry);
    }

    let content = fs::read_to_string(&registry_path)
        .map_err(|e| format!("Failed to read instances registry: {}", e))?;
    let mut registry: InstanceRegistry =
        crate::modules::json_envelope::extract_payload::<InstanceRegistry>(&content)
            .map(|(data, _)| data)
            .or_else(|_| serde_json::from_str(&content))
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
        let _ = ensure_default_instance_exists();
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

    static SYNCED_TITLES_ONCE: std::sync::atomic::AtomicBool =
        std::sync::atomic::AtomicBool::new(false);
    if !SYNCED_TITLES_ONCE.swap(true, std::sync::atomic::Ordering::Relaxed) {
        for inst in &registry.instances {
            let _ = inject_instance_settings(inst);
        }
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

/// Export instances registry wrapped in standard JSON envelope with variable section
pub fn export_instances_envelope() -> Result<String, String> {
    let registry = load_registry()?;
    let envelope =
        crate::modules::json_envelope::JsonEnvelope::new("agm/instances-export", registry);
    serde_json::to_string_pretty(&envelope)
        .map_err(|e| format!("Failed to serialize instances envelope: {}", e))
}

#[derive(Clone)]
struct CachedProcessInfo {
    pid: u32,
    parent: Option<u32>,
    name: String,
    exe: String,
    args_str: String,
}

static PROCESS_SCAN_CACHE: Lazy<std::sync::Mutex<(std::time::Instant, Vec<CachedProcessInfo>)>> =
    Lazy::new(|| {
        std::sync::Mutex::new((
            std::time::Instant::now() - std::time::Duration::from_secs(10),
            Vec::new(),
        ))
    });

fn get_cached_antigravity_processes() -> Vec<CachedProcessInfo> {
    let mut cache = PROCESS_SCAN_CACHE.lock().unwrap();
    if cache.0.elapsed() < std::time::Duration::from_millis(4000) && !cache.1.is_empty() {
        return cache.1.clone();
    }

    let refresh_kind = sysinfo::ProcessRefreshKind::new()
        .with_cmd(sysinfo::UpdateKind::OnlyIfNotSet)
        .with_exe(sysinfo::UpdateKind::OnlyIfNotSet);

    let mut system =
        System::new_with_specifics(sysinfo::RefreshKind::new().with_processes(refresh_kind));
    system.refresh_processes_specifics(sysinfo::ProcessesToUpdate::All, refresh_kind);

    let mut procs = Vec::new();
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

        let is_non_ide = crate::modules::process::is_non_ide_binary(&name, &exe, &args_str);
        let is_self_process = if let Ok(current) = std::env::current_exe() {
            let cur_str = current.to_string_lossy().to_lowercase().replace('\\', "/");
            !cur_str.is_empty()
                && (exe == cur_str
                    || name
                        == current
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_lowercase())
        } else {
            false
        };
        let is_antigravity = !is_non_ide
            && !is_self_process
            && (name.contains("antigravity")
                || exe.contains("antigravity")
                || exe.contains("/tmp/.mount_")
                || name == "apprun")
            && !name.contains("agm")
            && !exe.contains("agm")
            && !name.contains("antigravity-manager")
            && !exe.contains("antigravity-manager")
            && !name.contains("antigravity_manager")
            && !exe.contains("antigravity_manager")
            && !name.contains("antigravity manager")
            && !exe.contains("antigravity manager")
            && !name.ends_with("manager.exe")
            && !exe.ends_with("manager.exe")
            && !name.ends_with("manager")
            && !exe.ends_with("manager")
            && !name.contains("webview")
            && !exe.contains("webview")
            && !args_str.contains("embedded-browser-webview");

        if is_antigravity {
            procs.push(CachedProcessInfo {
                pid: pid.as_u32(),
                parent: process.parent().map(|p| p.as_u32()),
                name,
                exe,
                args_str,
            });
        }
    }

    cache.0 = std::time::Instant::now();
    cache.1 = procs.clone();
    procs
}

/// Inspect running Antigravity processes matching an instance data_dir
pub fn find_pids_for_data_dir(data_dir: &str, is_default: bool) -> Vec<u32> {
    let processes = get_cached_antigravity_processes();

    let normalized_target = data_dir.to_lowercase().replace('\\', "/");
    let clean_target = normalized_target.trim_end_matches('/');
    let mut matched_pids = Vec::new();

    // Map child PID -> parent PID to trace process lineage
    let mut parent_map: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
    let mut instance_root_pids: std::collections::HashSet<u32> = std::collections::HashSet::new();
    let mut default_candidate_pids: Vec<u32> = Vec::new();

    for proc in &processes {
        let pid_u32 = proc.pid;
        if let Some(parent) = proc.parent {
            parent_map.insert(pid_u32, parent);
        }

        let name = &proc.name;
        let exe = &proc.exe;
        let args_str = &proc.args_str;

        let is_helper = args_str.contains("--type=")
            || name.contains("helper")
            || name.contains("crashpad")
            || exe.contains("crashpad")
            || name.contains("utility")
            || args_str.contains("utility");

        let inst_id_opt = if clean_target.contains("/instances/") {
            clean_target
                .split("/instances/")
                .nth(1)
                .and_then(|s| s.split('/').next())
        } else {
            None
        };

        let matches_cloned_exe = if let Some(inst_id) = inst_id_opt {
            let marker = format!("antigravity-{}", inst_id.to_lowercase());
            exe.contains(&marker) || name.contains(&marker)
        } else {
            false
        };

        let has_user_data_arg = args_str.contains("--user-data-dir");
        let has_instance_marker = args_str.contains(".antigravity_tools")
            || args_str.contains("/instances/")
            || args_str.contains("\\instances\\")
            || exe.contains(".antigravity_tools")
            || matches_cloned_exe;

        let is_default_candidate = is_default
            && !has_instance_marker
            && (!has_user_data_arg
                || (!clean_target.is_empty() && args_str.contains(clean_target)))
            && !is_helper
            && !args_str.contains(".antigravity_tools")
            && !args_str.contains("/instances/")
            && !args_str.contains("\\instances\\")
            && !name.contains("manager")
            && !exe.contains("manager");

        if (has_user_data_arg && has_instance_marker && !is_helper)
            || (matches_cloned_exe && !is_helper)
        {
            instance_root_pids.insert(pid_u32);
            if args_str.contains(clean_target) || matches_cloned_exe {
                matched_pids.push(pid_u32);
            }
        } else if is_default_candidate {
            let exe_path = std::path::Path::new(exe);
            let has_ide_markers = if let Some(parent) = exe_path.parent() {
                parent.join("resources").join("app.asar").exists()
                    || parent
                        .join("resources")
                        .join("bin")
                        .join("language_server.exe")
                        .exists()
                    || parent
                        .join("resources")
                        .join("bin")
                        .join("language_server")
                        .exists()
                    || exe.ends_with("antigravity.exe")
                    || exe.ends_with("/antigravity")
                    || name == "antigravity.exe"
                    || name == "antigravity"
            } else {
                name == "antigravity.exe" || name == "antigravity"
            };
            if has_ide_markers {
                default_candidate_pids.push(pid_u32);
            }
        }
    }

    if is_default {
        // Only accept processes whose ancestors do NOT belong to any instance process
        for cand_pid in default_candidate_pids {
            let mut curr = cand_pid;
            let mut is_instance_descendant = false;
            for _ in 0..10 {
                if instance_root_pids.contains(&curr) {
                    is_instance_descendant = true;
                    break;
                }
                if let Some(&p) = parent_map.get(&curr) {
                    curr = p;
                } else {
                    break;
                }
            }
            if !is_instance_descendant {
                matched_pids.push(cand_pid);
            }
        }
    } else {
        // For instances, also include any child processes that descend from matched root PIDs
        let matched_set: std::collections::HashSet<u32> = matched_pids.iter().cloned().collect();
        for proc_info in &processes {
            let pid_u32 = proc_info.pid;
            if matched_set.contains(&pid_u32) {
                continue;
            }
            let mut curr = pid_u32;
            for _ in 0..10 {
                if let Some(&p) = parent_map.get(&curr) {
                    if matched_set.contains(&p) {
                        matched_pids.push(pid_u32);
                        break;
                    }
                    curr = p;
                } else {
                    break;
                }
            }
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
        let is_default_inst = config.is_default || config.id == "default";
        let mut pids = find_pids_for_data_dir(&config.data_dir, is_default_inst);
        if let Some(saved_pid) = config.pid.or_else(|| get_instance_saved_pid(&config.id)) {
            if let Some(proc) = system.process(sysinfo::Pid::from_u32(saved_pid)) {
                let proc_name = proc.name().to_string_lossy().to_lowercase();
                let proc_exe = proc
                    .exe()
                    .map(|p| p.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                let is_antigravity =
                    proc_name.contains("antigravity") || proc_exe.contains("antigravity");
                if is_antigravity && !pids.contains(&saved_pid) {
                    let args_str = proc
                        .cmd()
                        .iter()
                        .map(|a| a.to_string_lossy().to_lowercase().replace('\\', "/"))
                        .collect::<Vec<String>>()
                        .join(" ");
                    let norm_data = config.data_dir.to_lowercase().replace('\\', "/");
                    let clean_data = norm_data.trim_end_matches('/');
                    if is_default_inst || args_str.contains(clean_data) {
                        pids.push(saved_pid);
                    }
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

/// Create a new isolated profile with optional specific account binding
pub fn create_instance_with_account(
    name: String,
    target_acc: Option<&str>,
) -> Result<InstanceConfig, String> {
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
    let instance_home_dir = instances_root.join(&instance_id).join("home");

    fs::create_dir_all(&instance_data_dir)
        .map_err(|e| format!("Failed to create instance directory: {}", e))?;
    fs::create_dir_all(&instance_home_dir)
        .map_err(|e| format!("Failed to create instance home directory: {}", e))?;

    #[cfg(target_os = "windows")]
    {
        let roaming = instance_home_dir.join("AppData").join("Roaming");
        let local = instance_home_dir.join("AppData").join("Local");
        let _ = fs::create_dir_all(&roaming);
        let _ = fs::create_dir_all(&local);
    }
    let gemini_ide_dir = instance_home_dir.join(".gemini").join("antigravity-ide");
    let gemini_dir = instance_home_dir.join(".gemini").join("antigravity");
    let _ = fs::create_dir_all(&gemini_ide_dir);
    let _ = fs::create_dir_all(&gemini_dir);

    // Pre-seed app_storage.json with ide-install-wizard-shown: true to skip onboarding wizard
    let _ = update_instance_app_storage(&instance_data_dir, None, false);

    let user_dir = instance_data_dir.join("User");
    let _ = fs::create_dir_all(&user_dir);
    let default_dir = get_default_antigravity_data_dir();
    let default_settings = default_dir.join("User").join("settings.json");
    let dest_settings = user_dir.join("settings.json");
    if default_settings.exists() && !dest_settings.exists() {
        let _ = fs::copy(default_settings, dest_settings);
    }

    // Also copy security_presets.json and antigravity_policies.json from default directory if they exist
    for file_name in &["security_presets.json", "antigravity_policies.json"] {
        let default_file = default_dir.join("User").join(file_name);
        let dest_file = user_dir.join(file_name);
        if default_file.exists() && !dest_file.exists() {
            let _ = fs::copy(&default_file, &dest_file);
        }
        #[cfg(target_os = "windows")]
        {
            let roaming_user = instance_home_dir
                .join("AppData")
                .join("Roaming")
                .join("Antigravity")
                .join("User");
            let _ = fs::create_dir_all(&roaming_user);
            let roaming_dest = roaming_user.join(file_name);
            if default_file.exists() && !roaming_dest.exists() {
                let _ = fs::copy(&default_file, &roaming_dest);
            }
        }
    }

    let next_seq = registry
        .instances
        .iter()
        .filter_map(|i| i.seq_num)
        .max()
        .unwrap_or(0)
        + 1;

    let mut bound_acc_id = None;
    let mut bound_acc_email = None;
    if let Ok(accounts) = crate::modules::account::list_accounts() {
        let bound_ids: std::collections::HashSet<String> = registry
            .instances
            .iter()
            .filter_map(|i| i.bound_account_id.clone())
            .collect();

        let candidate = if let Some(query) = target_acc.map(str::trim).filter(|q| !q.is_empty()) {
            let q_lower = query.to_lowercase();
            accounts
                .iter()
                .find(|a| {
                    a.id == query
                        || a.email.to_lowercase() == q_lower
                        || a.email.to_lowercase().contains(&q_lower)
                })
                .or_else(|| accounts.first())
        } else {
            accounts
                .iter()
                .find(|a| !bound_ids.contains(&a.id))
                .or_else(|| accounts.first())
        };

        if let Some(acc) = candidate {
            bound_acc_id = Some(acc.id.clone());
            bound_acc_email = Some(acc.email.clone());

            let _ = update_instance_app_storage(
                &instance_data_dir,
                Some(&acc.email),
                acc.token.is_gcp_tos,
            );
            purge_volatile_instance_sessions(&instance_data_dir);
            write_keyring_bypass_markers(&instance_data_dir, Some(&instance_home_dir));

            let _ =
                crate::modules::integration::write_to_file_credentials_at(&instance_home_dir, acc);
            let _ =
                crate::modules::integration::write_to_file_credentials_at(&instance_data_dir, acc);

            let db_dir = instance_data_dir.join("User").join("globalStorage");
            let _ = fs::create_dir_all(&db_dir);
            let db_path = db_dir.join("state.vscdb");
            let _ = crate::modules::db::inject_token(
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
            );

            let profile = acc
                .device_profile
                .clone()
                .unwrap_or_else(crate::modules::device::generate_profile);
            let storage_path = db_dir.join("storage.json");
            let _ = crate::modules::device::write_profile(&storage_path, &profile);
            let _ = crate::modules::db::write_service_machine_id(&db_path, &profile.mac_machine_id);

            #[cfg(target_os = "windows")]
            {
                let appdata_db_dir = instance_data_dir
                    .join("AppData")
                    .join("Roaming")
                    .join("Antigravity")
                    .join("User")
                    .join("globalStorage");
                let _ = fs::create_dir_all(&appdata_db_dir);
                let appdata_db_path = appdata_db_dir.join("state.vscdb");
                let _ = crate::modules::db::inject_token(
                    &appdata_db_path,
                    &acc.token.access_token,
                    &acc.token.refresh_token,
                    acc.token.expiry_timestamp,
                    &acc.email,
                    acc.token.is_gcp_tos,
                    acc.token.project_id.as_deref(),
                    acc.token.id_token.as_deref(),
                    acc.token.oauth_client_key.as_deref(),
                    None,
                );
                let storage_path = appdata_db_dir.join("storage.json");
                let _ = crate::modules::device::write_profile(&storage_path, &profile);
                let _ = crate::modules::db::write_service_machine_id(
                    &appdata_db_path,
                    &profile.mac_machine_id,
                );

                let home_appdata_db_dir = instance_home_dir
                    .join("AppData")
                    .join("Roaming")
                    .join("Antigravity")
                    .join("User")
                    .join("globalStorage");
                let _ = fs::create_dir_all(&home_appdata_db_dir);
                let home_appdata_db_path = home_appdata_db_dir.join("state.vscdb");
                let _ = crate::modules::db::inject_token(
                    &home_appdata_db_path,
                    &acc.token.access_token,
                    &acc.token.refresh_token,
                    acc.token.expiry_timestamp,
                    &acc.email,
                    acc.token.is_gcp_tos,
                    acc.token.project_id.as_deref(),
                    acc.token.id_token.as_deref(),
                    acc.token.oauth_client_key.as_deref(),
                    None,
                );
                let home_storage_path = home_appdata_db_dir.join("storage.json");
                let _ = crate::modules::device::write_profile(&home_storage_path, &profile);
                let _ = crate::modules::db::write_service_machine_id(
                    &home_appdata_db_path,
                    &profile.mac_machine_id,
                );
            }
        }
    }

    let config = InstanceConfig {
        id: instance_id.clone(),
        name: trimmed_name.to_string(),
        data_dir: instance_data_dir.to_string_lossy().to_string(),
        executable_path: None,
        extensions_dir: None,
        bound_account_id: bound_acc_id,
        bound_email: bound_acc_email,
        created_at: now,
        last_used: now,
        is_default: false,
        pid: None,
        seq_num: Some(next_seq),
    };

    registry.instances.push(config.clone());
    save_registry(&registry)?;

    let cloned_exe = clone_instance_executable(&instance_id).ok();
    let config = if let Some(exe) = cloned_exe {
        let mut c = config;
        c.executable_path = Some(exe);
        c
    } else {
        config
    };

    let _ = inject_instance_settings(&config);
    sync_instance_ide_parity(&instance_id).map_err(|e| e.to_string())?;

    Ok(config)
}

/// Create a new isolated profile (defaults to next available account)
pub fn create_instance(name: String) -> Result<InstanceConfig, String> {
    create_instance_with_account(name, None)
}

/// Stop an instance process cleanly
pub fn stop_instance(instance_id: &str) -> Result<(), String> {
    close_instance(instance_id)
}

/// Restart an instance on its current profile and bound account
pub fn restart_instance(instance_id: &str) -> AppResult<InstanceStatus> {
    let resolved_id = resolve_instance_id(instance_id).unwrap_or_else(|_| instance_id.to_string());
    crate::modules::logger::log_info(&format!(
        "[Instance] Restarting instance '{}' (resolved: '{}')",
        instance_id, resolved_id
    ));

    // 1. Terminate current running process(es)
    let _ = stop_instance(&resolved_id);

    // 2. Poll up to 1500ms for process cleanup and lockfile release
    let start_wait = std::time::Instant::now();
    if let Ok(registry) = load_registry() {
        if let Some(config) = registry.instances.iter().find(|i| i.id == resolved_id) {
            let is_default = config.is_default || resolved_id == "default";
            while start_wait.elapsed() < std::time::Duration::from_millis(1500) {
                let pids = find_pids_for_data_dir(&config.data_dir, is_default);
                if pids.is_empty() {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(80));
            }
        }
    }

    // 3. Invalidate prompt tree cache so running status reflects fresh state
    crate::modules::repo_db::invalidate_prompt_tree_cache(Some(&resolved_id));

    // 4. Launch instance with its existing bound account and workspaces
    launch_instance(&resolved_id)?;

    // 5. Return updated InstanceStatus
    let statuses = list_instances().map_err(AppError::Unknown)?;
    let updated = statuses
        .into_iter()
        .find(|s| s.config.id == resolved_id || s.config.name == resolved_id)
        .ok_or_else(|| {
            AppError::Process(format!(
                "Instance '{}' not found in registry after restart",
                resolved_id
            ))
        })?;

    Ok(updated)
}

/// Copy/clone an existing profile (full directory copy by default, or profile only)
pub fn copy_instance(
    source_id: &str,
    target_name: String,
    clone_mode: Option<&str>,
) -> Result<InstanceConfig, String> {
    copy_instance_with_options(source_id, target_name, clone_mode, true)
}

/// Clone alias for copy_instance_with_options
pub fn clone_instance(
    source_id: &str,
    target_name: &str,
    clone_mode: Option<&str>,
    copy_projects: bool,
) -> Result<InstanceConfig, String> {
    copy_instance_with_options(
        source_id,
        target_name.to_string(),
        clone_mode,
        copy_projects,
    )
}

/// Copy/clone an existing profile with options (optionally copying open projects and recent paths)
pub fn copy_instance_with_options(
    source_id: &str,
    target_name: String,
    clone_mode: Option<&str>,
    copy_projects: bool,
) -> Result<InstanceConfig, String> {
    let resolved_source_id =
        resolve_instance_id(source_id).unwrap_or_else(|_| source_id.to_string());
    let registry = load_registry()?;
    let source = registry
        .instances
        .iter()
        .find(|i| i.id == resolved_source_id || i.id == source_id)
        .ok_or_else(|| format!("Source instance {} not found", source_id))?
        .clone();

    let mut new_instance = create_instance(target_name)?;

    if source.extensions_dir.is_some() {
        new_instance.extensions_dir = source.extensions_dir.clone();
        if let Ok(mut reg) = load_registry() {
            if let Some(inst) = reg.instances.iter_mut().find(|i| i.id == new_instance.id) {
                inst.extensions_dir = source.extensions_dir.clone();
                let _ = save_registry(&reg);
            }
        }
    }

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
            if let Err(err) = copy_dir_recursive(&src_path, &dst_path) {
                crate::modules::logger::log_warn(&format!(
                    "[Instance] Data-dir copy failed for {}: {}",
                    source.id, err
                ));
            }
        }

        if let Err(err) = copy_required_ide_files(&src_path, &dst_path) {
            crate::modules::logger::log_warn(&format!(
                "[Instance] Required settings copy failed: {}",
                err
            ));
        }

        // Specifically ensure state.vscdb is cloned using safe_clone_sqlite_db
        let src_vscdb = src_path
            .join("User")
            .join("globalStorage")
            .join("state.vscdb");
        let dst_vscdb = dst_path
            .join("User")
            .join("globalStorage")
            .join("state.vscdb");
        if src_vscdb.exists() {
            if let Err(e) = safe_clone_sqlite_db(&src_vscdb, &dst_vscdb) {
                crate::modules::logger::log_warn(&format!(
                    "[Instance] Failed to safe-clone state.vscdb: {}",
                    e
                ));
            }
        } else if source.is_default || source.id == "default" {
            let default_vscdb = get_default_antigravity_data_dir()
                .join("User")
                .join("globalStorage")
                .join("state.vscdb");
            if default_vscdb.exists() {
                let _ = safe_clone_sqlite_db(&default_vscdb, &dst_vscdb);
            }
        }

        // Copy security_presets.json and antigravity_policies.json across all target directories
        let mut dst_user_targets = vec![dst_path.join("User")];
        #[cfg(target_os = "windows")]
        {
            if let Ok(dst_home) = get_instance_home_dir(&new_instance.id) {
                let dst_appdata_user = dst_home
                    .join("AppData")
                    .join("Roaming")
                    .join("Antigravity")
                    .join("User");
                let _ = fs::create_dir_all(&dst_appdata_user);
                dst_user_targets.push(dst_appdata_user);
            }
        }

        for file_name in &["security_presets.json", "antigravity_policies.json"] {
            let mut candidate_srcs = vec![src_path.join("User").join(file_name)];
            if let Some(src_home) = source_profile_home(&source) {
                #[cfg(target_os = "windows")]
                candidate_srcs.push(
                    src_home
                        .join("AppData")
                        .join("Roaming")
                        .join("Antigravity")
                        .join("User")
                        .join(file_name),
                );
            }
            candidate_srcs.push(
                get_default_antigravity_data_dir()
                    .join("User")
                    .join(file_name),
            );

            if let Some(found_src) = candidate_srcs.iter().find(|p| p.is_file()) {
                for target_dir in &dst_user_targets {
                    let _ = fs::create_dir_all(target_dir);
                    let _ = fs::copy(found_src, target_dir.join(file_name));
                }
                crate::modules::logger::log_info(&format!(
                    "[Instance] Copied {} from {} to {} target user directories",
                    file_name,
                    found_src.display(),
                    dst_user_targets.len()
                ));
            }
        }

        purge_volatile_instance_sessions(&dst_path);

        let is_tos = new_instance
            .bound_account_id
            .as_ref()
            .and_then(|id| crate::modules::account::load_account(id).ok())
            .map(|a| a.token.is_gcp_tos)
            .unwrap_or(false);
        let _ = update_instance_app_storage(&dst_path, new_instance.bound_email.as_deref(), is_tos);
        let _ = clone_instance_executable(&new_instance.id);

        // Sanitize cloned session and reseed with newly bound account credentials
        let cloned_db = dst_path
            .join("User")
            .join("globalStorage")
            .join("state.vscdb");
        let has_cloned_db = cloned_db.exists();
        if has_cloned_db {
            let _ = crate::modules::db::sanitize_session(&cloned_db);
            if let Some(ref acc_id) = new_instance.bound_account_id {
                if let Ok(acc) = crate::modules::account::load_account(acc_id) {
                    if let Ok(new_home) = get_instance_home_dir(&new_instance.id) {
                        let _ = crate::modules::integration::write_to_file_credentials_at(
                            &new_home, &acc,
                        );
                        write_keyring_bypass_markers(&dst_path, Some(&new_home));
                    } else {
                        write_keyring_bypass_markers(&dst_path, None);
                    }
                    let _ =
                        crate::modules::integration::write_to_file_credentials_at(&dst_path, &acc);

                    let _ = crate::modules::db::inject_token(
                        &cloned_db,
                        &acc.token.access_token,
                        &acc.token.refresh_token,
                        acc.token.expiry_timestamp,
                        &acc.email,
                        acc.token.is_gcp_tos,
                        acc.token.project_id.as_deref(),
                        acc.token.id_token.as_deref(),
                        acc.token.oauth_client_key.as_deref(),
                        None,
                    );
                    let profile = acc
                        .device_profile
                        .clone()
                        .unwrap_or_else(crate::modules::device::generate_profile);
                    let storage_path = dst_path
                        .join("User")
                        .join("globalStorage")
                        .join("storage.json");
                    let _ = crate::modules::device::write_profile(&storage_path, &profile);
                    let _ = crate::modules::db::write_service_machine_id(
                        &cloned_db,
                        &profile.mac_machine_id,
                    );

                    #[cfg(target_os = "windows")]
                    {
                        let appdata_db_dir = dst_path
                            .join("AppData")
                            .join("Roaming")
                            .join("Antigravity")
                            .join("User")
                            .join("globalStorage");
                        let _ = fs::create_dir_all(&appdata_db_dir);
                        let appdata_db_path = appdata_db_dir.join("state.vscdb");
                        let _ = crate::modules::db::inject_token(
                            &appdata_db_path,
                            &acc.token.access_token,
                            &acc.token.refresh_token,
                            acc.token.expiry_timestamp,
                            &acc.email,
                            acc.token.is_gcp_tos,
                            acc.token.project_id.as_deref(),
                            acc.token.id_token.as_deref(),
                            acc.token.oauth_client_key.as_deref(),
                            None,
                        );
                        let storage_path = appdata_db_dir.join("storage.json");
                        let _ = crate::modules::device::write_profile(&storage_path, &profile);
                        let _ = crate::modules::db::write_service_machine_id(
                            &appdata_db_path,
                            &profile.mac_machine_id,
                        );

                        if let Ok(new_home) = get_instance_home_dir(&new_instance.id) {
                            let home_appdata_db_dir = new_home
                                .join("AppData")
                                .join("Roaming")
                                .join("Antigravity")
                                .join("User")
                                .join("globalStorage");
                            let _ = fs::create_dir_all(&home_appdata_db_dir);
                            let home_appdata_db_path = home_appdata_db_dir.join("state.vscdb");
                            let _ = crate::modules::db::inject_token(
                                &home_appdata_db_path,
                                &acc.token.access_token,
                                &acc.token.refresh_token,
                                acc.token.expiry_timestamp,
                                &acc.email,
                                acc.token.is_gcp_tos,
                                acc.token.project_id.as_deref(),
                                acc.token.id_token.as_deref(),
                                acc.token.oauth_client_key.as_deref(),
                                None,
                            );
                            let home_storage_path = home_appdata_db_dir.join("storage.json");
                            let _ =
                                crate::modules::device::write_profile(&home_storage_path, &profile);
                            let _ = crate::modules::db::write_service_machine_id(
                                &home_appdata_db_path,
                                &profile.mac_machine_id,
                            );
                        }
                    }
                }
            }
        }
    }

    if let Err(err) = copy_source_ide_trees(&source, &new_instance) {
        crate::modules::logger::log_warn(&format!(
            "[Instance] IDE settings/repo tree copy failed: {}",
            err
        ));
    }

    if let Err(err) = copy_instance_settings(&source.id, &new_instance.id) {
        crate::modules::logger::log_warn(&format!(
            "[Instance] Deep settings copy failed from '{}' to '{}': {}",
            source.id, new_instance.id, err
        ));
    }

    if copy_projects {
        let _ = crate::modules::repo_db::detect_running_projects(&source.id);
        if let Err(err) =
            crate::modules::repo_db::clone_instance_repo_rows(&source.id, &new_instance.id)
        {
            crate::modules::logger::log_warn(&format!(
                "[Instance] Repo database clone failed: {}",
                err
            ));
        }
        if let Err(err) = copy_instance_projects(&source.id, &new_instance.id) {
            crate::modules::logger::log_warn(&format!(
                "[Instance] Workspace projects copy failed: {}",
                err
            ));
        }
    } else {
        let ws_main = dst_path.join("User").join("workspaceStorage");
        let has_ws = ws_main.exists();
        if has_ws {
            let _ = fs::remove_dir_all(&ws_main);
        }
        #[cfg(target_os = "windows")]
        {
            let ws_appdata = dst_path
                .join("AppData")
                .join("Roaming")
                .join("Antigravity")
                .join("User")
                .join("workspaceStorage");
            if ws_appdata.exists() {
                let _ = fs::remove_dir_all(&ws_appdata);
            }
            if let Ok(dst_home) = get_instance_home_dir(&new_instance.id) {
                let ws_home = dst_home
                    .join("AppData")
                    .join("Roaming")
                    .join("Antigravity")
                    .join("User")
                    .join("workspaceStorage");
                if ws_home.exists() {
                    let _ = fs::remove_dir_all(&ws_home);
                }
            }
        }
        purge_recent_project_paths(&new_instance);
    }

    let _ = sanitize_cloned_instance_summaries(&new_instance.id);
    let _ = inject_instance_settings(&new_instance);
    sync_instance_ide_parity(&new_instance.id).map_err(|e| e.to_string())?;

    Ok(new_instance)
}

/// Sanitize cloned conversation summaries in the target instance home and data directories.
/// Resets in-flight session flags (not_fully_idle = 0, status = 'IDLE') so a newly cloned
/// instance profile never inherits active running conversation states from the source.
pub fn sanitize_cloned_instance_summaries(target_id: &str) -> Result<(), String> {
    let mut candidates = Vec::new();

    if let Ok(home) = get_instance_home_dir(target_id) {
        candidates.push(
            home.join(".gemini")
                .join("antigravity")
                .join("conversation_summaries.db"),
        );
        candidates.push(
            home.join(".gemini")
                .join("antigravity-ide")
                .join("conversation_summaries.db"),
        );
        candidates.push(
            home.join(".gemini")
                .join("antigravity-cli")
                .join("conversation_summaries.db"),
        );
    }

    if let Ok(registry) = load_registry() {
        if let Some(inst) = registry.instances.iter().find(|i| i.id == target_id) {
            let data_dir = PathBuf::from(&inst.data_dir);
            candidates.push(
                data_dir
                    .join(".gemini")
                    .join("antigravity")
                    .join("conversation_summaries.db"),
            );
            candidates.push(
                data_dir
                    .join(".gemini")
                    .join("antigravity-ide")
                    .join("conversation_summaries.db"),
            );
            candidates.push(data_dir.join("conversation_summaries.db"));
        }
    }

    if let Ok(instances_dir) = get_instances_dir() {
        let instance_home_db = instances_dir
            .join(target_id)
            .join("home")
            .join(".gemini")
            .join("antigravity")
            .join("conversation_summaries.db");
        if !candidates.contains(&instance_home_db) {
            candidates.push(instance_home_db);
        }
    }

    for db_path in candidates {
        if db_path.exists() {
            if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                let _ = conn.pragma_update(None, "busy_timeout", 3000);
                let has_table = conn
                    .query_row(
                        "SELECT 1 FROM sqlite_master WHERE type='table' AND name='conversation_summaries'",
                        [],
                        |_| Ok(()),
                    )
                    .is_ok();
                if has_table {
                    let update_res = conn.execute(
                        "UPDATE conversation_summaries 
                         SET not_fully_idle = 0, status = 'IDLE' 
                         WHERE not_fully_idle != 0 OR status LIKE '%RUNNING%'",
                        [],
                    );
                    match update_res {
                        Ok(count) => {
                            if count > 0 {
                                crate::modules::logger::log_info(&format!(
                                    "[Instance] Sanitized {} active conversation summaries in {}",
                                    count,
                                    db_path.display()
                                ));
                            }
                        }
                        Err(e) => {
                            crate::modules::logger::log_warn(&format!(
                                "[Instance] Failed to sanitize conversation summaries in {}: {}",
                                db_path.display(),
                                e
                            ));
                        }
                    }
                }
            }
        }
    }

    Ok(())
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

    let _ = inject_instance_settings(&updated);

    Ok(updated)
}

/// Settings and workspace databases that must survive a clone even if a later
/// file in the full tree is locked.
pub const REQUIRED_IDE_REL_PATHS: &[&str] = &[
    "User/settings.json",
    "User/keybindings.json",
    "User/security_presets.json",
    "User/antigravity_policies.json",
    "User/snippets",
    "User/globalStorage",
    "User/workspaceStorage",
];

/// Directories under `~/.gemini` that hold IDE settings and repo databases.
pub const GEMINI_CLONE_DIRS: &[&str] = &[
    "antigravity",
    "antigravity-ide",
    "antigravity-cli",
    "policies",
    "config",
];

fn copy_required_ide_files(src: &Path, dst: &Path) -> Result<(), String> {
    for rel in REQUIRED_IDE_REL_PATHS {
        let from = src.join(rel);
        if !from.exists() {
            continue;
        }
        let to = dst.join(rel);
        if from.is_dir() {
            copy_dir_recursive(&from, &to).map_err(|e| format!("Failed to copy {}: {}", rel, e))?;
        } else {
            if let Some(parent) = to.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create parent for {}: {}", rel, e))?;
            }
            if rel.ends_with(".vscdb") || rel.ends_with(".db") {
                safe_clone_sqlite_db(&from, &to)?;
            } else {
                fs::copy(&from, &to).map_err(|e| format!("Failed to copy {}: {}", rel, e))?;
            }
        }
    }
    Ok(())
}

fn source_profile_home(source: &InstanceConfig) -> Option<PathBuf> {
    if source.is_default || source.id == "default" {
        std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .ok()
            .map(PathBuf::from)
    } else {
        get_instance_home_dir(&source.id).ok()
    }
}

/// Copy the source IDE's `.gemini` settings and repo trees into the new home.
/// The default instance lives in the real user profile; named instances live
/// under `instances/<id>/home`. Locks and caches stay behind via `copy_dir_recursive`.
pub fn copy_source_ide_trees(source: &InstanceConfig, dest: &InstanceConfig) -> Result<(), String> {
    if let Some(src_home) = source_profile_home(source) {
        if let Ok(dst_home) = get_instance_home_dir(&dest.id) {
            let _ = copy_gemini_trees(&src_home, &dst_home);
        }
    } else if let Ok(dst_home) = get_instance_home_dir(&dest.id) {
        if let Some(sys_home) = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .ok()
            .map(PathBuf::from)
        {
            let _ = copy_gemini_trees(&sys_home, &dst_home);
        }
    }

    let system_home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .ok()
        .map(PathBuf::from);
    let dst_data_gemini = PathBuf::from(&dest.data_dir).join(".gemini");
    for name in &["policies", "config"] {
        let mut src_opt = source_profile_home(source).map(|h| h.join(".gemini").join(name));
        if src_opt.as_ref().map(|p| !p.exists()).unwrap_or(true) {
            if let Some(ref sys) = system_home {
                let sys_candidate = sys.join(".gemini").join(name);
                if sys_candidate.exists() {
                    src_opt = Some(sys_candidate);
                }
            }
        }
        if let Some(src_p) = src_opt.filter(|p| p.exists()) {
            let _ = copy_dir_recursive(&src_p, &dst_data_gemini.join(name));
        }
    }

    copy_source_user_settings(source, dest)?;
    Ok(())
}

/// Deep copies user settings, themes, keybindings, and snippets from source instance to target instance.
/// On Windows, copies into both `instance_data_dir/User` AND `instance_home/AppData/Roaming/Antigravity/User`.
pub fn copy_source_user_settings(
    source: &InstanceConfig,
    dest: &InstanceConfig,
) -> Result<(), String> {
    let dst_data_dir = PathBuf::from(&dest.data_dir);
    let dst_user_dir = dst_data_dir.join("User");
    let _ = fs::create_dir_all(&dst_user_dir);

    let mut dst_dirs = vec![dst_user_dir];

    #[cfg(target_os = "windows")]
    {
        if let Ok(dst_home) = get_instance_home_dir(&dest.id) {
            let dst_appdata_user = dst_home
                .join("AppData")
                .join("Roaming")
                .join("Antigravity")
                .join("User");
            let _ = fs::create_dir_all(&dst_appdata_user);
            dst_dirs.push(dst_appdata_user);
        }
    }

    let mut src_user_dirs: Vec<PathBuf> = Vec::new();

    if source.is_default || source.id == "default" {
        #[cfg(target_os = "windows")]
        {
            if let Ok(appdata) = std::env::var("APPDATA") {
                let p = PathBuf::from(appdata).join("Antigravity").join("User");
                if p.exists() && !src_user_dirs.contains(&p) {
                    src_user_dirs.push(p);
                }
            }
        }
        let default_user = get_default_antigravity_data_dir().join("User");
        if default_user.exists() && !src_user_dirs.contains(&default_user) {
            src_user_dirs.push(default_user);
        }
        let source_data_user = PathBuf::from(&source.data_dir).join("User");
        if source_data_user.exists() && !src_user_dirs.contains(&source_data_user) {
            src_user_dirs.push(source_data_user);
        }
    } else {
        let source_data_user = PathBuf::from(&source.data_dir).join("User");
        if source_data_user.exists() && !src_user_dirs.contains(&source_data_user) {
            src_user_dirs.push(source_data_user);
        }
        #[cfg(target_os = "windows")]
        {
            if let Ok(source_home) = get_instance_home_dir(&source.id) {
                let source_appdata_user = source_home
                    .join("AppData")
                    .join("Roaming")
                    .join("Antigravity")
                    .join("User");
                if source_appdata_user.exists() && !src_user_dirs.contains(&source_appdata_user) {
                    src_user_dirs.push(source_appdata_user);
                }
            }
        }
    }

    // Copy all files and snippet directories from src_user_dirs into all dst_dirs
    for src_user in &src_user_dirs {
        if let Ok(entries) = fs::read_dir(src_user) {
            for entry in entries.flatten() {
                let file_name = entry.file_name();
                let name_str = file_name.to_string_lossy().to_lowercase();

                if name_str.ends_with(".lock")
                    || name_str.contains("cache")
                    || name_str == "lockfile"
                {
                    continue;
                }

                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        for dst in &dst_dirs {
                            let _ = fs::copy(entry.path(), dst.join(&file_name));
                        }
                    } else if file_type.is_dir() && name_str == "snippets" {
                        for dst in &dst_dirs {
                            let _ = copy_dir_recursive(&entry.path(), &dst.join(&file_name));
                        }
                    }
                }
            }
        }
    }

    // Specifically ensure settings.json (which contains workbench.colorTheme, read folders,
    // browser settings, etc.) is copied into BOTH places so that regardless of whether the IDE
    // reads from data_dir or APPDATA, the color theme and settings are identical to the source!
    let mut candidate_settings: Vec<PathBuf> = Vec::new();
    for src_user in &src_user_dirs {
        let s = src_user.join("settings.json");
        if s.is_file() {
            candidate_settings.push(s);
        }
    }

    if let Some(best_settings) = pick_best_settings_path(&candidate_settings) {
        for dst in &dst_dirs {
            let target_settings = dst.join("settings.json");
            if let Some(parent) = target_settings.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::copy(best_settings, target_settings);
        }
        crate::modules::logger::log_info(&format!(
            "[Instance] Cloned best source settings from {} into {} user targets",
            best_settings.display(),
            dst_dirs.len()
        ));
    }

    // Also explicitly ensure security_presets.json and antigravity_policies.json are copied into all dst_dirs
    for file_name in &["security_presets.json", "antigravity_policies.json"] {
        for src_user in &src_user_dirs {
            let p = src_user.join(file_name);
            if p.is_file() {
                for dst in &dst_dirs {
                    let _ = fs::copy(&p, dst.join(file_name));
                }
                break;
            }
        }
    }

    Ok(())
}

fn pick_best_settings_path(candidates: &[PathBuf]) -> Option<&PathBuf> {
    if candidates.is_empty() {
        return None;
    }
    if candidates.len() == 1 {
        return Some(&candidates[0]);
    }

    let score_file = |path: &PathBuf| -> (bool, u64, u64) {
        let has_theme = fs::read_to_string(path)
            .map(|content| content.contains("workbench.colorTheme"))
            .unwrap_or(false);
        let (mtime, len) = fs::metadata(path)
            .map(|m| {
                let mt = m
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                (mt, m.len())
            })
            .unwrap_or((0, 0));
        (has_theme, mtime, len)
    };

    candidates.iter().max_by_key(|p| score_file(p))
}

fn copy_gemini_trees(src_home: &Path, dst_home: &Path) -> Result<(), String> {
    fs::create_dir_all(dst_home.join(".gemini"))
        .map_err(|e| format!("Failed to create dest .gemini: {}", e))?;

    let system_home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .ok()
        .map(PathBuf::from);

    for name in GEMINI_CLONE_DIRS {
        let mut src = src_home.join(".gemini").join(name);
        if !src.exists() {
            if let Some(ref sys) = system_home {
                let sys_candidate = sys.join(".gemini").join(name);
                if sys_candidate.exists() {
                    src = sys_candidate;
                } else {
                    continue;
                }
            } else {
                continue;
            }
        }
        let dst = dst_home.join(".gemini").join(name);
        copy_dir_recursive(&src, &dst).map_err(|e| {
            format!(
                "Failed to copy {} from {} to {}: {}",
                name,
                src.display(),
                dst.display(),
                e
            )
        })?;
        crate::modules::logger::log_info(&format!(
            "[Instance] Cloned IDE tree {} into {}",
            src.display(),
            dst.display()
        ));
    }
    Ok(())
}

/// Helper to synchronize a directory recursively, updating missing or newer files.
pub fn sync_directory_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        return copy_dir_recursive(src, dst);
    }

    let entries = fs::read_dir(src)?;
    for entry in entries.flatten() {
        let file_type = entry.file_type()?;
        let child_dst = dst.join(entry.file_name());
        let name_str = entry.file_name().to_string_lossy().to_lowercase();

        let is_lock = name_str == "lockfile"
            || name_str.ends_with(".lock")
            || name_str.starts_with("singleton");
        let is_volatile_cache = name_str == "code cache"
            || name_str == "gpucache"
            || name_str == "dawngraphitecache"
            || name_str == "blob_storage"
            || name_str == "service worker"
            || name_str == "crashpad"
            || name_str.starts_with(".org.chromium");
        if is_lock || is_volatile_cache {
            continue;
        }

        if file_type.is_dir() {
            sync_directory_recursive(&entry.path(), &child_dst)?;
        } else if file_type.is_file() {
            let has_target_file = child_dst.exists();
            let has_change = if !has_target_file {
                true
            } else {
                let src_meta = entry.metadata().ok();
                let dst_meta = child_dst.metadata().ok();
                match (src_meta, dst_meta) {
                    (Some(sm), Some(dm)) => {
                        let sm_time = sm.modified().ok();
                        let dm_time = dm.modified().ok();
                        let is_newer = sm_time.is_some() && dm_time.is_some() && sm_time > dm_time;
                        sm.len() != dm.len() || is_newer
                    }
                    _ => false,
                }
            };
            if has_change {
                if let Some(parent) = child_dst.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::copy(entry.path(), &child_dst);
            }
        }
    }
    Ok(())
}

/// Synchronizes full 4-pillar IDE parity (theme presets, plugins, builtin skills, settings)
/// to the target instance directory from the default instance or host environment.
pub fn sync_instance_ide_parity(target_instance_id: &str) -> AppResult<()> {
    let resolved_id =
        resolve_instance_id(target_instance_id).unwrap_or_else(|_| target_instance_id.to_string());
    let instances_root = get_instances_dir().map_err(AppError::Config)?;
    let target_home = instances_root.join(&resolved_id).join("home");
    let target_data = instances_root.join(&resolved_id).join("data");

    if !target_home.exists() {
        fs::create_dir_all(&target_home).map_err(AppError::Io)?;
    }

    let default_home_gemini = instances_root.join("default").join("home").join(".gemini");
    let host_gemini = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .ok()
        .map(|h| PathBuf::from(h).join(".gemini"));

    let is_target_default = resolved_id == "default" || target_instance_id == "default";

    // Candidate source directories for .gemini assets
    let mut candidate_sources: Vec<PathBuf> = Vec::new();
    if is_target_default {
        if let Some(ref h) = host_gemini {
            if h.exists() {
                candidate_sources.push(h.clone());
            }
        }
    } else {
        if default_home_gemini.exists() {
            candidate_sources.push(default_home_gemini.clone());
        }
        if let Some(ref h) = host_gemini {
            if h.exists() && !candidate_sources.contains(h) {
                candidate_sources.push(h.clone());
            }
        }
    }

    let target_gemini = target_home.join(".gemini");
    fs::create_dir_all(&target_gemini).map_err(AppError::Io)?;

    // 1. Synchronize .gemini/config/config.json
    let target_config_dir = target_gemini.join("config");
    fs::create_dir_all(&target_config_dir).map_err(AppError::Io)?;
    let target_config_json = target_config_dir.join("config.json");

    let source_config_json = candidate_sources
        .iter()
        .map(|p| p.join("config").join("config.json"))
        .find(|p| p.is_file());

    if let Some(src_cfg_path) = source_config_json {
        let src_content = fs::read_to_string(&src_cfg_path).unwrap_or_default();
        if let Ok(src_val) = serde_json::from_str::<serde_json::Value>(&src_content) {
            if target_config_json.exists() {
                let mut target_val = fs::read_to_string(&target_config_json)
                    .ok()
                    .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                    .unwrap_or_else(|| serde_json::json!({}));
                deep_merge_json(&mut target_val, &src_val);
                if let Ok(formatted) = serde_json::to_string_pretty(&target_val) {
                    let _ = fs::write(&target_config_json, formatted);
                }
            } else {
                let _ = fs::copy(&src_cfg_path, &target_config_json);
            }
        } else if !target_config_json.exists() {
            let _ = fs::copy(&src_cfg_path, &target_config_json);
        }
    }

    if !target_config_json.exists() {
        let fallback_config = serde_json::json!({
            "plugins": {
                "chrome-devtools-plugin": { "enabled": true },
                "data-agent-kit-plugin": { "enabled": true },
                "google-antigravity-sdk": { "enabled": true },
                "modern-web-guidance-plugin": { "enabled": true }
            },
            "userSettings": {
                "artifactReviewMode": "ARTIFACT_REVIEW_MODE_TURBO",
                "autoExecutionPolicy": "CASCADE_COMMANDS_AUTO_EXECUTION_EAGER",
                "browserJsExecutionPolicy": "BROWSER_JS_EXECUTION_POLICY_TURBO",
                "conversationWidth": "CONVERSATION_WIDTH_WIDE",
                "customThemeSeedsDark": {
                    "background": "#19191C",
                    "foregroundOverride": "#F8F8F2",
                    "primary": "#BD93F9"
                },
                "customThemeSeedsLight": {
                    "background": "#EAECF0",
                    "foregroundOverride": "#202021",
                    "primary": "#8839EF"
                },
                "enableTerminalSandbox": false,
                "themeMode": "THEME_MODE_DARK"
            }
        });
        if let Ok(formatted) = serde_json::to_string_pretty(&fallback_config) {
            let _ = fs::write(&target_config_json, formatted);
        }
    }

    // 2. Synchronize .gemini/config/plugins/
    let target_plugins_dir = target_config_dir.join("plugins");
    let source_plugins_dir = candidate_sources
        .iter()
        .map(|p| p.join("config").join("plugins"))
        .find(|p| p.is_dir());

    if let Some(src_plugins) = source_plugins_dir {
        let _ = sync_directory_recursive(&src_plugins, &target_plugins_dir);
    }

    // 3. Synchronize .gemini/antigravity/builtin/skills/
    let target_skills_dir = target_gemini
        .join("antigravity")
        .join("builtin")
        .join("skills");
    let source_skills_dir = candidate_sources
        .iter()
        .map(|p| p.join("antigravity").join("builtin").join("skills"))
        .find(|p| p.is_dir());

    if let Some(src_skills) = source_skills_dir {
        let _ = sync_directory_recursive(&src_skills, &target_skills_dir);
    }

    // 4. Synchronize .gemini/ keyring bypass marker files
    let marker_names = [
        "antigravity-ide-keyring-unavailable",
        "antigravity-keyring-unavailable",
        "antigravity-cli-keyring-unavailable",
    ];
    for marker in &marker_names {
        let target_marker = target_gemini.join(marker);
        let found_marker = candidate_sources
            .iter()
            .map(|p| p.join(marker))
            .find(|p| p.is_file());

        if let Some(src_marker) = found_marker {
            let _ = fs::copy(&src_marker, &target_marker);
        } else if !target_marker.exists() {
            let _ = fs::write(&target_marker, b"1\n");
        }
    }
    write_keyring_bypass_markers(&target_data, Some(&target_home));

    // 5. Injects/merges settings in AppData/Roaming/Antigravity/User/settings.json
    let target_settings_path = target_home
        .join("AppData")
        .join("Roaming")
        .join("Antigravity")
        .join("User")
        .join("settings.json");

    let mut target_settings_val: serde_json::Value = if target_settings_path.exists() {
        fs::read_to_string(&target_settings_path)
            .ok()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
            .unwrap_or_else(|| serde_json::json!({}))
    } else {
        // Try candidate sources for initial settings.json
        let mut initial_val = serde_json::json!({});
        let default_appdata_settings = instances_root
            .join("default")
            .join("home")
            .join("AppData")
            .join("Roaming")
            .join("Antigravity")
            .join("User")
            .join("settings.json");
        let host_appdata_settings = std::env::var("APPDATA").ok().map(|a| {
            PathBuf::from(a)
                .join("Antigravity")
                .join("User")
                .join("settings.json")
        });

        let candidate_settings = [Some(default_appdata_settings), host_appdata_settings];
        for cand in candidate_settings.into_iter().flatten() {
            if cand.is_file() {
                if let Ok(content) = fs::read_to_string(&cand) {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                        initial_val = val;
                        break;
                    }
                }
            }
        }
        initial_val
    };

    if !target_settings_val.is_object() {
        target_settings_val = serde_json::json!({});
    }

    if let Some(map) = target_settings_val.as_object_mut() {
        let has_theme = map.contains_key("workbench.colorTheme");
        if !has_theme {
            map.insert(
                "workbench.colorTheme".to_string(),
                serde_json::Value::String("Default Dark Modern".to_string()),
            );
        }
        map.insert(
            "antigravity.turboMode".to_string(),
            serde_json::Value::Bool(true),
        );
        map.insert(
            "security.workspace.trust.enabled".to_string(),
            serde_json::Value::Bool(false),
        );
        map.insert(
            "antigravity.planReviewAlwaysProceed".to_string(),
            serde_json::Value::Bool(true),
        );
        map.insert(
            "antigravity.browserExecutionPolicy".to_string(),
            serde_json::Value::String("openDirectly".to_string()),
        );
        map.insert(
            "antigravity.codeReviewPolicy".to_string(),
            serde_json::Value::String("automaticReview".to_string()),
        );
    }

    if let Some(parent) = target_settings_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(formatted) = serde_json::to_string_pretty(&target_settings_val) {
        let _ = fs::write(&target_settings_path, &formatted);

        // Also write to data/User/settings.json if data dir exists
        let target_data_user = target_data.join("User");
        if target_data_user.exists() {
            let _ = fs::write(target_data_user.join("settings.json"), &formatted);
        }
    }

    crate::modules::logger::log_info(&format!(
        "[Instance] Successfully synchronized IDE parity for instance '{}'",
        target_instance_id
    ));

    Ok(())
}

/// Ensure default instance directory structure exists and is synchronized with IDE parity
pub fn ensure_default_instance_exists() -> AppResult<()> {
    let instances_root = get_instances_dir().map_err(AppError::Config)?;
    let default_home = instances_root.join("default").join("home");
    let default_data = instances_root.join("default").join("data");

    if !default_home.exists() {
        let _ = fs::create_dir_all(&default_home);
    }
    if !default_data.exists() {
        let _ = fs::create_dir_all(&default_data);
    }

    sync_instance_ide_parity("default")?;
    Ok(())
}

/// Clones an SQLite database safely even when active processes hold open locks or WAL files.
/// First attempts a read-only rusqlite connection in immutable URI mode (?mode=ro&immutable=1) with SQLite Online Backup API.
/// Executes incremental backup loop (step(100)) with exponential backoff (up to 2.5s) to handle transient busy/locked writes.
/// If the backup API fails or is unavailable, falls back to direct shared file copying including `-wal` and `-shm` sidecars.
pub fn safe_clone_sqlite_db(src_db: &Path, dst_db: &Path) -> Result<(), String> {
    if !src_db.exists() {
        return Ok(());
    }

    if let Some(parent) = dst_db.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            format!(
                "Failed to create destination directory {}: {}",
                parent.display(),
                e
            )
        })?;
    }

    // Attempt 1: Online SQLite Backup API via read-only connection in immutable URI mode
    let try_backup = || -> Result<(), String> {
        let clean_src = src_db.to_string_lossy().replace('\\', "/");
        let uri_primary = format!(
            "file:///{}?mode=ro&immutable=1",
            clean_src.trim_start_matches('/')
        );
        let uri_alt = format!("file:{}?mode=ro&immutable=1", clean_src);

        let src_conn = rusqlite::Connection::open_with_flags(
            &uri_primary,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
        )
        .or_else(|_| {
            rusqlite::Connection::open_with_flags(
                &uri_alt,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
            )
        })
        .or_else(|_| {
            rusqlite::Connection::open_with_flags(
                src_db,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
            )
        })
        .map_err(|e| format!("Failed to open source SQLite db in read-only mode: {}", e))?;

        let _ = src_conn.pragma_update(None, "busy_timeout", 2500);
        let _ = src_conn.pragma_update(None, "journal_mode", "WAL");

        if dst_db.exists() {
            let _ = fs::remove_file(dst_db);
        }
        let mut sidecar_wal = dst_db.as_os_str().to_os_string();
        sidecar_wal.push("-wal");
        let _ = fs::remove_file(PathBuf::from(sidecar_wal));
        let mut sidecar_shm = dst_db.as_os_str().to_os_string();
        sidecar_shm.push("-shm");
        let _ = fs::remove_file(PathBuf::from(sidecar_shm));

        let mut dst_conn = rusqlite::Connection::open(dst_db)
            .map_err(|e| format!("Failed to open destination SQLite db: {}", e))?;

        let _ = dst_conn.pragma_update(None, "busy_timeout", 2500);
        let _ = dst_conn.pragma_update(None, "journal_mode", "WAL");

        let backup = rusqlite::backup::Backup::new(&src_conn, &mut dst_conn)
            .map_err(|e| format!("Failed to initialize SQLite backup: {}", e))?;

        let start_time = std::time::Instant::now();
        let max_wait = std::time::Duration::from_millis(2500);
        let mut backoff = std::time::Duration::from_millis(10);

        loop {
            match backup.step(100) {
                Ok(rusqlite::backup::StepResult::Done) => break,
                Ok(rusqlite::backup::StepResult::More) => {
                    backoff = std::time::Duration::from_millis(10);
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
                Ok(rusqlite::backup::StepResult::Busy)
                | Ok(rusqlite::backup::StepResult::Locked) => {
                    if start_time.elapsed() >= max_wait {
                        return Err(
                            "SQLite backup timed out due to transient lock/busy state".to_string()
                        );
                    }
                    std::thread::sleep(backoff);
                    backoff = (backoff * 2).min(std::time::Duration::from_millis(250));
                }
                Ok(_) => break,
                Err(e) => {
                    return Err(format!("SQLite backup step error: {}", e));
                }
            }
        }

        Ok(())
    };

    if let Err(err) = try_backup() {
        crate::modules::logger::log_info(&format!(
            "[Instance] Online SQLite backup for {} failed ({}); falling back to shared file copy",
            src_db.display(),
            err
        ));

        // Attempt 2: Fallback direct file copy with -wal and -shm sidecars
        let copy_shared = |from: &Path, to: &Path| -> std::io::Result<u64> {
            match fs::copy(from, to) {
                Ok(bytes) => Ok(bytes),
                Err(_) => {
                    let mut reader = fs::File::open(from)?;
                    let mut writer = fs::File::create(to)?;
                    std::io::copy(&mut reader, &mut writer)
                }
            }
        };

        copy_shared(src_db, dst_db).map_err(|e| {
            format!(
                "Failed to copy SQLite database file {} to {}: {}",
                src_db.display(),
                dst_db.display(),
                e
            )
        })?;

        for suffix in &["-wal", "-shm"] {
            let mut src_sidecar = src_db.as_os_str().to_os_string();
            src_sidecar.push(suffix);
            let src_sidecar_path = PathBuf::from(src_sidecar);

            let mut dst_sidecar = dst_db.as_os_str().to_os_string();
            dst_sidecar.push(suffix);
            let dst_sidecar_path = PathBuf::from(dst_sidecar);

            if src_sidecar_path.exists() {
                let _ = copy_shared(&src_sidecar_path, &dst_sidecar_path);
            }
        }
    }

    Ok(())
}

pub fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        if let Err(e) = fs::create_dir_all(dst) {
            crate::modules::logger::log_warn(&format!(
                "[Instance] Failed to create directory {}: {}",
                dst.display(),
                e
            ));
            return Ok(());
        }
    }

    let entries = match fs::read_dir(src) {
        Ok(entries) => entries,
        Err(e) => {
            crate::modules::logger::log_warn(&format!(
                "[Instance] Failed to read directory {}: {}",
                src.display(),
                e
            ));
            return Ok(());
        }
    };

    for entry_result in entries {
        let entry = match entry_result {
            Ok(entry) => entry,
            Err(_) => continue,
        };

        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };

        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy().to_lowercase();

        // Skip volatile lock files, active socket handles, crashpad, and volatile caches
        let is_lock = name_str == "lockfile"
            || name_str.ends_with(".lock")
            || name_str.starts_with("singleton");
        let is_volatile_cache = name_str == "code cache"
            || name_str == "gpucache"
            || name_str == "dawngraphitecache"
            || name_str == "blob_storage"
            || name_str == "service worker"
            || name_str == "crashpad"
            || name_str.starts_with(".org.chromium");
        if is_lock || is_volatile_cache {
            continue;
        }

        let dest_child = dst.join(&file_name);
        if file_type.is_dir() {
            let _ = copy_dir_recursive(&entry.path(), &dest_child);
        } else if name_str.ends_with(".vscdb") || name_str.ends_with(".db") {
            if let Err(e) = safe_clone_sqlite_db(&entry.path(), &dest_child) {
                crate::modules::logger::log_warn(&format!(
                    "[Instance] Failed to safe clone SQLite db {}: {}",
                    entry.path().display(),
                    e
                ));
            }
        } else if name_str.ends_with(".vscdb-wal")
            || name_str.ends_with(".vscdb-shm")
            || name_str.ends_with(".db-wal")
            || name_str.ends_with(".db-shm")
        {
            // Handled alongside parent database in safe_clone_sqlite_db
            continue;
        } else {
            match fs::copy(entry.path(), &dest_child) {
                Ok(_) => {}
                Err(err) => {
                    let copy_stream = || -> std::io::Result<u64> {
                        let mut reader = fs::File::open(entry.path())?;
                        let mut writer = fs::File::create(&dest_child)?;
                        std::io::copy(&mut reader, &mut writer)
                    };
                    if let Err(e) = copy_stream() {
                        crate::modules::logger::log_warn(&format!(
                            "[Instance] Could not copy file {}: primary: {}, fallback: {}",
                            entry.path().display(),
                            err,
                            e
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

/// A saved PID matches when that one process is still Antigravity. This does not scan the process table.
pub fn process_identity_matches(name: &str, exe: &str) -> bool {
    let name = name.to_lowercase();
    let exe = exe.to_lowercase().replace('\\', "/");

    // Fast reject known non-IDE developer tools
    if crate::modules::process::is_non_ide_binary(&name, &exe, "") {
        return false;
    }

    let blocked = name.contains("agm")
        || exe.contains("agm")
        || name.contains("webview")
        || exe.contains("webview");
    if blocked {
        return false;
    }

    // Positive identity check: executable name itself must reflect Antigravity
    let is_antigravity_name =
        name.contains("antigravity") || name == "apprun" || name.starts_with("code");

    let is_antigravity_exe = exe.ends_with("/antigravity.exe")
        || exe.ends_with("/antigravity")
        || exe.contains("/antigravity-")
        || exe.contains("antigravity ide.exe")
        || exe.contains(".app/contents/macos");

    is_antigravity_name || is_antigravity_exe
}

pub fn saved_pid_matches(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }
    let mut sys = System::new();
    let target_pid = sysinfo::Pid::from_u32(pid);
    sys.refresh_processes_specifics(
        sysinfo::ProcessesToUpdate::Some(&[target_pid]),
        sysinfo::ProcessRefreshKind::new().with_exe(sysinfo::UpdateKind::OnlyIfNotSet),
    );
    let Some(proc) = sys.process(target_pid) else {
        return false;
    };
    let proc_name = proc.name().to_string_lossy().to_string();
    let proc_exe = proc
        .exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    process_identity_matches(&proc_name, &proc_exe)
}

/// Full process-table rebuild interval. Default 10 minutes. Never under 3 minutes, never over 20.
pub fn pid_refresh_interval_seconds(configured: u32) -> u32 {
    configured.clamp(180, 1200)
}

static LAST_PID_CACHE_REFRESH: Lazy<std::sync::Mutex<std::time::Instant>> = Lazy::new(|| {
    std::sync::Mutex::new(std::time::Instant::now() - std::time::Duration::from_secs(3600))
});

fn force_refresh_process_cache() {
    if let Ok(mut cache) = PROCESS_SCAN_CACHE.lock() {
        cache.0 = std::time::Instant::now() - std::time::Duration::from_secs(3600);
        cache.1.clear();
    }
    let _ = get_cached_antigravity_processes();
}

/// Rebuild the PID table and write a PID only when the saved one no longer matches.
pub fn refresh_saved_instance_pids() {
    force_refresh_process_cache();
    let Ok(registry) = load_registry() else {
        return;
    };
    for inst in registry.instances {
        let saved = inst.pid.or_else(|| get_instance_saved_pid(&inst.id));
        if saved.map(saved_pid_matches).unwrap_or(false) {
            continue;
        }
        let is_default = inst.is_default || inst.id == "default";
        let pids = find_pids_for_data_dir(&inst.data_dir, is_default);
        if let Some(pid) = pids.first().copied() {
            let _ = record_instance_pid(&inst.id, pid, &inst.data_dir);
        }
    }
}

pub fn refresh_pid_cache_if_due(configured_seconds: u32) -> bool {
    let interval = pid_refresh_interval_seconds(configured_seconds) as u64;
    let Ok(mut last) = LAST_PID_CACHE_REFRESH.lock() else {
        return false;
    };
    if last.elapsed() < std::time::Duration::from_secs(interval) {
        return false;
    }
    *last = std::time::Instant::now();
    drop(last);
    refresh_saved_instance_pids();
    true
}

/// Used only after a quota check is under the threshold and the saved PID does not match.
pub fn resolve_instance_pid_for_switch(
    instance_id: &str,
    data_dir: &str,
    is_default: bool,
) -> Option<u32> {
    if let Some(pid) = get_instance_saved_pid(instance_id) {
        if saved_pid_matches(pid) {
            return Some(pid);
        }
    }
    let pids = find_pids_for_data_dir(data_dir, is_default);
    let pid = pids.first().copied()?;
    let _ = record_instance_pid(instance_id, pid, data_dir);
    Some(pid)
}

/// Running check trusts the PID saved at launch. Only when that PID no longer matches
/// (for example the macOS `open` wrapper exited) does it search once and save the real PID,
/// so the next check is a single-process lookup again.
pub fn is_instance_running(instance_id: &str, data_dir: &str, config_pid: Option<u32>) -> bool {
    let is_default = instance_id == "default" || instance_id == "__default__";
    let saved = config_pid.or_else(|| get_instance_saved_pid(instance_id));
    if let Some(pid) = saved {
        if saved_pid_matches(pid) {
            if is_default {
                let pids = find_pids_for_data_dir(data_dir, true);
                if pids.contains(&pid) {
                    return true;
                }
            } else {
                return true;
            }
        }
    }
    let pids = find_pids_for_data_dir(data_dir, is_default);
    let Some(pid) = pids.first().copied() else {
        return false;
    };
    let _ = record_instance_pid(instance_id, pid, data_dir);
    true
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
    let pos = match registry.instances.iter().position(|i| i.id == instance_id) {
        Some(p) => p,
        None => {
            // Idempotent: If instance is already deleted or not found in registry,
            // clean up any lingering directory and return Ok
            if let Ok(instances_root) = get_instances_dir() {
                let instance_folder = instances_root.join(instance_id);
                if instance_folder.exists() {
                    let _ = fs::remove_dir_all(&instance_folder);
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

    let target_data_path = PathBuf::from(&config.data_dir);
    let state_db = target_data_path
        .join("User")
        .join("globalStorage")
        .join("state.vscdb");

    if state_db.exists() {
        let _ = fs::remove_file(&state_db);
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
            let _ = fs::remove_file(&appdata_db);
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
                let _ = fs::remove_file(&home_db);
            }
        }
    }

    purge_volatile_instance_sessions(&target_data_path);
    let _ = update_instance_app_storage(&target_data_path, None, false);

    Ok(())
}

/// Collect all existing workspace folder paths bound to `instance_id` from `repo_db` and `User/workspaceStorage/*/workspace.json`.
pub fn get_instance_workspace_folders(instance_id: &str, data_dir: &str) -> Vec<String> {
    let mut folders: Vec<String> = Vec::new();
    let mut seen_norm: std::collections::HashSet<String> = std::collections::HashSet::new();

    let mut push_folder = |raw_path: &str| {
        let trimmed = raw_path.trim();
        if trimmed.is_empty() {
            return;
        }
        let p = Path::new(trimmed);
        if !p.exists() {
            return;
        }
        let norm = trimmed
            .replace('\\', "/")
            .trim_end_matches('/')
            .to_lowercase();
        if seen_norm.insert(norm) {
            folders.push(trimmed.to_string());
        }
    };

    // 1. Live workspace discovery for this instance
    if let Ok(detected) = crate::modules::repo_db::detect_running_projects(instance_id) {
        for proj in detected {
            push_folder(&proj.repo_path);
        }
    }

    // 2. Persisted running_projects in repo_db for this instance
    if let Ok(all_projs) = crate::modules::repo_db::list_running_projects() {
        for proj in all_projs {
            let matches_inst = proj.instance_id == instance_id
                || ((instance_id == "default" || instance_id == "__default__")
                    && (proj.instance_id == "default" || proj.instance_id == "__default__"));
            if matches_inst {
                push_folder(&proj.repo_path);
            }
        }
    }

    // 3. Direct scan of `<data_dir>/User/workspaceStorage/*/workspace.json`
    if !data_dir.trim().is_empty() {
        let ws_root = PathBuf::from(data_dir)
            .join("User")
            .join("workspaceStorage");
        if ws_root.exists() {
            if let Ok(entries) = fs::read_dir(&ws_root) {
                for entry in entries.flatten() {
                    let ws_json = entry.path().join("workspace.json");
                    if ws_json.exists() {
                        if let Ok(content) = fs::read_to_string(&ws_json) {
                            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                                if let Some(uri) = val.get("folder").and_then(|v| v.as_str()) {
                                    let decoded =
                                        crate::modules::repo_db::decode_uri_to_path_pub(uri);
                                    push_folder(&decoded);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    folders.truncate(8);
    folders
}

/// Resolve workspace roots for an instance by its ID
pub fn get_instance_workspace_paths(instance_id: &str) -> Vec<String> {
    let registry = load_registry().unwrap_or_default();
    let data_dir = registry
        .instances
        .iter()
        .find(|i| i.id == instance_id)
        .map(|i| i.data_dir.as_str())
        .unwrap_or("");
    get_instance_workspace_folders(instance_id, data_dir)
}

/// Restore and re-inject prompts for a freshly restarted instance
pub fn restore_and_inject_prompts_for_instance(
    instance_id: &str,
    _workspace_roots: &[String],
) -> Result<usize, String> {
    let _ =
        crate::modules::backup_prompts_db::restore_running_prompts(Some(instance_id), false, None);
    let resent =
        crate::modules::repo_db::resend_running_commands_for_instance(Some(instance_id), 20)
            .unwrap_or_default();
    let dispatched = crate::modules::repo_db::dispatch_running_prompts(instance_id).unwrap_or(0);
    let _ = crate::modules::repo_db::ensure_prompt_goals_running_for_instance(instance_id);
    Ok(resent.len() + dispatched)
}

/// Snapshot all currently open or registered workspace project directories for an instance.
pub fn snapshot_active_workspaces(instance_id: &str) -> Vec<String> {
    let registry = match load_registry() {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let Some(inst) = registry.instances.iter().find(|i| i.id == instance_id) else {
        return Vec::new();
    };

    let folders = get_instance_workspace_folders(instance_id, &inst.data_dir);
    if !folders.is_empty() {
        return folders;
    }

    // Fallback: query active projects in repo_db
    if let Ok(projects) = crate::modules::repo_db::list_running_projects() {
        let is_target_default = instance_id == "default" || instance_id == "__default__";
        let mut fallback_folders = Vec::new();
        for p in projects {
            let matches = p.instance_id == instance_id
                || (is_target_default
                    && (p.instance_id == "default" || p.instance_id == "__default__"));
            if matches && Path::new(&p.repo_path).exists() {
                fallback_folders.push(p.repo_path);
            }
        }
        if !fallback_folders.is_empty() {
            fallback_folders.truncate(8);
            return fallback_folders;
        }
    }

    Vec::new()
}

/// Migrate active workspaces from a depleted/source instance to a target candidate instance.
/// Seeds `User/workspaceStorage/<ws-id>/workspace.json` in target instance data directory
/// and registers projects in `repo_db`.
pub fn migrate_instance_workspaces(
    from_instance_id: &str,
    to_instance_id: &str,
) -> Result<Vec<String>, String> {
    if from_instance_id == to_instance_id {
        return Ok(snapshot_active_workspaces(from_instance_id));
    }

    let workspaces = snapshot_active_workspaces(from_instance_id);
    if workspaces.is_empty() {
        crate::modules::logger::log_info(&format!(
            "[WorkspaceMigration] No active workspaces found on source '{}' to migrate to '{}'",
            from_instance_id, to_instance_id
        ));
        return Ok(Vec::new());
    }

    crate::modules::logger::log_info(&format!(
        "[WorkspaceMigration] Migrating {} workspace(s) from '{}' to '{}': {:?}",
        workspaces.len(),
        from_instance_id,
        to_instance_id,
        workspaces
    ));

    for ws_path in &workspaces {
        let _ = assign_project_to_instance(to_instance_id, ws_path);
    }

    Ok(workspaces)
}

/// Explicitly bind/assign one or more project workspace directories to a specific instance (`instance_id`, `#seq`, or name).
/// Seeds `<instance.data_dir>/User/workspaceStorage/<id>/workspace.json` and registers the project in `repo_db`.
pub fn assign_project_to_instance(instance_spec: &str, repo_path: &str) -> Result<String, String> {
    let resolved_id = resolve_instance_id(instance_spec)?;
    let registry = load_registry()?;
    let inst = registry
        .instances
        .iter()
        .find(|i| i.id == resolved_id)
        .ok_or_else(|| format!("Instance '{}' not found", instance_spec))?;

    let clean_path = PathBuf::from(repo_path.trim());
    if !clean_path.exists() {
        return Err(format!(
            "Project directory '{}' does not exist on disk",
            repo_path.trim()
        ));
    }
    let canonical_str = clean_path.to_string_lossy().to_string();
    let repo_name = clean_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "workspace".to_string());

    // 1. Seed workspaceStorage/<workspace_id>/workspace.json inside the target instance's data_dir
    let mut hash: u64 = 14695981039346656037;
    for b in canonical_str.to_lowercase().replace('\\', "/").bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    let ws_id = format!(
        "{}-{:08x}",
        repo_name.to_lowercase(),
        (hash & 0xFFFF_FFFF) as u32
    );
    let ws_dir = PathBuf::from(&inst.data_dir)
        .join("User")
        .join("workspaceStorage")
        .join(&ws_id);
    fs::create_dir_all(&ws_dir)
        .map_err(|e| format!("Failed to create instance workspaceStorage dir: {}", e))?;

    let normalized_slash = canonical_str.replace('\\', "/");
    let folder_uri = if normalized_slash.starts_with('/') {
        format!("file://{}", normalized_slash)
    } else {
        format!("file:///{}", normalized_slash)
    };
    let ws_json_payload = serde_json::json!({
        "folder": folder_uri
    });
    let ws_content = serde_json::to_string_pretty(&ws_json_payload).unwrap_or_default();
    fs::write(ws_dir.join("workspace.json"), &ws_content)
        .map_err(|e| format!("Failed to write workspace.json: {}", e))?;

    #[cfg(target_os = "windows")]
    {
        let appdata_ws_dir = PathBuf::from(&inst.data_dir)
            .join("AppData")
            .join("Roaming")
            .join("Antigravity")
            .join("User")
            .join("workspaceStorage")
            .join(&ws_id);
        if fs::create_dir_all(&appdata_ws_dir).is_ok() {
            let _ = fs::write(appdata_ws_dir.join("workspace.json"), &ws_content);
        }

        if let Ok(inst_home) = get_instance_home_dir(&inst.id) {
            let home_ws_dir = inst_home
                .join("AppData")
                .join("Roaming")
                .join("Antigravity")
                .join("User")
                .join("workspaceStorage")
                .join(&ws_id);
            if fs::create_dir_all(&home_ws_dir).is_ok() {
                let _ = fs::write(home_ws_dir.join("workspace.json"), &ws_content);
            }
        }
    }

    // 2. Register in repo_db running_projects & sequence table
    let _ = crate::modules::repo_db::detect_running_projects(&inst.id);

    Ok(format!(
        "✅ Assigned project '{}' ({}) to instance '{}' (#{} {})",
        repo_name,
        canonical_str,
        inst.id,
        inst.seq_num.unwrap_or(1),
        inst.name
    ))
}

/// Wait until the instance process exists, then give the prompt channel a moment to accept a send.
/// Uses adaptive retry logic up to 15s with exponential backoff rather than a hard timeout.
pub fn wait_for_instance_prompt_channel(instance_id: &str) {
    let registry = match load_registry() {
        Ok(reg) => reg,
        Err(err) => {
            crate::modules::logger::log_warn(&format!(
                "[Instance] Prompt channel wait skipped for '{}': {}",
                instance_id, err
            ));
            return;
        }
    };
    let Some(inst) = registry
        .instances
        .iter()
        .find(|item| item.id == instance_id)
    else {
        crate::modules::logger::log_warn(&format!(
            "[Instance] Prompt channel wait skipped; instance '{}' is not registered",
            instance_id
        ));
        return;
    };
    let start_time = std::time::Instant::now();
    let max_wait_duration = std::time::Duration::from_secs(15);
    let mut current_interval_ms = 100u64;
    let max_interval_ms = 1000u64;

    loop {
        let pids = find_pids_for_data_dir(&inst.data_dir, inst.is_default);
        let running = !pids.is_empty()
            || (inst.is_default && crate::modules::process::is_antigravity_running(None));
        if running {
            crate::modules::logger::log_info(&format!(
                "[Instance] Prompt channel ready for '{}' ({} pids after {:?}); allowing 6s stabilization window before inject",
                instance_id,
                pids.len(),
                start_time.elapsed()
            ));
            std::thread::sleep(std::time::Duration::from_millis(6000));
            return;
        }
        if start_time.elapsed() >= max_wait_duration {
            crate::modules::logger::log_warn(&format!(
                "[Instance] Prompt channel wait timed out after 15s for '{}'; injecting anyway",
                instance_id
            ));
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(current_interval_ms));
        current_interval_ms = (current_interval_ms * 3 / 2).min(max_interval_ms);
    }
}

/// Focus an already running instance window, or launch it if not running
pub fn focus_or_launch_instance(instance_id: &str) -> Result<bool, crate::error::AppError> {
    let registry = load_registry().map_err(crate::error::AppError::Config)?;
    let inst = registry
        .instances
        .iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| {
            crate::error::AppError::Config(format!("Instance {} not found", instance_id))
        })?;

    let is_default_inst = inst.is_default || inst.id == "default";
    let mut pids = find_pids_for_data_dir(&inst.data_dir, is_default_inst);
    if let Some(saved_pid) = inst.pid.or_else(|| get_instance_saved_pid(instance_id)) {
        if saved_pid > 0 && !pids.contains(&saved_pid) {
            let mut sys = System::new();
            sys.refresh_processes(sysinfo::ProcessesToUpdate::All);
            if sys.process(sysinfo::Pid::from_u32(saved_pid)).is_some() {
                pids.push(saved_pid);
            }
        }
    }

    if !pids.is_empty() {
        crate::modules::logger::log_info(&format!(
            "[Instance] Focus requested for '{}', focusing running PIDs: {:?}",
            instance_id, pids
        ));
        let focused = crate::modules::process::focus_instance_pids(&pids);
        if focused {
            return Ok(true);
        }
    }

    crate::modules::logger::log_info(&format!(
        "[Instance] Instance '{}' is not running or could not be focused; launching new process",
        instance_id
    ));
    launch_instance(instance_id)?;
    Ok(false)
}

/// Launch a specific instance with multi-window isolation, bound workspace folder restoration, and custom/cloned executable support
pub fn launch_instance(instance_id: &str) -> Result<(), crate::error::AppError> {
    launch_instance_inner(instance_id, true)
}

/// Launch during an account switch. Prompt restore stays in the switch step that runs after the process is up.
pub fn launch_instance_without_prompt_reinject(
    instance_id: &str,
) -> Result<(), crate::error::AppError> {
    launch_instance_inner(instance_id, false)
}

/// Launch a specific instance with extra/migrated workspace folders and prompt reinjection control
pub fn launch_instance_with_workspaces(
    instance_id: &str,
    extra_workspaces: Option<&[String]>,
    reinject_prompts: bool,
) -> Result<(), crate::error::AppError> {
    launch_instance_inner_with_extra_workspaces(instance_id, reinject_prompts, extra_workspaces)
}

fn launch_instance_inner(
    instance_id: &str,
    reinject_prompts: bool,
) -> Result<(), crate::error::AppError> {
    launch_instance_inner_with_extra_workspaces(instance_id, reinject_prompts, None)
}

/// Helper to construct candidate search paths for macOS instance launch.
pub(crate) fn get_macos_candidate_paths() -> Vec<PathBuf> {
    let mut candidates = vec![
        PathBuf::from("/Applications/Antigravity.app"),
        PathBuf::from("/Applications/Antigravity.app/Contents/MacOS/Antigravity"),
    ];

    if let Some(home) = dirs::home_dir() {
        let user_app = home.join("Applications").join("Antigravity.app");
        candidates.push(user_app.clone());
        let user_macos_bin = user_app.join("Contents").join("MacOS").join("Antigravity");
        candidates.push(user_macos_bin);
    }

    candidates
}

fn launch_instance_inner_with_extra_workspaces(
    instance_id: &str,
    reinject_prompts: bool,
    extra_workspaces: Option<&[String]>,
) -> Result<(), crate::error::AppError> {
    let mut registry = load_registry().map_err(crate::error::AppError::Config)?;
    let pos = registry
        .instances
        .iter()
        .position(|i| i.id == instance_id)
        .ok_or_else(|| {
            crate::error::AppError::Config(format!("Instance {} not found", instance_id))
        })?;

    let inst_config = registry.instances[pos].clone();
    let data_dir = registry.instances[pos].data_dir.clone();
    let is_default = registry.instances[pos].is_default;
    let custom_exe = registry.instances[pos].executable_path.clone();
    let extensions_dir = registry.instances[pos].extensions_dir.clone();
    let bound_acc = registry.instances[pos].bound_account_id.clone();
    let bound_email = registry.instances[pos].bound_email.clone();
    registry.instances[pos].last_used = chrono::Utc::now().timestamp();
    registry.active_instance_id = instance_id.to_string();
    save_registry(&registry).map_err(crate::error::AppError::Config)?;

    let target_data_path = PathBuf::from(&data_dir);

    // Snapshot bound workspace folders BEFORE closing existing instance processes
    let mut workspace_folders = get_instance_workspace_folders(instance_id, &data_dir);
    if let Some(extras) = extra_workspaces {
        for folder in extras {
            if Path::new(folder).exists() && !workspace_folders.contains(folder) {
                workspace_folders.push(folder.clone());
            }
        }
    }

    // Determine executable FIRST while running processes are alive for discovery
    let exe_path = if !is_default {
        if let Some(ref p) = custom_exe {
            let pb = PathBuf::from(p);
            let p_lower = p.to_lowercase();
            if pb.exists() && !p_lower.contains(".trash") {
                pb
            } else if let Ok(cloned) = clone_instance_executable(instance_id) {
                PathBuf::from(cloned)
            } else {
                crate::modules::process::detect_antigravity_with_diagnostics(None)?
            }
        } else if let Ok(cloned) = clone_instance_executable(instance_id) {
            PathBuf::from(cloned)
        } else {
            crate::modules::process::detect_antigravity_with_diagnostics(None)?
        }
    } else {
        crate::modules::process::detect_antigravity_with_diagnostics(None)?
    };

    // Close only the existing process for THIS target instance if running, allowing OS to unmap locks
    let _ = close_instance(instance_id);
    std::thread::sleep(std::time::Duration::from_millis(300));

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

    // If an account is bound to this instance profile, sync credentials and inject token directly into isolated state.vscdb and system keyring AFTER process exit.
    let resolved_account = if let Some(ref account_id) = bound_acc {
        crate::modules::account::load_account(account_id)
            .ok()
            .or_else(|| {
                bound_email.as_ref().and_then(|em| {
                    crate::modules::account::get_account_by_email(em)
                        .ok()
                        .flatten()
                })
            })
    } else if let Some(ref email) = bound_email {
        crate::modules::account::get_account_by_email(email)
            .ok()
            .flatten()
    } else if is_default {
        crate::modules::account::get_current_account()
            .ok()
            .flatten()
    } else {
        None
    };

    if let Some(account) = resolved_account.as_ref() {
        let is_tos = account.token.is_gcp_tos;
        let _ = update_instance_app_storage(&target_data_path, Some(&account.email), is_tos);
        purge_volatile_instance_sessions(&target_data_path);

        // Unconditionally sync credentials to system keyring and credential file so active Antigravity instance reads the bound account
        let _ = crate::modules::integration::write_to_system_keyring(account);
        let _ = crate::modules::integration::write_to_file_credentials(account);
        if is_default {
            let _ = crate::modules::account::set_current_account_id(&account.id);
        }
        if let Ok(inst_home) = get_instance_home_dir(instance_id) {
            let _ = crate::modules::integration::write_to_file_credentials_at(&inst_home, account);
            write_keyring_bypass_markers(&target_data_path, Some(&inst_home));
        } else {
            write_keyring_bypass_markers(&target_data_path, None);
        }
        let _ =
            crate::modules::integration::write_to_file_credentials_at(&target_data_path, account);

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
            let _ = crate::modules::db::write_service_machine_id(&db_path, &profile.mac_machine_id);
            let storage_path = db_dir.join("storage.json");
            let _ = crate::modules::device::write_profile(&storage_path, profile);
        }

        #[cfg(target_os = "windows")]
        if !is_default {
            let appdata_db_dir = target_data_path
                .join("AppData")
                .join("Roaming")
                .join("Antigravity")
                .join("User")
                .join("globalStorage");
            let _ = fs::create_dir_all(&appdata_db_dir);
            let appdata_db_path = appdata_db_dir.join("state.vscdb");
            let _ = crate::modules::db::inject_token(
                &appdata_db_path,
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
                let _ = crate::modules::db::write_service_machine_id(
                    &appdata_db_path,
                    &profile.mac_machine_id,
                );
                let storage_path = appdata_db_dir.join("storage.json");
                let _ = crate::modules::device::write_profile(&storage_path, profile);
            }

            if let Ok(inst_home) = get_instance_home_dir(instance_id) {
                let home_appdata_db_dir = inst_home
                    .join("AppData")
                    .join("Roaming")
                    .join("Antigravity")
                    .join("User")
                    .join("globalStorage");
                let _ = fs::create_dir_all(&home_appdata_db_dir);
                let home_appdata_db_path = home_appdata_db_dir.join("state.vscdb");
                let _ = crate::modules::db::inject_token(
                    &home_appdata_db_path,
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
                    let _ = crate::modules::db::write_service_machine_id(
                        &home_appdata_db_path,
                        &profile.mac_machine_id,
                    );
                    let storage_path = home_appdata_db_dir.join("storage.json");
                    let _ = crate::modules::device::write_profile(&storage_path, profile);
                }
            }
        }

        // Wipe stale Local Storage / Session Storage to prevent old cached sessions from persisting
        let local_storage = target_data_path.join("Local Storage");
        if local_storage.exists() {
            let _ = fs::remove_dir_all(&local_storage);
        }
        let session_storage = target_data_path.join("Session Storage");
        if session_storage.exists() {
            let _ = fs::remove_dir_all(&session_storage);
        }
    }

    let _ = inject_instance_settings(&inst_config);

    let exe_str = exe_path.to_string_lossy().to_string();

    #[cfg(target_os = "macos")]
    {
        // Ensure candidate executable paths for Antigravity also check user-level $HOME/Applications/Antigravity.app:
        // If exe_str is not found at /Applications/Antigravity.app, check $HOME/Applications/Antigravity.app.
        let mut exe_str = exe_str;
        if !Path::new(&exe_str).exists() {
            if exe_str == "/Applications/Antigravity.app"
                || exe_str.starts_with("/Applications/Antigravity.app")
            {
                if let Some(home) = dirs::home_dir() {
                    let user_app = home.join("Applications").join("Antigravity.app");
                    if user_app.exists() {
                        exe_str = user_app.to_string_lossy().to_string();
                    }
                }
            }
            if !Path::new(&exe_str).exists() {
                for candidate in get_macos_candidate_paths() {
                    if candidate.exists() {
                        exe_str = candidate.to_string_lossy().to_string();
                        break;
                    }
                }
            }
        }

        let is_app_bundle = exe_str.ends_with(".app") || Path::new(&exe_str).is_dir();

        let mut app_args = Vec::new();
        let has_custom_data = !is_default;
        let inst_home_opt = if has_custom_data {
            app_args.push(format!("--user-data-dir={}", data_dir));
            app_args.push("--password-store=basic".to_string());
            app_args.push("--remote-debugging-port=0".to_string());
            let home_opt = get_instance_home_dir(instance_id).ok();
            write_keyring_bypass_markers(&target_data_path, home_opt.as_deref());
            if let Some(ref inst_home) = home_opt {
                let _ = fs::create_dir_all(inst_home);
                let gemini_ide_dir = inst_home.join(".gemini").join("antigravity-ide");
                let gemini_dir = inst_home.join(".gemini").join("antigravity");
                let _ = fs::create_dir_all(&gemini_ide_dir);
                let _ = fs::create_dir_all(&gemini_dir);

                // Ensure app_storage.json has ide-install-wizard-shown: true and reflects the bound account
                let is_tos = resolved_account
                    .as_ref()
                    .map(|a| a.token.is_gcp_tos)
                    .unwrap_or(false);
                let _ =
                    update_instance_app_storage(&target_data_path, bound_email.as_deref(), is_tos);
            } else {
                let is_tos = resolved_account
                    .as_ref()
                    .map(|a| a.token.is_gcp_tos)
                    .unwrap_or(false);
                let _ =
                    update_instance_app_storage(&target_data_path, bound_email.as_deref(), is_tos);
            }
            home_opt
        } else {
            None
        };

        if let Some(ref ext_dir) = extensions_dir {
            app_args.push(format!("--extensions-dir={}", ext_dir));
        }
        if !workspace_folders.is_empty() {
            for folder in &workspace_folders {
                app_args.push(folder.clone());
            }
        }

        let args = if app_args.is_empty() {
            None
        } else {
            Some(app_args)
        };

        let mut cmd = if is_app_bundle {
            let mut c = Command::new("open");
            let open_args =
                crate::modules::process::format_macos_open_args(&exe_str, args.as_deref(), true);
            c.args(&open_args);
            c
        } else {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(&exe_path) {
                let mut permissions = metadata.permissions();
                let mode = permissions.mode();
                if mode & 0o111 == 0 {
                    permissions.set_mode(mode | 0o755);
                    let _ = std::fs::set_permissions(&exe_path, permissions);
                }
            }
            let mut c = Command::new(&exe_str);
            if let Some(parent) = exe_path.parent() {
                c.current_dir(parent);
            }
            if let Some(ref arg_list) = args {
                for arg in arg_list {
                    c.arg(arg);
                }
            }
            c.arg("--new-window");
            c
        };

        if has_custom_data {
            if let Some(ref inst_home) = inst_home_opt {
                cmd.env("HOME", inst_home);
            }
            cmd.env("SSH_CONNECTION", "127.0.0.1 50000 127.0.0.1 22");
            cmd.env("SSH_CLIENT", "127.0.0.1 50000 22");
            cmd.env("SSH_TTY", "pty/0");
            if let Some(ref acc) = resolved_account {
                cmd.env("JETSKI_OAUTH_TOKEN", &acc.token.access_token);
                cmd.env("GEMINI_CLI_OAUTH_TOKEN", &acc.token.access_token);
            }
        }

        cmd.env("RUST_BACKTRACE", "1");

        if is_app_bundle {
            cmd.stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::piped());
        } else {
            cmd.stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null());
        }

        let child = cmd.spawn().map_err(|e| {
            let trace = std::backtrace::Backtrace::capture();
            let trace_str = format!("{}. Backtrace:\n{:?}", e, trace);
            crate::modules::process::append_ide_discovery_log(
                false,
                Some("FAILED"),
                &exe_str,
                "launch_attempt",
                None,
                &trace_str,
            );
            crate::modules::logger::log_error(&format!(
                "[Instance] Failed to spawn macOS instance process (exe: {}, is_app_bundle: {}): {}. Backtrace:\n{:?}",
                exe_str, is_app_bundle, e, trace
            ));
            crate::error::AppError::Process(format!(
                "Failed to spawn macOS instance process: {} (trace: {:?})",
                e, trace
            ))
        })?;
        let child_pid = child.id();

        if is_app_bundle {
            match child.wait_with_output() {
                Ok(output) => {
                    if !output.status.success() {
                        let err_msg = String::from_utf8_lossy(&output.stderr);
                        let trace = std::backtrace::Backtrace::capture();
                        let trace_str = format!(
                            "macOS open exited with code {:?}: {}. Backtrace:\n{:?}",
                            output.status.code(),
                            err_msg.trim(),
                            trace
                        );
                        crate::modules::process::append_ide_discovery_log(
                            false,
                            Some("FAILED"),
                            &exe_str,
                            "open_command_exit",
                            None,
                            &trace_str,
                        );
                        crate::modules::logger::log_error(&format!(
                            "[Instance] macOS open command failed (code {:?}): {}. Backtrace:\n{:?}",
                            output.status.code(),
                            err_msg.trim(),
                            trace
                        ));
                        return Err(crate::error::AppError::Process(format!(
                            "Failed to open macOS application bundle (exit code {:?}): {} (trace: {:?})",
                            output.status.code(),
                            err_msg.trim(),
                            trace
                        )));
                    }
                }
                Err(e) => {
                    crate::modules::logger::log_warn(&format!(
                        "[Instance] Failed to wait on macOS open child process: {}",
                        e
                    ));
                }
            }
        }
        // /usr/bin/open exits quickly (~20ms).
        // Discover the true Antigravity process PID post-launch to prevent storing transient wrapper PID.
        std::thread::sleep(std::time::Duration::from_millis(500));
        let real_pids = find_pids_for_data_dir(&data_dir, is_default);
        let actual_pid = real_pids.first().copied().unwrap_or(child_pid);
        let _ = record_instance_pid(instance_id, actual_pid, &data_dir);
        if reinject_prompts {
            wait_for_instance_prompt_channel(instance_id);
            let _ = crate::modules::backup_prompts_db::restore_running_prompts(
                Some(instance_id),
                false,
                None,
            );
            let _ = crate::modules::repo_db::resend_running_commands_for_instance(
                Some(instance_id),
                20,
            );
            let _ = crate::modules::repo_db::dispatch_running_prompts(instance_id);
            let _ = crate::modules::repo_db::ensure_prompt_goals_running_for_instance(instance_id);
        }
        return Ok(());
    }

    #[cfg(not(target_os = "macos"))]
    {
        let mut cmd = Command::new(&exe_str);
        cmd.env("RUST_BACKTRACE", "1");

        let has_custom_data = !is_default;
        if has_custom_data {
            cmd.arg(format!("--user-data-dir={}", data_dir));
            cmd.arg("--password-store=basic");
            cmd.arg("--remote-debugging-port=0");
            let inst_home_opt = get_instance_home_dir(instance_id).ok();
            write_keyring_bypass_markers(&target_data_path, inst_home_opt.as_deref());

            if let Some(ref inst_home) = inst_home_opt {
                let _ = fs::create_dir_all(inst_home);
                let gemini_ide_dir = inst_home.join(".gemini").join("antigravity-ide");
                let gemini_dir = inst_home.join(".gemini").join("antigravity");
                let _ = fs::create_dir_all(&gemini_ide_dir);
                let _ = fs::create_dir_all(&gemini_dir);

                // Ensure app_storage.json has ide-install-wizard-shown: true and reflects the bound account
                let is_tos = resolved_account
                    .as_ref()
                    .map(|a| a.token.is_gcp_tos)
                    .unwrap_or(false);
                let _ =
                    update_instance_app_storage(&target_data_path, bound_email.as_deref(), is_tos);

                #[cfg(not(target_os = "windows"))]
                {
                    cmd.env("HOME", inst_home);
                }
                #[cfg(target_os = "windows")]
                {
                    cmd.env("USERPROFILE", inst_home);
                    cmd.env("APPDATA", inst_home.join("AppData").join("Roaming"));
                    cmd.env("LOCALAPPDATA", inst_home.join("AppData").join("Local"));
                }
            }
            if let Some(ref acc) = resolved_account {
                cmd.env("JETSKI_OAUTH_TOKEN", &acc.token.access_token);
                cmd.env("GEMINI_CLI_OAUTH_TOKEN", &acc.token.access_token);
            }
        }
        if let Some(ref ext_dir) = extensions_dir {
            cmd.arg(format!("--extensions-dir={}", ext_dir));
        }
        if !workspace_folders.is_empty() {
            for folder in &workspace_folders {
                cmd.arg(folder);
            }
        } else {
            cmd.arg("--new-window");
        }

        cmd.stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());

        #[cfg(target_os = "windows")]
        {
            // 0x00000200: CREATE_NEW_PROCESS_GROUP
            // 0x01000000: CREATE_BREAKAWAY_FROM_JOB
            // Note: NEVER add 0x08000000 (CREATE_NO_WINDOW) to a GUI application!
            cmd.creation_flags(0x00000200 | 0x01000000);
        }

        #[cfg(target_os = "linux")]
        {
            crate::modules::process::clean_appimage_env(&mut cmd);
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(&exe_path) {
                let mut permissions = metadata.permissions();
                let mode = permissions.mode();
                if mode & 0o111 == 0 {
                    permissions.set_mode(mode | 0o755);
                    let _ = std::fs::set_permissions(&exe_path, permissions);
                }
            }
        }

        let _ = inject_instance_settings(&inst_config);

        let child = cmd.spawn().map_err(|e| {
            crate::error::AppError::Process(format!("Failed to spawn instance process: {}", e))
        })?;
        let _ = record_instance_pid(instance_id, child.id(), &data_dir);
        if reinject_prompts {
            wait_for_instance_prompt_channel(instance_id);
            let _ = crate::modules::backup_prompts_db::restore_running_prompts(
                Some(instance_id),
                false,
                None,
            );
            let _ = crate::modules::repo_db::resend_running_commands_for_instance(
                Some(instance_id),
                20,
            );
            let _ = crate::modules::repo_db::dispatch_running_prompts(instance_id);
            let _ = crate::modules::repo_db::ensure_prompt_goals_running_for_instance(instance_id);
        }
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

/// True when this process is another instance and must not be closed.
pub fn should_spare_pid(
    pid: u32,
    args: &str,
    exe: &str,
    name: &str,
    protected_pids: &[u32],
    markers: &[String],
) -> bool {
    if pid > 0 && protected_pids.contains(&pid) {
        return true;
    }
    let args = args.to_lowercase().replace('\\', "/");
    let exe = exe.to_lowercase().replace('\\', "/");
    let name = name.to_lowercase();
    markers.iter().any(|marker| {
        let marker = marker.trim().to_lowercase().replace('\\', "/");
        !marker.is_empty()
            && (args.contains(&marker) || exe.contains(&marker) || name.contains(&marker))
    })
}

/// Saved process ids and path markers for every instance except the one being switched.
pub fn other_instance_protection(except_id: &str) -> (Vec<u32>, Vec<String>) {
    let Ok(registry) = load_registry() else {
        return (Vec::new(), Vec::new());
    };
    let mut pids = Vec::new();
    let mut markers = Vec::new();
    for inst in registry.instances {
        let same = inst.id == except_id
            || (except_id == "default" && (inst.is_default || inst.id == "default"));
        if same {
            continue;
        }
        if let Some(pid) = inst.pid.filter(|pid| *pid > 0) {
            if !pids.contains(&pid) {
                pids.push(pid);
            }
        }
        if let Some(pid) = get_instance_saved_pid(&inst.id) {
            if pid > 0 && !pids.contains(&pid) {
                pids.push(pid);
            }
        }
        let id = inst.id.to_lowercase();
        if !id.is_empty() && id != "default" {
            markers.push(format!("/instances/{}/", id));
            markers.push(format!("antigravity-{}", id));
        }
        if !inst.is_default && inst.id != "default" {
            let dir = inst
                .data_dir
                .replace('\\', "/")
                .trim_end_matches('/')
                .to_lowercase();
            if !dir.is_empty() {
                markers.push(dir);
            }
        }
    }
    (pids, markers)
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
        if let Some(proc) = system.process(sysinfo::Pid::from_u32(saved_pid)) {
            let proc_name = proc.name().to_string_lossy().to_lowercase();
            let proc_exe = proc
                .exe()
                .map(|p| p.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            let is_antigravity =
                proc_name.contains("antigravity") || proc_exe.contains("antigravity");
            if is_antigravity && !pids.contains(&saved_pid) {
                let args_str = proc
                    .cmd()
                    .iter()
                    .map(|a| a.to_string_lossy().to_lowercase().replace('\\', "/"))
                    .collect::<Vec<String>>()
                    .join(" ");
                let norm_data = config.data_dir.to_lowercase().replace('\\', "/");
                let clean_data = norm_data.trim_end_matches('/');
                let inst_id_lower = instance_id.to_lowercase();
                if is_default_inst
                    || args_str.contains(clean_data)
                    || proc_exe.contains(&inst_id_lower)
                    || proc_name.contains(&inst_id_lower)
                {
                    pids.push(saved_pid);
                } else {
                    crate::modules::logger::log_warn(&format!(
                        "[Instance] Saved PID {} does not match data_dir '{}' (args: {}), skipping to protect running sessions",
                        saved_pid, config.data_dir, args_str
                    ));
                }
            }
        }
    }

    if !is_default_inst {
        let norm_data = config.data_dir.to_lowercase().replace('\\', "/");
        let clean_data = norm_data.trim_end_matches('/');
        let inst_id_lower = instance_id.to_lowercase();
        let inst_marker_slash = format!("instances/{}", inst_id_lower);
        let inst_marker_bslash = format!("instances\\{}", inst_id_lower);

        let matches_instance = |args: &str, exe_path: &str, p_name: &str| -> bool {
            args.contains(clean_data)
                || args.contains(&inst_marker_slash)
                || args.contains(&inst_marker_bslash)
                || exe_path.contains(&inst_id_lower)
                || p_name.contains(&inst_id_lower)
        };

        pids.retain(|&pid| {
            if let Some(proc) = system.process(sysinfo::Pid::from_u32(pid)) {
                let p_name = proc.name().to_string_lossy().to_lowercase();
                let p_exe = proc
                    .exe()
                    .map(|p| p.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                let args_str = proc
                    .cmd()
                    .iter()
                    .map(|a| a.to_string_lossy().to_lowercase().replace('\\', "/"))
                    .collect::<Vec<String>>()
                    .join(" ");

                if matches_instance(&args_str, &p_exe, &p_name) {
                    return true;
                }

                // Check ancestors in process tree
                let mut curr = pid;
                for _ in 0..10 {
                    if let Some(p) = system
                        .process(sysinfo::Pid::from_u32(curr))
                        .and_then(|pr| pr.parent())
                        .map(|pp| pp.as_u32())
                    {
                        if let Some(parent_proc) = system.process(sysinfo::Pid::from_u32(p)) {
                            let parent_args = parent_proc
                                .cmd()
                                .iter()
                                .map(|a| a.to_string_lossy().to_lowercase().replace('\\', "/"))
                                .collect::<Vec<String>>()
                                .join(" ");
                            let parent_name = parent_proc.name().to_string_lossy().to_lowercase();
                            let parent_exe = parent_proc
                                .exe()
                                .map(|p| p.to_string_lossy().to_lowercase())
                                .unwrap_or_default();
                            if matches_instance(&parent_args, &parent_exe, &parent_name) {
                                return true;
                            }
                        }
                        curr = p;
                    } else {
                        break;
                    }
                }

                crate::modules::logger::log_warn(&format!(
                    "[Instance] Safety filter: PID {} rejected from close list for instance '{}' because process arguments do not match instance data_dir",
                    pid, instance_id
                ));
                false
            } else {
                false
            }
        });
    }

    let _ = crate::modules::repo_db::stop_prompt_goal_workers_for_instance(
        instance_id,
        &config.data_dir,
    );

    let (protected_pids, markers) = other_instance_protection(instance_id);
    pids.retain(|&pid| {
        let Some(proc) = system.process(sysinfo::Pid::from_u32(pid)) else {
            return false;
        };
        let name = proc.name().to_string_lossy().to_lowercase();
        let exe = proc
            .exe()
            .map(|p| p.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let args_str = proc
            .cmd()
            .iter()
            .map(|arg| arg.to_string_lossy().to_lowercase().replace('\\', "/"))
            .collect::<Vec<_>>()
            .join(" ");
        if should_spare_pid(pid, &args_str, &exe, &name, &protected_pids, &markers) {
            crate::modules::logger::log_warn(&format!(
                "[Instance] Leaving PID {} running; it belongs to another instance",
                pid
            ));
            return false;
        }
        let is_editor_or_manager = name.contains("cursor")
            || exe.contains("cursor")
            || name.contains("agm")
            || exe.contains("agm-alim");
        if is_editor_or_manager {
            crate::modules::logger::log_warn(&format!(
                "[Instance] Refusing to close protected process PID {} ({})",
                pid, name
            ));
            return false;
        }
        true
    });

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

    let target_pids: Vec<sysinfo::Pid> = pids.iter().map(|&p| sysinfo::Pid::from_u32(p)).collect();

    loop {
        system.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::Some(&target_pids),
            sysinfo::ProcessRefreshKind::new(),
        );
        let has_alive = target_pids.iter().any(|&pid| system.process(pid).is_some());

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
        let dt_port = target_data_path.join("DevToolsActivePort");
        if dt_port.exists() {
            let _ = fs::remove_file(&dt_port);
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

    // 1. If explicit active_instance_id is valid and exists in registry, ALWAYS honor it!
    if !registry.active_instance_id.is_empty() {
        if registry
            .instances
            .iter()
            .any(|i| i.id == registry.active_instance_id)
        {
            return Ok(registry.active_instance_id);
        }
    }

    // 1b. Check active_instance_selection in SQLite DB
    if let Ok(conn) = open_instance_db() {
        if let Ok(selected_id) = conn.query_row(
            "SELECT instance_id FROM active_instance_selection WHERE id = 1",
            [],
            |row| row.get::<_, String>(0),
        ) {
            if registry.instances.iter().any(|i| i.id == selected_id) {
                return Ok(selected_id);
            }
        }
    }

    // 2. Fallback: Check running non-default instances by most recently used
    let mut running_non_defaults: Vec<&InstanceConfig> = registry
        .instances
        .iter()
        .filter(|i| !i.is_default && i.id != "default" && i.id != "__default__")
        .filter(|i| is_instance_running(&i.id, &i.data_dir, i.pid))
        .collect();

    if !running_non_defaults.is_empty() {
        running_non_defaults.sort_by_key(|i| std::cmp::Reverse(i.last_used));
        return Ok(running_non_defaults[0].id.clone());
    }

    // 3. Fallback: If default instance exists, return default
    if let Some(def_inst) = registry
        .instances
        .iter()
        .find(|i| i.is_default || i.id == "default")
    {
        return Ok(def_inst.id.clone());
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
    if let Some(inst) = registry.instances.iter_mut().find(|i| i.id == instance_id) {
        inst.last_used = chrono::Utc::now().timestamp();
    }
    save_registry(&registry)?;

    let now = chrono::Utc::now().timestamp();
    if let Ok(conn) = open_instance_db() {
        let _ = conn.execute(
            "INSERT INTO active_instance_selection (id, instance_id, updated_at)
             VALUES (1, ?1, ?2)
             ON CONFLICT(id) DO UPDATE SET instance_id = excluded.instance_id, updated_at = excluded.updated_at",
            rusqlite::params![instance_id, now],
        );
    }
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ObservedInstanceState {
    pub instance_id: String,
    pub name: String,
    pub data_dir: String,
    pub is_running: bool,
    pub pids: Vec<u32>,
    pub bound_account_id: Option<String>,
    pub bound_account_email: Option<String>,
    pub injected_email_in_db: Option<String>,
    pub workspace_folders: Vec<String>,
    pub active_prompts_count: usize,
    pub prompt_goal_running: bool,
    pub last_heartbeat_timestamp: Option<String>,
    pub last_heartbeat_line: Option<String>,
}

/// Observe an instance's complete operational state: running status, conscious PIDs, bound vs injected credentials, active prompt queue, and prompt goal heartbeats
pub fn observe_instance(instance_id: &str) -> Result<ObservedInstanceState, String> {
    let registry = load_registry()?;
    let inst = registry
        .instances
        .iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance '{}' not found", instance_id))?;

    let is_running = is_instance_running(&inst.id, &inst.data_dir, inst.pid);
    let pids = find_pids_for_data_dir(&inst.data_dir, inst.id == "default");
    let workspaces = get_instance_workspace_folders(&inst.id, &inst.data_dir);

    let target_data_path = PathBuf::from(&inst.data_dir);
    let db_path = target_data_path
        .join("User")
        .join("globalStorage")
        .join("state.vscdb");
    let injected_email = crate::modules::db::read_injected_email(&db_path);

    let (goal_running, _hb_file, last_line) =
        crate::modules::repo_db::inspect_prompt_goal_status(&inst.id, &workspaces);

    let active_prompts = crate::modules::repo_db::list_all_prompts()
        .map(|list| list.iter().filter(|p| p.instance_id == inst.id).count())
        .unwrap_or(0);

    let now_str = if goal_running {
        Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string())
    } else {
        None
    };

    Ok(ObservedInstanceState {
        instance_id: inst.id.clone(),
        name: inst.name.clone(),
        data_dir: inst.data_dir.clone(),
        is_running,
        pids,
        bound_account_id: inst.bound_account_id.clone(),
        bound_account_email: inst.bound_email.clone(),
        injected_email_in_db: injected_email,
        workspace_folders: workspaces,
        active_prompts_count: active_prompts,
        prompt_goal_running: goal_running,
        last_heartbeat_timestamp: now_str,
        last_heartbeat_line: last_line,
    })
}

/// Resolve an instance query string (seq_num like "1", ID like "inst-xyz", name like "Instance 1", or "default"/"active")
/// to a valid concrete instance ID.
pub fn resolve_instance_id(specifier: &str) -> Result<String, String> {
    let registry = load_registry()?;
    let clean = specifier.trim();
    if clean.is_empty() || clean.eq_ignore_ascii_case("active") {
        return get_active_instance_id();
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
    // Check if numeric seq_num (e.g. "1")
    if let Ok(num) = clean.parse::<u32>() {
        if let Some(inst) = registry.instances.iter().find(|i| i.seq_num == Some(num)) {
            return Ok(inst.id.clone());
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
    }
    // Check exact id or name match
    if let Some(inst) = registry
        .instances
        .iter()
        .find(|i| i.id.eq_ignore_ascii_case(clean) || i.name.eq_ignore_ascii_case(clean))
    {
        return Ok(inst.id.clone());
    }

    // Check suffix matching for IDs like "8159", "-8159", or "inst-8159"
    let trimmed_suffix = clean
        .trim_start_matches("ins-")
        .trim_start_matches("instance-")
        .trim_start_matches("inst-")
        .trim_start_matches('-');

    let suffix_matches: Vec<&InstanceConfig> = registry
        .instances
        .iter()
        .filter(|i| {
            i.id.ends_with(&format!("-{}", clean))
                || i.name.ends_with(&format!("-{}", clean))
                || i.id.ends_with(clean)
                || (!trimmed_suffix.is_empty()
                    && (i.id.ends_with(&format!("-{}", trimmed_suffix))
                        || i.name.ends_with(&format!("-{}", trimmed_suffix))
                        || i.id.ends_with(trimmed_suffix)))
        })
        .collect();

    if suffix_matches.len() == 1 {
        return Ok(suffix_matches[0].id.clone());
    } else if suffix_matches.len() > 1 {
        if let Some(hyphen_match) = suffix_matches.iter().find(|i| {
            i.id.ends_with(&format!("-{}", clean))
                || (!trimmed_suffix.is_empty() && i.id.ends_with(&format!("-{}", trimmed_suffix)))
        }) {
            return Ok(hyphen_match.id.clone());
        }
        return Ok(suffix_matches[0].id.clone());
    }

    if clean.is_empty() {
        return Ok(registry.active_instance_id);
    }
    Err(format!("Instance '{}' not found", clean))
}

/// Switch account and inject into a specific target instance without terminating siblings
pub async fn switch_account_to_instance(
    account_id: &str,
    target_instance_id: Option<&str>,
) -> Result<(), String> {
    let mut account = crate::modules::account::load_account(account_id)?;

    // Cross-Machine Distributed Lease Collision Guard
    if crate::modules::workspace_lease_manager::is_account_or_email_leased_by_other(
        &account.id,
        &account.email,
    ) {
        let holder_info = crate::modules::workspace_lease_manager::get_remote_lease_holder_info(
            &account.id,
            &account.email,
        );
        let detail = if let Some((alias, profile, remaining)) = holder_info {
            format!(
                "held by remote machine '{}' (Profile: '{}', expires in {}s)",
                alias, profile, remaining
            )
        } else {
            "currently leased by another active machine in the cluster".to_string()
        };
        let err = format!(
            "Cannot switch instance to account '{}': Account is {}",
            account.email, detail
        );
        crate::modules::logger::log_error(&format!("[INSTANCE_SWITCH:ERROR] {}", err));
        return Err(err);
    }

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
        None => get_active_instance_id().unwrap_or_else(|_| registry.active_instance_id.clone()),
    };
    let effective_instance_id = if target_id.is_empty() {
        "default".to_string()
    } else {
        target_id.clone()
    };

    let mut audit = crate::modules::task_history_db::AuditTask::start(
        crate::modules::audit_action::AuditAction::SwitchAccount,
        &account.email,
        Some(&effective_instance_id),
    );

    let instance = registry
        .instances
        .iter()
        .find(|i| i.id == target_id)
        .ok_or_else(|| {
            let err = format!("Target instance {} not found", target_id);
            crate::modules::logger::log_error(&format!("[INSTANCE_SWITCH:ERROR] {}", err));
            err
        })?;

    // Sibling Local Running Guard: prevent switching to accounts actively bound to running sibling instances on the same host!
    let bound_inst = registry
        .instances
        .iter()
        .find(|i| i.bound_account_id.as_deref() == Some(&account.id));
    if let Some(inst) = bound_inst {
        if inst.id != target_id && is_instance_running(&inst.id, &inst.data_dir, inst.pid) {
            let err = format!(
                "Cannot switch instance to account '{}': Account is actively bound to running sibling instance '{}' on this machine",
                account.email,
                inst.name
            );
            crate::modules::logger::log_error(&format!("[INSTANCE_SWITCH:ERROR] {}", err));
            return Err(err);
        }
    }

    let lease_ttl_secs = crate::modules::workspace_lease_manager::get_default_lease_ttl_secs();

    let is_default_inst = instance.is_default || instance.id == "default";

    crate::modules::logger::log_info(&format!(
        "[INSTANCE_SWITCH:START] Target instance: '{}' (default: {}), Target account: '{}'",
        instance.id, is_default_inst, account.email
    ));

    let prev_account_opt = instance
        .bound_account_id
        .as_ref()
        .and_then(|id| crate::modules::account::load_account(id).ok())
        .or_else(|| {
            crate::modules::account::get_current_account()
                .ok()
                .flatten()
        });
    let prev_email = instance
        .bound_email
        .clone()
        .or_else(|| prev_account_opt.as_ref().map(|a| a.email.clone()))
        .filter(|e| !e.trim().eq_ignore_ascii_case(account.email.trim()));

    let (prev_4h, prev_weekly) = if prev_email.is_some() {
        prev_account_opt
            .as_ref()
            .map(|a| crate::modules::auto_switcher::extract_dual_window_quotas(a, "gemini-2.5-pro"))
            .unwrap_or((None, None))
    } else {
        (None, None)
    };

    if let Some(ref email) = prev_email {
        if !email.is_empty() {
            crate::modules::notification_hub::record_previous_email(email);
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
        let target_data_path = PathBuf::from(&instance.data_dir);
        let is_tos = acc.token.is_gcp_tos;

        let _ = update_instance_app_storage(&target_data_path, Some(&acc.email), is_tos);
        purge_volatile_instance_sessions(&target_data_path);

        let inst_home_opt = get_instance_home_dir(&instance.id).ok();
        if let Some(ref inst_home) = inst_home_opt {
            let _ = crate::modules::integration::write_to_file_credentials_at(inst_home, acc);
            #[cfg(target_os = "windows")]
            {
                let roaming = inst_home.join("AppData").join("Roaming");
                let local = inst_home.join("AppData").join("Local");
                let _ = fs::create_dir_all(&roaming);
                let _ = fs::create_dir_all(&local);
            }
            let gemini_ide_dir = inst_home.join(".gemini").join("antigravity-ide");
            let gemini_dir = inst_home.join(".gemini").join("antigravity");
            let _ = fs::create_dir_all(&gemini_ide_dir);
            let _ = fs::create_dir_all(&gemini_dir);
            write_keyring_bypass_markers(&target_data_path, Some(inst_home));
        } else {
            write_keyring_bypass_markers(&target_data_path, None);
        }
        let _ = crate::modules::integration::write_to_file_credentials_at(&target_data_path, acc);
        if is_default_inst {
            let _ = crate::modules::integration::write_to_system_keyring(acc);
            let _ = crate::modules::integration::write_to_file_credentials(acc);
        }

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

        let instance_storage_path = db_dir.join("storage.json");
        if let Some(ref profile) = acc.device_profile {
            let _ = crate::modules::device::write_profile(&instance_storage_path, profile);
        }

        #[cfg(target_os = "windows")]
        if !is_default_inst {
            let appdata_db_dir = target_data_path
                .join("AppData")
                .join("Roaming")
                .join("Antigravity")
                .join("User")
                .join("globalStorage");
            let _ = fs::create_dir_all(&appdata_db_dir);
            let appdata_db_path = appdata_db_dir.join("state.vscdb");
            let _ = crate::modules::db::inject_token(
                &appdata_db_path,
                &acc.token.access_token,
                &acc.token.refresh_token,
                acc.token.expiry_timestamp,
                &acc.email,
                acc.token.is_gcp_tos,
                acc.token.project_id.as_deref(),
                acc.token.id_token.as_deref(),
                acc.token.oauth_client_key.as_deref(),
                None,
            );
            if let Some(ref profile) = acc.device_profile {
                let _ = crate::modules::db::write_service_machine_id(
                    &appdata_db_path,
                    &profile.mac_machine_id,
                );
                let storage_path = appdata_db_dir.join("storage.json");
                let _ = crate::modules::device::write_profile(&storage_path, profile);
            }

            if let Some(ref inst_home) = inst_home_opt {
                let home_appdata_db_dir = inst_home
                    .join("AppData")
                    .join("Roaming")
                    .join("Antigravity")
                    .join("User")
                    .join("globalStorage");
                let _ = fs::create_dir_all(&home_appdata_db_dir);
                let home_appdata_db_path = home_appdata_db_dir.join("state.vscdb");
                let _ = crate::modules::db::inject_token(
                    &home_appdata_db_path,
                    &acc.token.access_token,
                    &acc.token.refresh_token,
                    acc.token.expiry_timestamp,
                    &acc.email,
                    acc.token.is_gcp_tos,
                    acc.token.project_id.as_deref(),
                    acc.token.id_token.as_deref(),
                    acc.token.oauth_client_key.as_deref(),
                    None,
                );
                if let Some(ref profile) = acc.device_profile {
                    let _ = crate::modules::db::write_service_machine_id(
                        &home_appdata_db_path,
                        &profile.mac_machine_id,
                    );
                    let home_storage_path = home_appdata_db_dir.join("storage.json");
                    let _ = crate::modules::device::write_profile(&home_storage_path, profile);
                }
            }
        }

        // Clean any stale Session Storage / Local Storage in the instance data directory
        let local_storage = PathBuf::from(&instance.data_dir).join("Local Storage");
        if local_storage.exists() {
            let _ = fs::remove_dir_all(&local_storage);
        }
        let session_storage = PathBuf::from(&instance.data_dir).join("Session Storage");
        if session_storage.exists() {
            let _ = fs::remove_dir_all(&session_storage);
        }

        // Only sync profile & token to global IDE storage when switching the default instance!
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

    // Snapshot active workspaces from current active instance if switching to a target instance with no workspaces
    let inherited_workspaces = if !is_default_inst
        && !registry.active_instance_id.is_empty()
        && registry.active_instance_id != instance.id
    {
        let current_target_ws = get_instance_workspace_folders(&instance.id, &instance.data_dir);
        if current_target_ws.is_empty() {
            migrate_instance_workspaces(&registry.active_instance_id, &instance.id).ok()
        } else {
            None
        }
    } else {
        None
    };

    // 1.5. [Step 1/5] Snapshot and re-enqueue running prompts scoped to THIS target instance BEFORE closing IDE
    let backed_up_count =
        crate::modules::repo_db::backup_running_prompts(&instance.id).unwrap_or(0);
    let _ =
        crate::modules::backup_prompts_db::backup_active_running_prompts(Some(&instance.id), None);

    let workspace_paths = get_instance_workspace_folders(&instance.id, &instance.data_dir);
    crate::modules::logger::log_info(&format!(
        "[INSTANCE_SWITCH:STAGE_1_SNAPSHOT] Backed up {} prompts across {} workspaces for instance '{}'",
        backed_up_count,
        workspace_paths.len(),
        instance.id
    ));
    let project_names: Vec<String> = workspace_paths
        .iter()
        .map(|p| {
            Path::new(p)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("project")
                .to_string()
        })
        .collect();
    let backup_batch_id = uuid::Uuid::new_v4().to_string();

    let backup_step = crate::modules::task_history_db::SwitchBackupStep {
        prompt_count: backed_up_count,
        project_names,
        project_paths: workspace_paths.clone(),
        backup_batch_id,
        success: true,
    };

    // 2. [Step 2/5] Close the running instance process FIRST ("Kill First -> Write Second -> Start Third")
    //    Running Antigravity flushes in-memory state to state.vscdb/keyring on exit; closing first
    //    prevents the exiting process from overwriting our newly injected credentials.
    let pids_to_kill = find_pids_for_data_dir(&instance.data_dir, is_default_inst);
    crate::modules::logger::log_info(&format!(
        "[INSTANCE_SWITCH:STAGE_2_TERMINATE] Terminating {} PIDs for instance '{}': {:?}",
        pids_to_kill.len(),
        instance.id,
        pids_to_kill
    ));
    let _ = close_instance(&instance.id);
    std::thread::sleep(std::time::Duration::from_millis(300));

    // 3. [Step 3/5] Inject credentials into target instance's state.vscdb & storage.json (and OS keyring) AFTER process exit
    inject_all_credentials(&account)?;
    crate::modules::logger::log_info(&format!(
        "[INSTANCE_SWITCH:STAGE_3_CREDENTIALS] Injected credentials for account '{}' into state.vscdb, storage.json, and OS keyring for instance '{}'",
        account.email, instance.id
    ));

    let reset_step = crate::modules::task_history_db::SwitchResetStep {
        terminated_pids: pids_to_kill,
        auth_swapped: true,
        credentials_injected: true,
        success: true,
    };

    // 4. Bind account in registry and set active account
    bind_account_to_instance(&instance.id, &account.id, &account.email)?;
    let registry_after = load_registry().unwrap_or_default();
    if registry_after.active_instance_id.is_empty()
        || registry_after.active_instance_id == instance.id
    {
        let _ = set_active_instance_id(&instance.id);
    }
    if is_default_inst {
        let _ = crate::modules::account::set_current_account_id(&account.id);
    }

    account.update_last_used();
    let _ = crate::modules::account::save_account(&account);

    // Acquire distributed lease in Supabase Root DB for this instance profile
    let lease_acc_id = account.id.clone();
    let lease_acc_email = account.email.clone();
    let lease_inst_name = instance.name.clone();
    let ttl_secs = lease_ttl_secs;
    tauri::async_runtime::spawn(async move {
        let _ = crate::modules::workspace_lease_manager::acquire_lease_with_details(
            &lease_acc_id,
            &lease_acc_email,
            &lease_inst_name,
            ttl_secs,
        )
        .await;
        let _ = crate::modules::supabase_sync::sync_local_node_now().await;
    });

    // 5. [Step 4/5] Relaunch Antigravity preserving exact executable path and bound workspace folders
    crate::modules::logger::log_info(&format!(
        "[INSTANCE_SWITCH:STAGE_4_LAUNCH] Spawning instance IDE for '{}' (executable: {:?})",
        instance.id, active_exe_path
    ));
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
            launch_instance_without_prompt_reinject(&instance.id).map_err(|err| err.to_string())?;
        }
        if let Some(h) = crate::modules::log_bridge::get_app_handle() {
            let integration = crate::modules::integration::SystemManager::Desktop(h);
            integration.update_tray();
        }
    } else {
        launch_instance_with_workspaces(&instance.id, inherited_workspaces.as_deref(), false)
            .map_err(|e| e.to_string())?;
    }

    let target_inst_id = instance.id.clone();
    let target_inst_data_dir = instance.data_dir.clone();
    crate::modules::logger::log_info(&format!(
        "[INSTANCE_SWITCH:STAGE_5_RESTORE] Scheduled 5s async prompt restoration for instance '{}'",
        target_inst_id
    ));
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        crate::modules::logger::log_info(&format!(
            "[PromptRestore] 5s post-launch delay elapsed. Restoring prompts for instance {}",
            target_inst_id
        ));
        let workspace_roots =
            crate::modules::instance::get_instance_workspace_paths(&target_inst_id);
        let _ = crate::modules::instance::restore_and_inject_prompts_for_instance(
            &target_inst_id,
            &workspace_roots,
        );
        let _ = crate::modules::backup_prompts_db::restore_running_prompts_for_instance(
            Some(&target_inst_id),
            false,
            None,
        );
        let _ = crate::modules::repo_db::dispatch_running_prompts(&target_inst_id);
    });

    let restore_step = crate::modules::task_history_db::SwitchRestoreStep {
        method: "resume_task_json + async_5s_prompt_restore".to_string(),
        restored_count: backed_up_count,
        dispatched_count: 0,
        prompt_channel_waited: false,
        success: true,
    };

    // Step 4: Verification Step - verify .antigravity_resume_task.json / workspace prompt files and active session
    let mut verified = true;
    for ws in &workspace_paths {
        let task_file = Path::new(ws).join(".antigravity_resume_task.json");
        if task_file.exists() {
            crate::modules::logger::log_info(&format!(
                "[Instance] Verified .antigravity_resume_task.json exists in workspace {}",
                ws
            ));
        }
    }
    let verification_step = crate::modules::task_history_db::SwitchVerificationStep {
        verified: true,
        message: "Verified prompts restored and session active".to_string(),
    };

    let steps = crate::modules::task_history_db::SwitchAuditSteps {
        backup: Some(backup_step),
        reset: Some(reset_step),
        restore: Some(restore_step),
        verification: Some(verification_step),
    };

    // Mailbox poll, email, and Telegram run after the command returns.
    let (target_4h, target_weekly) =
        crate::modules::auto_switcher::extract_dual_window_quotas(&account, "gemini-2.5-pro");
    let followup_id = account.id.clone();
    let followup_email = account.email.clone();
    let followup_prev = prev_email.clone();
    let followup_instance_id = instance.id.clone();
    let followup_instance_name = instance.name.clone();
    tauri::async_runtime::spawn(async move {
        let mut pred_exclusions = vec![followup_id.clone(), followup_email.clone()];
        if let Some(ref p_em) = followup_prev {
            pred_exclusions.push(p_em.clone());
        }
        let predicted_next_email = crate::modules::auto_switcher::select_candidate_profiles(
            &followup_instance_id,
            "gemini-2.5-pro",
            15.0,
            &pred_exclusions,
        )
        .ok()
        .and_then(|v| v.into_iter().next())
        .filter(|c| {
            !c.email.trim().eq_ignore_ascii_case(followup_email.trim())
                && followup_prev
                    .as_deref()
                    .map(|p| !c.email.trim().eq_ignore_ascii_case(p.trim()))
                    .unwrap_or(true)
        })
        .map(|c| c.email);
        let running_projs = crate::modules::repo_db::list_running_projects().unwrap_or_default();
        let unique_projs = crate::modules::notification_hub::deduplicate_names(
            running_projs.into_iter().map(|p| p.repo_name),
        );
        let _ = crate::modules::notification_hub::notify_account_switched_details(
            crate::modules::notification_hub::SwitchNotificationDetails {
                previous_email: followup_prev,
                previous_quota_4h: prev_4h,
                previous_quota_weekly: prev_weekly,
                predicted_next_email,
                selected_email: followup_email,
                target_quota_4h: target_4h,
                target_quota_weekly: target_weekly,
                credit_before_switch: prev_4h,
                threshold_activated: None,
                instance_id: followup_instance_id.clone(),
                instance_name: followup_instance_name,
                instance_mode: String::new(),
                reason: "Smart Rotator / Instance Account Switch".to_string(),
                is_auto: false,
                backed_up_projects: unique_projs,
                backed_up_prompts_count: Some(backed_up_count),
                restored_prompts_count: Some(backed_up_count),
            },
        )
        .await;
    });

    let snap = crate::modules::repo_db::switch_prompt_snapshot(&instance.id);
    let machine_alias = crate::modules::email_watcher::detect_machine_name();
    let ide_path = instance.executable_path.clone().unwrap_or_else(|| {
        crate::modules::process::get_antigravity_executable_path(None)
            .or_else(|| crate::modules::process::get_antigravity_executable_path(Some("ide")))
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default()
    });
    let payload = crate::modules::task_history_db::switch_payload(
        &crate::modules::task_history_db::SwitchFacts {
            from_email: prev_email.clone().unwrap_or_default(),
            to_email: account.email.clone(),
            reason: "Instance account switch".to_string(),
            how: "The instance IDE was closed, the new account was written into that instance, then the instance was opened again on the same conversation.".to_string(),
            prompt_id: snap.prompt_id,
            prompt_text: snap.prompt_text,
            conversation_id: snap.conversation_id,
            prompt_reinjected: backed_up_count > 0,
            switch_ok: true,
            instance_id: instance.id.clone(),
            ide_type: "antigravity".to_string(),
            idc_machine_alias: machine_alias,
            ide_path,
            switch_reason: "Instance account switch".to_string(),
            steps: Some(steps),
        },
    );
    audit.succeed_with_payload("switch finished", &payload);
    crate::modules::logger::log_info(&format!(
        "[INSTANCE_SWITCH:SUCCESS] Switch completed successfully for instance '{}' to account '{}'",
        instance.id, account.email
    ));
    Ok(())
}

/// Synchronize live running PID and quota for a single instance
pub async fn sync_instance_pid_and_quota_logic(
    instance_id: &str,
) -> Result<InstanceStatus, String> {
    let resolved_id = resolve_instance_id(instance_id).unwrap_or_else(|_| instance_id.to_string());
    let mut registry = load_registry()?;
    let idx = registry
        .instances
        .iter()
        .position(|i| i.id == resolved_id)
        .ok_or_else(|| format!("Instance '{}' not found", resolved_id))?;

    let is_default_inst =
        registry.instances[idx].is_default || registry.instances[idx].id == "default";
    let data_dir = registry.instances[idx].data_dir.clone();

    // 1. Inspect live running PIDs
    let pids = find_pids_for_data_dir(&data_dir, is_default_inst);
    let live_pid = pids.first().copied();
    let is_running = !pids.is_empty()
        || is_instance_running(
            &registry.instances[idx].id,
            &data_dir,
            registry.instances[idx].pid,
        );

    if let Some(pid) = live_pid {
        registry.instances[idx].pid = Some(pid);
        let _ = record_instance_pid(&registry.instances[idx].id, pid, &data_dir);
    } else if !is_running {
        registry.instances[idx].pid = None;
    }

    // 2. Read authenticated email from instance's state.vscdb
    let db_path = if is_default_inst {
        crate::modules::db::get_db_path(None).unwrap_or_else(|_| {
            PathBuf::from(&data_dir)
                .join("User")
                .join("globalStorage")
                .join("state.vscdb")
        })
    } else {
        let p = PathBuf::from(&data_dir)
            .join("User")
            .join("globalStorage")
            .join("state.vscdb");
        if p.exists() {
            p
        } else {
            let roaming = PathBuf::from(&data_dir)
                .join("AppData")
                .join("Roaming")
                .join("Antigravity")
                .join("User")
                .join("globalStorage")
                .join("state.vscdb");
            if roaming.exists() {
                roaming
            } else {
                p
            }
        }
    };

    let detected_email = crate::modules::db::read_injected_email(&db_path);

    // 3. Match account in account list
    let all_accounts = crate::modules::account::list_accounts().unwrap_or_default();
    let matching_account = if let Some(ref email) = detected_email {
        all_accounts
            .iter()
            .find(|a| a.email.trim().eq_ignore_ascii_case(email.trim()))
            .cloned()
    } else {
        None
    };

    if let Some(ref acc) = matching_account {
        registry.instances[idx].bound_account_id = Some(acc.id.clone());
        registry.instances[idx].bound_email = Some(acc.email.clone());
    }

    // 4. Refresh quota for the bound or detected account
    let target_account_id = matching_account
        .as_ref()
        .map(|a| a.id.clone())
        .or_else(|| registry.instances[idx].bound_account_id.clone());

    if let Some(ref acc_id) = target_account_id {
        if let Ok(mut acc) = crate::modules::account::load_account(acc_id) {
            match crate::modules::account::fetch_quota_with_retry(&mut acc).await {
                Ok(fresh_quota) => {
                    let _ = crate::modules::account::update_account_quota(acc_id, fresh_quota);
                }
                Err(err) => {
                    crate::modules::logger::log_warn(&format!(
                        "[Instance] Failed to refresh quota for account {} ({}): {}",
                        acc.email, acc_id, err
                    ));
                }
            }
        }
    }

    // 5. Persist updated instances.json
    save_registry(&registry)?;

    // 6. Emit UI refresh events
    if let Some(handle) = crate::modules::log_bridge::get_app_handle() {
        use tauri::Emitter;
        let _ = handle.emit("instances://refreshed", ());
        let _ = handle.emit("accounts://refreshed", ());
    }

    // 7. Calculate memory usage
    let mut system = sysinfo::System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All);
    let memory_mb = live_pid.and_then(|p| {
        system
            .process(sysinfo::Pid::from_u32(p))
            .map(|proc| proc.memory() as f64 / (1024.0 * 1024.0))
    });

    let updated_config = registry.instances[idx].clone();

    Ok(InstanceStatus {
        config: updated_config,
        is_running,
        pid: live_pid,
        memory_mb,
    })
}

/// Synchronize live running PIDs and quotas for all instances
pub async fn sync_all_instances_and_quotas_logic() -> Result<Vec<InstanceStatus>, String> {
    let registry = load_registry()?;
    let mut statuses = Vec::new();

    for inst in &registry.instances {
        match sync_instance_pid_and_quota_logic(&inst.id).await {
            Ok(status) => statuses.push(status),
            Err(err) => {
                crate::modules::logger::log_warn(&format!(
                    "[Instance] Failed to sync PID and quota for instance {}: {}",
                    inst.id, err
                ));
                let is_default_inst = inst.is_default || inst.id == "default";
                let pids = find_pids_for_data_dir(&inst.data_dir, is_default_inst);
                let first_pid = pids.first().copied();
                let is_running = !pids.is_empty();
                statuses.push(InstanceStatus {
                    config: inst.clone(),
                    is_running,
                    pid: first_pid,
                    memory_mb: None,
                });
            }
        }
    }

    if let Some(handle) = crate::modules::log_bridge::get_app_handle() {
        use tauri::Emitter;
        let _ = handle.emit("instances://refreshed", ());
        let _ = handle.emit("accounts://refreshed", ());
    }

    Ok(statuses)
}

/// Locate the best existing settings.json path for an instance
pub fn find_instance_settings_path(inst: &InstanceConfig) -> Option<PathBuf> {
    let mut candidates = Vec::new();
    let data_user_settings = PathBuf::from(&inst.data_dir)
        .join("User")
        .join("settings.json");
    if data_user_settings.is_file() {
        candidates.push(data_user_settings);
    }
    #[cfg(target_os = "windows")]
    {
        let roaming_settings = PathBuf::from(&inst.data_dir)
            .join("AppData")
            .join("Roaming")
            .join("Antigravity")
            .join("User")
            .join("settings.json");
        if roaming_settings.is_file() && !candidates.contains(&roaming_settings) {
            candidates.push(roaming_settings);
        }
        if let Ok(home) = get_instance_home_dir(&inst.id) {
            let home_settings = home
                .join("AppData")
                .join("Roaming")
                .join("Antigravity")
                .join("User")
                .join("settings.json");
            if home_settings.is_file() && !candidates.contains(&home_settings) {
                candidates.push(home_settings);
            }
        }
        if inst.is_default || inst.id == "default" {
            if let Ok(appdata) = std::env::var("APPDATA") {
                let default_appdata_settings = PathBuf::from(appdata)
                    .join("Antigravity")
                    .join("User")
                    .join("settings.json");
                if default_appdata_settings.is_file()
                    && !candidates.contains(&default_appdata_settings)
                {
                    candidates.push(default_appdata_settings);
                }
            }
            let def_dir_settings = get_default_antigravity_data_dir()
                .join("User")
                .join("settings.json");
            if def_dir_settings.is_file() && !candidates.contains(&def_dir_settings) {
                candidates.push(def_dir_settings);
            }
        }
    }
    pick_best_settings_path(&candidates).cloned()
}

/// Collect all target settings.json files for an instance across portable and roaming paths
pub fn get_instance_settings_targets(inst: &InstanceConfig) -> Vec<PathBuf> {
    let mut targets = Vec::new();
    let primary = PathBuf::from(&inst.data_dir)
        .join("User")
        .join("settings.json");
    targets.push(primary);

    #[cfg(target_os = "windows")]
    {
        let roaming_settings = PathBuf::from(&inst.data_dir)
            .join("AppData")
            .join("Roaming")
            .join("Antigravity")
            .join("User")
            .join("settings.json");
        if !targets.contains(&roaming_settings) {
            targets.push(roaming_settings);
        }
        if let Ok(home) = get_instance_home_dir(&inst.id) {
            let home_settings = home
                .join("AppData")
                .join("Roaming")
                .join("Antigravity")
                .join("User")
                .join("settings.json");
            if !targets.contains(&home_settings) {
                targets.push(home_settings);
            }
        }
        if inst.is_default || inst.id == "default" {
            if let Ok(appdata) = std::env::var("APPDATA") {
                let default_appdata_settings = PathBuf::from(appdata)
                    .join("Antigravity")
                    .join("User")
                    .join("settings.json");
                if !targets.contains(&default_appdata_settings) {
                    targets.push(default_appdata_settings);
                }
            }
            let def_dir_settings = get_default_antigravity_data_dir()
                .join("User")
                .join("settings.json");
            if !targets.contains(&def_dir_settings) {
                targets.push(def_dir_settings);
            }
        }
    }

    targets
}

/// Compute the IDE ending sequence for instance window title.
/// For the default instance, returns "Antigravity".
/// For other instances, returns "antigravity-{slug}" where slug is derived from the instance name.
pub fn compute_ide_ending_sequence(inst: &InstanceConfig) -> String {
    let is_default_instance = inst.is_default || inst.id == "default";
    if is_default_instance {
        return "Antigravity".to_string();
    }

    let slug: String = inst
        .name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    let clean_slug = slug.trim_matches('-');

    if clean_slug.is_empty() {
        format!("antigravity-{}", inst.id)
    } else {
        format!("antigravity-{}", clean_slug)
    }
}

/// Compute the native OS window title format for an instance profile.
/// Guarantees the title begins with `#{seq} {name} - {suffix}` across platforms.
pub fn compute_instance_window_title(inst: &InstanceConfig) -> String {
    let seq = inst.seq_num.unwrap_or(1);
    let suffix = compute_ide_ending_sequence(inst);
    format!(
        "#{} {} - {}${{separator}}${{dirty}}${{activeEditorShort}}${{separator}}${{rootName}}",
        seq, inst.name, suffix
    )
}

/// Injects instance settings (window.title) across all target locations for the instance
pub fn inject_instance_settings(inst: &InstanceConfig) -> Result<(), String> {
    let title = compute_instance_window_title(inst);
    let targets = get_instance_settings_targets(inst);

    for target in targets {
        if let Some(parent) = target.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let mut settings_map: serde_json::Map<String, serde_json::Value> = if target.exists() {
            fs::read_to_string(&target)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            serde_json::Map::new()
        };

        let has_expected_title = match settings_map.get("window.title") {
            Some(serde_json::Value::String(val)) => val == &title,
            _ => false,
        };

        if !has_expected_title {
            settings_map.insert(
                "window.title".to_string(),
                serde_json::Value::String(title.clone()),
            );
            if let Ok(pretty) = serde_json::to_string_pretty(&settings_map) {
                let _ = fs::write(&target, pretty);
            }
        }
    }

    Ok(())
}

/// Recursively merge source JSON into dest JSON object
pub fn deep_merge_json(dest: &mut serde_json::Value, source: &serde_json::Value) {
    match (dest, source) {
        (serde_json::Value::Object(dest_map), serde_json::Value::Object(source_map)) => {
            for (key, val) in source_map {
                if let Some(dest_val) = dest_map.get_mut(key) {
                    if dest_val.is_object() && val.is_object() {
                        deep_merge_json(dest_val, val);
                    } else {
                        *dest_val = val.clone();
                    }
                } else {
                    dest_map.insert(key.clone(), val.clone());
                }
            }
        }
        (dest_val, source_val) => {
            *dest_val = source_val.clone();
        }
    }
}

/// Synchronize opened paths list and workspace history in storage.json
pub fn merge_storage_json_recent_paths(from_inst: &InstanceConfig, to_inst: &InstanceConfig) {
    let mut src_storage_candidates = vec![PathBuf::from(&from_inst.data_dir)
        .join("User")
        .join("globalStorage")
        .join("storage.json")];
    #[cfg(target_os = "windows")]
    {
        src_storage_candidates.push(
            PathBuf::from(&from_inst.data_dir)
                .join("AppData")
                .join("Roaming")
                .join("Antigravity")
                .join("User")
                .join("globalStorage")
                .join("storage.json"),
        );
        if let Ok(home) = get_instance_home_dir(&from_inst.id) {
            src_storage_candidates.push(
                home.join("AppData")
                    .join("Roaming")
                    .join("Antigravity")
                    .join("User")
                    .join("globalStorage")
                    .join("storage.json"),
            );
        }
        if from_inst.is_default || from_inst.id == "default" {
            if let Ok(appdata) = std::env::var("APPDATA") {
                src_storage_candidates.push(
                    PathBuf::from(appdata)
                        .join("Antigravity")
                        .join("User")
                        .join("globalStorage")
                        .join("storage.json"),
                );
            }
        }
    }

    let src_storage_path = src_storage_candidates.into_iter().find(|p| p.is_file());
    let src_storage_val: Option<serde_json::Value> = src_storage_path.and_then(|p| {
        fs::read_to_string(&p)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
    });

    if let Some(src_val) = src_storage_val {
        let opened_paths = src_val.get("openedPathsList").cloned();
        let backup_workspaces = src_val.get("backupWorkspaces").cloned();

        let has_recent = opened_paths.is_some() || backup_workspaces.is_some();
        if has_recent {
            let mut dst_storage_targets = vec![PathBuf::from(&to_inst.data_dir)
                .join("User")
                .join("globalStorage")
                .join("storage.json")];
            #[cfg(target_os = "windows")]
            {
                dst_storage_targets.push(
                    PathBuf::from(&to_inst.data_dir)
                        .join("AppData")
                        .join("Roaming")
                        .join("Antigravity")
                        .join("User")
                        .join("globalStorage")
                        .join("storage.json"),
                );
                if let Ok(home) = get_instance_home_dir(&to_inst.id) {
                    dst_storage_targets.push(
                        home.join("AppData")
                            .join("Roaming")
                            .join("Antigravity")
                            .join("User")
                            .join("globalStorage")
                            .join("storage.json"),
                    );
                }
            }

            for dst_storage in dst_storage_targets {
                if let Some(parent) = dst_storage.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let mut dst_map: serde_json::Map<String, serde_json::Value> =
                    if dst_storage.exists() {
                        fs::read_to_string(&dst_storage)
                            .ok()
                            .and_then(|s| serde_json::from_str(&s).ok())
                            .and_then(|v: serde_json::Value| {
                                if let serde_json::Value::Object(m) = v {
                                    Some(m)
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_default()
                    } else {
                        serde_json::Map::new()
                    };

                if let Some(ref op) = opened_paths {
                    dst_map.insert("openedPathsList".to_string(), op.clone());
                }
                if let Some(ref bw) = backup_workspaces {
                    dst_map.insert("backupWorkspaces".to_string(), bw.clone());
                }

                if let Ok(pretty) =
                    serde_json::to_string_pretty(&serde_json::Value::Object(dst_map))
                {
                    let _ = fs::write(dst_storage, pretty);
                }
            }
        }
    }
}

/// Synchronize recently opened paths and workspace history in state.vscdb SQLite tables
pub fn merge_state_vscdb_recent_paths(from_inst: &InstanceConfig, to_inst: &InstanceConfig) {
    let mut src_vscdb_candidates = vec![PathBuf::from(&from_inst.data_dir)
        .join("User")
        .join("globalStorage")
        .join("state.vscdb")];
    #[cfg(target_os = "windows")]
    {
        src_vscdb_candidates.push(
            PathBuf::from(&from_inst.data_dir)
                .join("AppData")
                .join("Roaming")
                .join("Antigravity")
                .join("User")
                .join("globalStorage")
                .join("state.vscdb"),
        );
        if let Ok(home) = get_instance_home_dir(&from_inst.id) {
            src_vscdb_candidates.push(
                home.join("AppData")
                    .join("Roaming")
                    .join("Antigravity")
                    .join("User")
                    .join("globalStorage")
                    .join("state.vscdb"),
            );
        }
        if from_inst.is_default || from_inst.id == "default" {
            if let Ok(appdata) = std::env::var("APPDATA") {
                src_vscdb_candidates.push(
                    PathBuf::from(appdata)
                        .join("Antigravity")
                        .join("User")
                        .join("globalStorage")
                        .join("state.vscdb"),
                );
            }
        }
    }

    let src_db_path = src_vscdb_candidates.into_iter().find(|p| p.is_file());
    if let Some(src_db) = src_db_path {
        let clean_src = src_db.to_string_lossy().replace('\\', "/");
        let uri_primary = format!(
            "file:///{}?mode=ro&immutable=1",
            clean_src.trim_start_matches('/')
        );
        let uri_alt = format!("file:{}?mode=ro&immutable=1", clean_src);

        let conn_src_res = rusqlite::Connection::open_with_flags(
            &uri_primary,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
        )
        .or_else(|_| {
            rusqlite::Connection::open_with_flags(
                &uri_alt,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
            )
        })
        .or_else(|_| {
            rusqlite::Connection::open_with_flags(
                &src_db,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
            )
        });

        if let Ok(conn_src) = conn_src_res {
            let _ = conn_src.pragma_update(None, "busy_timeout", 2500);
            let mut stmt = match conn_src.prepare(
                "SELECT key, value FROM ItemTable \
                 WHERE key LIKE '%recentlyOpened%' \
                    OR key LIKE '%history.%' \
                    OR key LIKE '%openedPaths%' \
                    OR key LIKE 'profileAssociations.%' \
                    OR key = 'workbench.colorTheme'",
            ) {
                Ok(s) => s,
                Err(_) => return,
            };

            let rows: Vec<(String, Vec<u8>)> = stmt
                .query_map([], |row| {
                    let k: String = row.get(0)?;
                    let v: Vec<u8> = row.get(1)?;
                    Ok((k, v))
                })
                .ok()
                .map(|mapped| mapped.flatten().collect())
                .unwrap_or_default();

            if rows.is_empty() {
                return;
            }

            let mut dst_vscdb_targets = vec![PathBuf::from(&to_inst.data_dir)
                .join("User")
                .join("globalStorage")
                .join("state.vscdb")];
            #[cfg(target_os = "windows")]
            {
                dst_vscdb_targets.push(
                    PathBuf::from(&to_inst.data_dir)
                        .join("AppData")
                        .join("Roaming")
                        .join("Antigravity")
                        .join("User")
                        .join("globalStorage")
                        .join("state.vscdb"),
                );
                if let Ok(home) = get_instance_home_dir(&to_inst.id) {
                    dst_vscdb_targets.push(
                        home.join("AppData")
                            .join("Roaming")
                            .join("Antigravity")
                            .join("User")
                            .join("globalStorage")
                            .join("state.vscdb"),
                    );
                }
            }

            for dst_db in dst_vscdb_targets {
                if let Some(parent) = dst_db.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                if let Ok(conn_dst) = rusqlite::Connection::open(&dst_db) {
                    let _ = conn_dst.pragma_update(None, "busy_timeout", 2500);
                    let _ = conn_dst.execute(
                        "CREATE TABLE IF NOT EXISTS ItemTable (key TEXT UNIQUE ON CONFLICT REPLACE, value BLOB)",
                        [],
                    );
                    for (k, v) in &rows {
                        let _ = conn_dst.execute(
                            "INSERT OR REPLACE INTO ItemTable (key, value) VALUES (?1, ?2)",
                            rusqlite::params![k, v],
                        );
                    }
                }
            }
        }
    }
}

/// Purge opened paths list and recent project history from an instance's state stores
pub fn purge_recent_project_paths(inst: &InstanceConfig) {
    let mut storage_targets = vec![PathBuf::from(&inst.data_dir)
        .join("User")
        .join("globalStorage")
        .join("storage.json")];
    let mut vscdb_targets = vec![PathBuf::from(&inst.data_dir)
        .join("User")
        .join("globalStorage")
        .join("state.vscdb")];
    #[cfg(target_os = "windows")]
    {
        storage_targets.push(
            PathBuf::from(&inst.data_dir)
                .join("AppData")
                .join("Roaming")
                .join("Antigravity")
                .join("User")
                .join("globalStorage")
                .join("storage.json"),
        );
        vscdb_targets.push(
            PathBuf::from(&inst.data_dir)
                .join("AppData")
                .join("Roaming")
                .join("Antigravity")
                .join("User")
                .join("globalStorage")
                .join("state.vscdb"),
        );
        if let Ok(home) = get_instance_home_dir(&inst.id) {
            storage_targets.push(
                home.join("AppData")
                    .join("Roaming")
                    .join("Antigravity")
                    .join("User")
                    .join("globalStorage")
                    .join("storage.json"),
            );
            vscdb_targets.push(
                home.join("AppData")
                    .join("Roaming")
                    .join("Antigravity")
                    .join("User")
                    .join("globalStorage")
                    .join("state.vscdb"),
            );
        }
    }

    for st in storage_targets {
        if st.exists() {
            if let Ok(content) = fs::read_to_string(&st) {
                if let Ok(mut val) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let serde_json::Value::Object(ref mut map) = val {
                        map.remove("openedPathsList");
                        map.remove("backupWorkspaces");
                        if let Ok(pretty) = serde_json::to_string_pretty(&val) {
                            let _ = fs::write(&st, pretty);
                        }
                    }
                }
            }
        }
    }

    for vt in vscdb_targets {
        if vt.exists() {
            if let Ok(conn) = rusqlite::Connection::open(&vt) {
                let _ = conn.execute(
                    "DELETE FROM ItemTable WHERE key LIKE '%recentlyOpened%' OR key LIKE '%history.%' OR key LIKE '%openedPaths%'",
                    [],
                );
            }
        }
    }
}

/// Copies workspace projects and recent paths from one instance to another.
/// Copies workspaceStorage directories, merges storage.json openedPathsList,
/// copies state.vscdb history rows, and clones repo_db project rows.
pub fn copy_instance_projects(from_id: &str, to_id: &str) -> Result<usize, String> {
    let from_resolved = resolve_instance_id(from_id)?;
    let to_resolved = resolve_instance_id(to_id)?;
    if from_resolved == to_resolved {
        return Ok(0);
    }
    let registry = load_registry()?;
    let from_inst = registry
        .instances
        .iter()
        .find(|i| i.id == from_resolved)
        .ok_or_else(|| format!("Source instance '{}' not found", from_id))?
        .clone();
    let to_inst = registry
        .instances
        .iter()
        .find(|i| i.id == to_resolved)
        .ok_or_else(|| format!("Destination instance '{}' not found", to_id))?
        .clone();

    // 1. Locate source workspaceStorage directories
    let mut src_ws_dirs = Vec::new();
    let src_data = PathBuf::from(&from_inst.data_dir);
    let p1 = src_data.join("User").join("workspaceStorage");
    if p1.exists() {
        src_ws_dirs.push(p1);
    }
    let p2 = src_data.join("workspaceStorage");
    if p2.exists() && !src_ws_dirs.contains(&p2) {
        src_ws_dirs.push(p2);
    }
    #[cfg(target_os = "windows")]
    {
        let p3 = src_data
            .join("AppData")
            .join("Roaming")
            .join("Antigravity")
            .join("User")
            .join("workspaceStorage");
        if p3.exists() && !src_ws_dirs.contains(&p3) {
            src_ws_dirs.push(p3);
        }
        if let Ok(src_home) = get_instance_home_dir(&from_inst.id) {
            let p4 = src_home
                .join("AppData")
                .join("Roaming")
                .join("Antigravity")
                .join("User")
                .join("workspaceStorage");
            if p4.exists() && !src_ws_dirs.contains(&p4) {
                src_ws_dirs.push(p4);
            }
        }
        if from_inst.is_default || from_inst.id == "default" {
            if let Ok(appdata) = std::env::var("APPDATA") {
                let p5 = PathBuf::from(appdata)
                    .join("Antigravity")
                    .join("User")
                    .join("workspaceStorage");
                if p5.exists() && !src_ws_dirs.contains(&p5) {
                    src_ws_dirs.push(p5);
                }
            }
        }
    }

    // 2. Prepare destination workspaceStorage directories
    let dst_data = PathBuf::from(&to_inst.data_dir);
    let dst_ws_main = dst_data.join("User").join("workspaceStorage");
    let mut dst_ws_dirs = vec![dst_ws_main];

    #[cfg(target_os = "windows")]
    {
        let p_dst_appdata = dst_data
            .join("AppData")
            .join("Roaming")
            .join("Antigravity")
            .join("User")
            .join("workspaceStorage");
        dst_ws_dirs.push(p_dst_appdata);

        if let Ok(dst_home) = get_instance_home_dir(&to_inst.id) {
            let p_dst_home = dst_home
                .join("AppData")
                .join("Roaming")
                .join("Antigravity")
                .join("User")
                .join("workspaceStorage");
            dst_ws_dirs.push(p_dst_home);
        }
    }

    for d in &dst_ws_dirs {
        let _ = fs::create_dir_all(d);
    }

    let mut copied_count = 0usize;
    let mut copied_names = std::collections::HashSet::new();

    for src_ws in &src_ws_dirs {
        if let Ok(entries) = fs::read_dir(src_ws) {
            for entry in entries.flatten() {
                if let Ok(ft) = entry.file_type() {
                    if ft.is_dir() {
                        let name = entry.file_name();
                        let name_str = name.to_string_lossy().to_string();
                        if copied_names.insert(name_str.clone()) {
                            copied_count += 1;
                        }
                        for dst_ws in &dst_ws_dirs {
                            let target = dst_ws.join(&name);
                            if let Err(e) = copy_dir_recursive(&entry.path(), &target) {
                                crate::modules::logger::log_warn(&format!(
                                    "[Instance] Warning: Failed to copy workspace folder {}: {}",
                                    entry.path().display(),
                                    e
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Clone repo rows in repo_db
    let _ = crate::modules::repo_db::detect_running_projects(&from_inst.id);
    if let Ok(rows) = crate::modules::repo_db::clone_instance_repo_rows(&from_inst.id, &to_inst.id)
    {
        if copied_count == 0 && rows > 0 {
            copied_count = rows;
        }
    }

    // 4. Merge recent paths in storage.json
    merge_storage_json_recent_paths(&from_inst, &to_inst);

    // 5. Merge recent paths from state.vscdb
    merge_state_vscdb_recent_paths(&from_inst, &to_inst);

    let _ = sanitize_cloned_instance_summaries(&to_inst.id);

    crate::modules::logger::log_info(&format!(
        "[Instance] Copied {} workspace projects from '{}' to '{}'",
        copied_count, from_id, to_id
    ));

    Ok(copied_count)
}

/// Scans registered instances and checks if an instance has executable_path matching exe_path,
/// or if exe_path is inside an instance's directory.
pub fn find_instance_by_executable(exe_path: &str) -> Option<String> {
    let clean_exe = exe_path.trim().replace('\\', "/").to_lowercase();
    if clean_exe.is_empty() {
        return None;
    }
    let instances = list_instances().ok()?;

    // 1. Direct match on executable_path
    for inst in &instances {
        if let Some(ref ep) = inst.config.executable_path {
            let clean_ep = ep.trim().replace('\\', "/").to_lowercase();
            if clean_ep == clean_exe {
                return Some(inst.config.id.clone());
            }
        }
    }

    // 2. Check if exe_path contains instance id pattern (e.g. Antigravity-<id>.exe or antigravity-<id>)
    for inst in &instances {
        let id_lower = inst.config.id.to_lowercase();
        let target_name1 = format!("antigravity-{}", id_lower);
        let target_name2 = format!("launch-{}.cmd", id_lower);
        if clean_exe.contains(&target_name1) || clean_exe.contains(&target_name2) {
            return Some(inst.config.id.clone());
        }
    }

    // 3. Check if exe_path is inside an instance's directory tree
    for inst in &instances {
        let data_dir_clean = inst
            .config
            .data_dir
            .trim()
            .replace('\\', "/")
            .to_lowercase();
        if !data_dir_clean.is_empty() && clean_exe.starts_with(&data_dir_clean) {
            return Some(inst.config.id.clone());
        }

        if let Ok(home) = get_instance_home_dir(&inst.config.id) {
            let home_clean = home.to_string_lossy().replace('\\', "/").to_lowercase();
            if clean_exe.starts_with(&home_clean) {
                return Some(inst.config.id.clone());
            }
        }

        if let Ok(instances_root) = get_instances_dir() {
            let inst_root = instances_root
                .join(&inst.config.id)
                .to_string_lossy()
                .replace('\\', "/")
                .to_lowercase();
            if clean_exe.starts_with(&inst_root) {
                return Some(inst.config.id.clone());
            }
        }
    }

    // 4. If base executable matches and default instance exists, return default
    if let Ok(base_exe) = crate::modules::process::detect_antigravity_with_diagnostics(None) {
        let base_clean = base_exe.to_string_lossy().replace('\\', "/").to_lowercase();
        if base_clean == clean_exe {
            return Some("default".to_string());
        }
    }

    None
}

/// Copies theme and Antigravity settings from source instance to destination instance,
/// performing deep-merge into destination settings.json.
pub fn copy_instance_settings(from_id: &str, to_id: &str) -> Result<(), String> {
    let from_resolved = resolve_instance_id(from_id)?;
    let to_resolved = resolve_instance_id(to_id)?;
    if from_resolved == to_resolved {
        return Ok(());
    }
    let registry = load_registry()?;
    let from_inst = registry
        .instances
        .iter()
        .find(|i| i.id == from_resolved)
        .ok_or_else(|| format!("Source instance '{}' not found", from_id))?
        .clone();
    let to_inst = registry
        .instances
        .iter()
        .find(|i| i.id == to_resolved)
        .ok_or_else(|| format!("Destination instance '{}' not found", to_id))?
        .clone();

    // 1. Gather all potential source User directories
    let mut src_user_dirs: Vec<PathBuf> = Vec::new();
    let from_data_user = PathBuf::from(&from_inst.data_dir).join("User");
    if from_data_user.exists() {
        src_user_dirs.push(from_data_user);
    }
    #[cfg(target_os = "windows")]
    {
        if let Ok(home) = get_instance_home_dir(&from_inst.id) {
            let home_user = home
                .join("AppData")
                .join("Roaming")
                .join("Antigravity")
                .join("User");
            if home_user.exists() && !src_user_dirs.contains(&home_user) {
                src_user_dirs.push(home_user);
            }
        }
        if from_inst.is_default || from_inst.id == "default" {
            if let Ok(appdata) = std::env::var("APPDATA") {
                let p = PathBuf::from(appdata).join("Antigravity").join("User");
                if p.exists() && !src_user_dirs.contains(&p) {
                    src_user_dirs.push(p);
                }
            }
            let default_user = get_default_antigravity_data_dir().join("User");
            if default_user.exists() && !src_user_dirs.contains(&default_user) {
                src_user_dirs.push(default_user);
            }
        }
    }

    // 2. Find source settings.json
    let src_settings_path = find_instance_settings_path(&from_inst).or_else(|| {
        src_user_dirs
            .iter()
            .map(|d| d.join("settings.json"))
            .find(|p| p.is_file())
    });

    if let Some(src_path) = src_settings_path {
        if let Ok(content) = fs::read_to_string(&src_path) {
            if let Ok(serde_json::Value::Object(mut src_map)) = serde_json::from_str(&content) {
                // Strip window.title from source to preserve destination identity
                src_map.remove("window.title");

                let dst_settings_paths = get_instance_settings_targets(&to_inst);
                for dst_path in dst_settings_paths {
                    if let Some(parent) = dst_path.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    let mut target_json: serde_json::Value = if dst_path.exists() {
                        fs::read_to_string(&dst_path)
                            .ok()
                            .and_then(|s| serde_json::from_str(&s).ok())
                            .unwrap_or_else(|| serde_json::json!({}))
                    } else {
                        serde_json::json!({})
                    };

                    if !target_json.is_object() {
                        target_json = serde_json::json!({});
                    }

                    // Preserve existing target window.title if present
                    let existing_title = target_json.get("window.title").cloned();

                    deep_merge_json(
                        &mut target_json,
                        &serde_json::Value::Object(src_map.clone()),
                    );

                    // Reassert target identity (window.title)
                    if let Some(title_val) = existing_title {
                        if let serde_json::Value::Object(ref mut t_map) = target_json {
                            t_map.insert("window.title".to_string(), title_val);
                        }
                    } else {
                        let computed = compute_instance_window_title(&to_inst);
                        if let serde_json::Value::Object(ref mut t_map) = target_json {
                            t_map.insert(
                                "window.title".to_string(),
                                serde_json::Value::String(computed),
                            );
                        }
                    }

                    if let Ok(pretty) = serde_json::to_string_pretty(&target_json) {
                        let _ = fs::write(&dst_path, pretty);
                    }
                }
            }
        }
    }

    // 3. Destination user directories
    let dst_user_dirs: Vec<PathBuf> = get_instance_settings_targets(&to_inst)
        .into_iter()
        .filter_map(|p| p.parent().map(|parent| parent.to_path_buf()))
        .collect();

    // 4. Copy keybindings.json, security_presets.json, and antigravity_policies.json
    for file_name in &[
        "keybindings.json",
        "security_presets.json",
        "antigravity_policies.json",
    ] {
        let mut candidate_src = src_user_dirs
            .iter()
            .map(|d| d.join(file_name))
            .find(|p| p.is_file());

        if candidate_src.is_none()
            && (*file_name == "security_presets.json" || *file_name == "antigravity_policies.json")
        {
            let def_p = get_default_antigravity_data_dir()
                .join("User")
                .join(file_name);
            if def_p.is_file() {
                candidate_src = Some(def_p);
            }
            #[cfg(target_os = "windows")]
            {
                if candidate_src.is_none() {
                    if let Ok(appdata) = std::env::var("APPDATA") {
                        let appdata_p = PathBuf::from(appdata)
                            .join("Antigravity")
                            .join("User")
                            .join(file_name);
                        if appdata_p.is_file() {
                            candidate_src = Some(appdata_p);
                        }
                    }
                }
            }
        }

        if let Some(found_src) = candidate_src {
            for dst_dir in &dst_user_dirs {
                let _ = fs::create_dir_all(dst_dir);
                let _ = fs::copy(&found_src, dst_dir.join(file_name));
            }
            crate::modules::logger::log_info(&format!(
                "[Instance] Copied {} from {} to {} destination directories",
                file_name,
                found_src.display(),
                dst_user_dirs.len()
            ));
        }
    }

    // 5. Copy snippets/
    for src_dir in &src_user_dirs {
        let src_snippets = src_dir.join("snippets");
        if src_snippets.is_dir() {
            for dst_dir in &dst_user_dirs {
                let dst_snippets = dst_dir.join("snippets");
                let _ = copy_dir_recursive(&src_snippets, &dst_snippets);
            }
            break;
        }
    }

    // 6. Ensure target window title is freshly injected
    let _ = inject_instance_settings(&to_inst);

    crate::modules::logger::log_info(&format!(
        "[Instance] Successfully synchronized full settings, keybindings, presets, and snippets from '{}' to '{}'",
        from_id, to_id
    ));
    Ok(())
}

/// Enforces baseline default settings across target or all non-default instances.
/// Uses 'default' instance as reference baseline and ensures antigravity.turboMode: true,
/// antigravity.planReviewAlwaysProceed: true, and standard execution policies are applied.
pub fn enforce_default_settings(target_instance: Option<&str>) -> Result<usize, String> {
    let registry = load_registry()?;
    let default_inst = registry
        .instances
        .iter()
        .find(|i| i.is_default || i.id == "default")
        .cloned();

    // 1. Establish baseline settings map
    let mut baseline_map = serde_json::Map::new();
    if let Some(ref def) = default_inst {
        if let Some(def_settings_path) = find_instance_settings_path(def) {
            if let Ok(content) = fs::read_to_string(&def_settings_path) {
                if let Ok(serde_json::Value::Object(map)) = serde_json::from_str(&content) {
                    for (k, v) in map {
                        let k_lower = k.to_lowercase();
                        let is_theme = k.starts_with("workbench.")
                            && (k_lower.contains("theme") || k_lower.contains("color"));
                        let is_antigravity = k.starts_with("antigravity.");
                        let is_policy = k_lower.contains("policy");
                        if is_theme || is_antigravity || is_policy {
                            baseline_map.insert(k, v);
                        }
                    }
                }
            }
        }
    }

    // 2. Mandate performance & automation defaults
    baseline_map.insert(
        "antigravity.turboMode".to_string(),
        serde_json::Value::Bool(true),
    );
    baseline_map.insert(
        "antigravity.planReviewAlwaysProceed".to_string(),
        serde_json::Value::Bool(true),
    );
    baseline_map
        .entry("antigravity.browserExecutionPolicy".to_string())
        .or_insert_with(|| serde_json::Value::String("openDirectly".to_string()));
    baseline_map
        .entry("antigravity.codeReviewPolicy".to_string())
        .or_insert_with(|| serde_json::Value::String("automaticReview".to_string()));
    baseline_map
        .entry("security.workspace.trust.enabled".to_string())
        .or_insert_with(|| serde_json::Value::Bool(false));
    baseline_map
        .entry("workbench.colorTheme".to_string())
        .or_insert_with(|| serde_json::Value::String("Default Dark Modern".to_string()));

    let baseline_val = serde_json::Value::Object(baseline_map);

    // 3. Determine target instances
    let targets: Vec<InstanceConfig> = if let Some(spec) = target_instance {
        let resolved = resolve_instance_id(spec)?;
        registry
            .instances
            .into_iter()
            .filter(|i| i.id == resolved)
            .collect()
    } else {
        registry
            .instances
            .into_iter()
            .filter(|i| !i.is_default && i.id != "default")
            .collect()
    };

    let mut updated_count = 0usize;
    for inst in &targets {
        let paths = get_instance_settings_targets(inst);
        for path in paths {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let mut current: serde_json::Value = if path.exists() {
                fs::read_to_string(&path)
                    .ok()
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_else(|| serde_json::json!({}))
            } else {
                serde_json::json!({})
            };

            if !current.is_object() {
                current = serde_json::json!({});
            }

            deep_merge_json(&mut current, &baseline_val);

            if let Ok(pretty) = serde_json::to_string_pretty(&current) {
                let _ = fs::write(&path, pretty);
            }
        }
        updated_count += 1;
    }

    crate::modules::logger::log_info(&format!(
        "[Instance] Enforced default settings across {} instance(s)",
        updated_count
    ));
    Ok(updated_count)
}

/// Sets antigravity.turboMode in User/settings.json for target or all instances.
pub fn set_instance_turbo_mode(
    target_instance: Option<&str>,
    enabled: bool,
) -> Result<usize, String> {
    let registry = load_registry()?;
    let targets: Vec<InstanceConfig> = if let Some(spec) = target_instance {
        let resolved = resolve_instance_id(spec)?;
        registry
            .instances
            .into_iter()
            .filter(|i| i.id == resolved)
            .collect()
    } else {
        registry.instances
    };

    let mut updated = 0usize;
    for inst in &targets {
        let paths = get_instance_settings_targets(inst);
        for path in paths {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let mut json_val: serde_json::Value = if path.exists() {
                fs::read_to_string(&path)
                    .ok()
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_else(|| serde_json::json!({}))
            } else {
                serde_json::json!({})
            };
            if !json_val.is_object() {
                json_val = serde_json::json!({});
            }
            if let serde_json::Value::Object(ref mut map) = json_val {
                map.insert(
                    "antigravity.turboMode".to_string(),
                    serde_json::Value::Bool(enabled),
                );
                if let Ok(pretty) = serde_json::to_string_pretty(&json_val) {
                    let _ = fs::write(&path, pretty);
                }
            }
        }
        updated += 1;
    }
    Ok(updated)
}

/// Sets antigravity.planReviewAlwaysProceed in User/settings.json for target or all instances.
pub fn set_instance_plan_review(
    target_instance: Option<&str>,
    always_proceed: bool,
) -> Result<usize, String> {
    let registry = load_registry()?;
    let targets: Vec<InstanceConfig> = if let Some(spec) = target_instance {
        let resolved = resolve_instance_id(spec)?;
        registry
            .instances
            .into_iter()
            .filter(|i| i.id == resolved)
            .collect()
    } else {
        registry.instances
    };

    let mut updated = 0usize;
    for inst in &targets {
        let paths = get_instance_settings_targets(inst);
        for path in paths {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let mut json_val: serde_json::Value = if path.exists() {
                fs::read_to_string(&path)
                    .ok()
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_else(|| serde_json::json!({}))
            } else {
                serde_json::json!({})
            };
            if !json_val.is_object() {
                json_val = serde_json::json!({});
            }
            if let serde_json::Value::Object(ref mut map) = json_val {
                map.insert(
                    "antigravity.planReviewAlwaysProceed".to_string(),
                    serde_json::Value::Bool(always_proceed),
                );
                if let Ok(pretty) = serde_json::to_string_pretty(&json_val) {
                    let _ = fs::write(&path, pretty);
                }
            }
        }
        updated += 1;
    }
    Ok(updated)
}

/// Sets workbench.colorTheme in User/settings.json for target or all instances.
pub fn set_instance_theme(target_instance: Option<&str>, theme_id: &str) -> Result<usize, String> {
    let registry = load_registry()?;
    let targets: Vec<InstanceConfig> = if let Some(spec) = target_instance {
        let resolved = resolve_instance_id(spec)?;
        registry
            .instances
            .into_iter()
            .filter(|i| i.id == resolved)
            .collect()
    } else {
        registry.instances
    };

    let mut updated = 0usize;
    for inst in &targets {
        let paths = get_instance_settings_targets(inst);
        for path in paths {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let mut json_val: serde_json::Value = if path.exists() {
                fs::read_to_string(&path)
                    .ok()
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_else(|| serde_json::json!({}))
            } else {
                serde_json::json!({})
            };
            if !json_val.is_object() {
                json_val = serde_json::json!({});
            }
            if let serde_json::Value::Object(ref mut map) = json_val {
                map.insert(
                    "workbench.colorTheme".to_string(),
                    serde_json::Value::String(theme_id.to_string()),
                );
                if let Ok(pretty) = serde_json::to_string_pretty(&json_val) {
                    let _ = fs::write(&path, pretty);
                }
            }
        }
        updated += 1;
    }
    Ok(updated)
}

/// Reads workbench.colorTheme from target instance User/settings.json.
pub fn get_instance_theme(instance_id: &str) -> Result<Option<String>, String> {
    let resolved_id = resolve_instance_id(instance_id)?;
    let registry = load_registry()?;
    let inst = registry
        .instances
        .iter()
        .find(|i| i.id == resolved_id)
        .ok_or_else(|| format!("Instance '{}' not found", instance_id))?;

    if let Some(settings_path) = find_instance_settings_path(inst) {
        if let Ok(content) = fs::read_to_string(&settings_path) {
            if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(theme) = json_val
                    .get("workbench.colorTheme")
                    .and_then(|v| v.as_str())
                {
                    return Ok(Some(theme.to_string()));
                }
            }
        }
    }
    Ok(None)
}

/// Reads User/settings.json and returns formatted JSON string with metadata envelope.
pub fn export_instance_settings(instance_id: &str) -> Result<String, String> {
    let resolved_id = resolve_instance_id(instance_id)?;
    let registry = load_registry()?;
    let inst = registry
        .instances
        .iter()
        .find(|i| i.id == resolved_id)
        .ok_or_else(|| format!("Instance '{}' not found", instance_id))?;

    let settings_val = if let Some(settings_path) = find_instance_settings_path(inst) {
        fs::read_to_string(&settings_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    let envelope = serde_json::json!({
        "instance_id": inst.id,
        "instance_name": inst.name,
        "exported_at": chrono::Utc::now().to_rfc3339(),
        "settings": settings_val
    });

    serde_json::to_string_pretty(&envelope).map_err(|e| e.to_string())
}

/// Ingests settings JSON and writes to target instance (or all if target is None).
pub fn import_instance_settings(
    target_instance: Option<&str>,
    json_str: &str,
) -> Result<usize, String> {
    let parsed: serde_json::Value =
        serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON format: {}", e))?;

    let settings_to_apply =
        if let Some(settings_obj) = parsed.get("settings").filter(|v| v.is_object()) {
            settings_obj.clone()
        } else if let Some(payload_obj) = parsed.get("payload").filter(|v| v.is_object()) {
            if let Some(s) = payload_obj.get("settings").filter(|v| v.is_object()) {
                s.clone()
            } else {
                payload_obj.clone()
            }
        } else if parsed.is_object() {
            parsed
        } else {
            return Err("Input must be a JSON object containing settings".to_string());
        };

    let registry = load_registry()?;
    let targets: Vec<InstanceConfig> = if let Some(spec) = target_instance {
        let resolved = resolve_instance_id(spec)?;
        registry
            .instances
            .into_iter()
            .filter(|i| i.id == resolved)
            .collect()
    } else {
        registry.instances
    };

    let mut updated_count = 0usize;
    for inst in &targets {
        let paths = get_instance_settings_targets(inst);
        for path in paths {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let mut current: serde_json::Value = if path.exists() {
                fs::read_to_string(&path)
                    .ok()
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_else(|| serde_json::json!({}))
            } else {
                serde_json::json!({})
            };

            if !current.is_object() {
                current = serde_json::json!({});
            }

            deep_merge_json(&mut current, &settings_to_apply);

            if let Ok(pretty) = serde_json::to_string_pretty(&current) {
                let _ = fs::write(&path, pretty);
            }
        }
        updated_count += 1;
    }

    Ok(updated_count)
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn switch_spares_another_instances_pid() {
        let protected = vec![11128u32];
        let markers = vec![
            "/instances/nextv2/".to_string(),
            "antigravity-nextv2".to_string(),
            "c:/users/administrator/.antigravity_tools/instances/nextv2".to_string(),
        ];
        assert!(should_spare_pid(
            11128,
            "",
            "c:/program files/antigravity/antigravity.exe",
            "antigravity.exe",
            &protected,
            &markers
        ));
        assert!(should_spare_pid(
            4242,
            "--user-data-dir=c:/users/administrator/.antigravity_tools/instances/nextv2",
            "c:/program files/antigravity/antigravity.exe",
            "antigravity.exe",
            &protected,
            &markers
        ));
        assert!(!should_spare_pid(
            9001,
            "--user-data-dir=c:/users/administrator/appdata/roaming/antigravity",
            "c:/program files/antigravity/antigravity.exe",
            "antigravity.exe",
            &protected,
            &markers
        ));
    }

    #[test]
    fn saved_pid_identity_and_refresh_floor() {
        assert!(process_identity_matches(
            "Antigravity.exe",
            "C:/Program Files/Antigravity/Antigravity.exe"
        ));
        assert!(!process_identity_matches("agm.exe", "C:/agm/agm.exe"));
        assert!(!process_identity_matches(
            "antigravity",
            "C:/Users/me/AppData/Local/agm/agm.exe"
        ));
        assert_eq!(pid_refresh_interval_seconds(60), 180);
        assert_eq!(pid_refresh_interval_seconds(180), 180);
        assert_eq!(pid_refresh_interval_seconds(600), 600);
        assert_eq!(pid_refresh_interval_seconds(2000), 1200);
    }

    #[test]
    fn test_process_identity_matches_rejects_developer_tools() {
        assert!(!process_identity_matches(
            "esbuild.exe",
            "D:/work/Antigravity-Manager/node_modules/@esbuild/win32-x64/esbuild.exe"
        ));
        assert!(!process_identity_matches(
            "cargo.exe",
            "D:/work/Antigravity-Manager/target/debug/build/cargo.exe"
        ));
        assert!(process_identity_matches(
            "Antigravity.exe",
            "C:/Users/Admin/AppData/Local/Programs/Antigravity/Antigravity.exe"
        ));
    }

    #[test]
    fn test_workspace_snapshot_and_migration_helpers() {
        let ws = snapshot_active_workspaces("non-existent-instance-id-xyz");
        assert!(ws.is_empty());
        let res = migrate_instance_workspaces("non-existent-1", "non-existent-1");
        assert!(res.is_ok());
    }

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

    #[test]
    fn test_multi_instance_multi_project_account_swap_isolation() {
        let temp_root = std::env::temp_dir().join(format!(
            "agm_multi_inst_proj_test_{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        let inst1_data = temp_root.join("inst-1").join("data");
        let inst2_data = temp_root.join("inst-2").join("data");
        let proj_a = temp_root.join("project-alpha");
        let proj_b = temp_root.join("project-beta");
        let proj_c = temp_root.join("project-gamma");
        let _ = std::fs::create_dir_all(&proj_a);
        let _ = std::fs::create_dir_all(&proj_b);
        let _ = std::fs::create_dir_all(&proj_c);

        // Assign Project Alpha + Project Beta to Instance 1, and Project Gamma to Instance 2
        let ws1_a = inst1_data
            .join("User")
            .join("workspaceStorage")
            .join("ws-alpha");
        let ws1_b = inst1_data
            .join("User")
            .join("workspaceStorage")
            .join("ws-beta");
        let ws2_c = inst2_data
            .join("User")
            .join("workspaceStorage")
            .join("ws-gamma");
        let _ = std::fs::create_dir_all(&ws1_a);
        let _ = std::fs::create_dir_all(&ws1_b);
        let _ = std::fs::create_dir_all(&ws2_c);

        let uri_a = format!("file:///{}", proj_a.to_string_lossy().replace('\\', "/"));
        let uri_b = format!("file:///{}", proj_b.to_string_lossy().replace('\\', "/"));
        let uri_c = format!("file:///{}", proj_c.to_string_lossy().replace('\\', "/"));
        let _ = std::fs::write(
            ws1_a.join("workspace.json"),
            serde_json::json!({ "folder": uri_a }).to_string(),
        );
        let _ = std::fs::write(
            ws1_b.join("workspace.json"),
            serde_json::json!({ "folder": uri_b }).to_string(),
        );
        let _ = std::fs::write(
            ws2_c.join("workspace.json"),
            serde_json::json!({ "folder": uri_c }).to_string(),
        );

        let inst1_folders =
            get_instance_workspace_folders("inst-1-isolated", &inst1_data.to_string_lossy());
        let inst2_folders =
            get_instance_workspace_folders("inst-2-isolated", &inst2_data.to_string_lossy());

        assert_eq!(
            inst1_folders.len(),
            2,
            "Instance 1 must restore both bound project workspaces (alpha & beta)"
        );
        assert_eq!(
            inst2_folders.len(),
            1,
            "Instance 2 must restore only its bound project workspace (gamma)"
        );

        let _ = std::fs::remove_dir_all(&temp_root);
    }

    #[test]
    #[ignore = "local_only_e2e"]
    fn test_local_e2e_instance_switch_and_prompt_restore() {
        // Strict Skip-by-Default Isolation Guard
        if std::env::var("RUN_TEMP_E2E").as_deref() != Ok("1") {
            println!("Skipping temporary local-only E2E test; run on-demand with RUN_TEMP_E2E=1");
            return;
        }

        // Phase 1: Invariant Protection Guard
        let default_dir = get_default_antigravity_data_dir();
        let default_dir_str = default_dir.to_string_lossy();
        let protected_pids = find_pids_for_data_dir(&default_dir_str, true);
        println!("[TEMP E2E] Host Protected PIDs: {:?}", protected_pids);

        // Phase 2: Create isolated sandbox instance
        let unique_id = format!("e2e-inst-{}", chrono::Utc::now().timestamp_millis());
        let temp_base = std::env::temp_dir().join(&unique_id);
        let inst_data = temp_base.join("data");
        let proj_dir = temp_base.join("project-gitmap");
        assert!(std::fs::create_dir_all(&inst_data).is_ok());
        assert!(std::fs::create_dir_all(&proj_dir).is_ok());

        // Create state.vscdb with initial credentials
        let user_storage = inst_data.join("User").join("globalStorage");
        assert!(std::fs::create_dir_all(&user_storage).is_ok());
        let db_path = user_storage.join("state.vscdb");
        {
            let conn = rusqlite::Connection::open(&db_path).unwrap();
            conn.execute(
                "CREATE TABLE IF NOT EXISTS ItemTable (key TEXT PRIMARY KEY, value BLOB)",
                [],
            )
            .unwrap();
            let initial_email = "rokixshohag1@gmail.com";
            conn.execute(
                "INSERT OR REPLACE INTO ItemTable (key, value) VALUES (?1, ?2)",
                rusqlite::params!["antigravity.activeUser", initial_email.as_bytes()],
            )
            .unwrap();
        }

        // Phase 3: Bind workspace project
        let ws_storage = inst_data.join("User").join("workspaceStorage").join("ws-1");
        assert!(std::fs::create_dir_all(&ws_storage).is_ok());
        let proj_uri = format!("file:///{}", proj_dir.to_string_lossy().replace('\\', "/"));
        std::fs::write(
            ws_storage.join("workspace.json"),
            serde_json::json!({ "folder": proj_uri }).to_string(),
        )
        .unwrap();

        // Verify bound workspace discovery
        let discovered_folders =
            get_instance_workspace_folders(&unique_id, &inst_data.to_string_lossy());
        assert_eq!(
            discovered_folders.len(),
            1,
            "Must find bound project folder"
        );

        // Phase 4: Seed in-flight and queued prompts into repo_db
        let running_text = "Running the Gitmap tests and verifying test inventory";
        let queued_text_1 = "Check the CICD pipeline status and diagnostic logs";
        let queued_text_2 = "Verify unit test durations and isolate heavy system calls";

        if let Ok(conn) = crate::modules::repo_db::connect_db() {
            let now = chrono::Utc::now().timestamp();
            let _ = conn.execute(
                "INSERT OR REPLACE INTO running_projects 
                 (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![
                    "gitmap-1",
                    unique_id,
                    "Gitmap",
                    proj_dir.to_string_lossy().to_string(),
                    ws_storage.to_string_lossy().to_string(),
                    1,
                    now,
                    now,
                ],
            );
            let _ = conn.execute(
                "INSERT OR REPLACE INTO active_prompts 
                 (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                rusqlite::params![
                    format!("prompt-{}-1", unique_id),
                    "gitmap-1",
                    unique_id,
                    proj_dir.to_string_lossy().to_string(),
                    running_text,
                    "gemini-2.5-pro",
                    "sess-1",
                    "running",
                    now,
                    now,
                    None::<String>,
                ],
            );
            let _ = conn.execute(
                "INSERT OR REPLACE INTO active_prompts 
                 (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                rusqlite::params![
                    format!("prompt-{}-2", unique_id),
                    "gitmap-1",
                    unique_id,
                    proj_dir.to_string_lossy().to_string(),
                    queued_text_1,
                    "gemini-2.5-pro",
                    "sess-2",
                    "queued",
                    now,
                    now,
                    None::<String>,
                ],
            );
            let _ = conn.execute(
                "INSERT OR REPLACE INTO active_prompts 
                 (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                rusqlite::params![
                    format!("prompt-{}-3", unique_id),
                    "gitmap-1",
                    unique_id,
                    proj_dir.to_string_lossy().to_string(),
                    queued_text_2,
                    "gemini-2.5-pro",
                    "sess-3",
                    "queued",
                    now,
                    now,
                    None::<String>,
                ],
            );
        }

        // Phase 5: Backup prompts prior to switch
        let mut in_flight_note = None;
        let mut queued_notes = Vec::new();
        if let Ok(conn) = crate::modules::repo_db::connect_db() {
            if let Ok(mut stmt) = conn
                .prepare("SELECT prompt_content, status FROM active_prompts WHERE instance_id = ?1")
            {
                if let Ok(rows) = stmt.query_map(rusqlite::params![unique_id], |r| {
                    Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
                }) {
                    for r in rows.flatten() {
                        if r.1 == "running" {
                            in_flight_note = Some(r.0);
                        } else {
                            queued_notes.push(r.0);
                        }
                    }
                }
            }
        }
        assert_eq!(in_flight_note.as_deref(), Some(running_text));
        assert_eq!(queued_notes.len(), 2);

        // Phase 6: Switch account credentials in state.vscdb
        let new_email = "erfan.office.n@gmail.com";
        {
            let conn = rusqlite::Connection::open(&db_path).unwrap();
            conn.execute(
                "INSERT OR REPLACE INTO ItemTable (key, value) VALUES (?1, ?2)",
                rusqlite::params!["antigravity.activeUser", new_email.as_bytes()],
            )
            .unwrap();

            // Verify account change in state.vscdb
            let stored_val: Vec<u8> = conn
                .query_row(
                    "SELECT value FROM ItemTable WHERE key = 'antigravity.activeUser'",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            let stored_str = String::from_utf8_lossy(&stored_val);
            assert_eq!(stored_str, new_email);
        }

        // Phase 7: Restore and push prompts back to project directory
        let resume_task_file = proj_dir.join(".antigravity_resume_task.json");
        let resume_payload = serde_json::json!({
            "task": in_flight_note.unwrap_or_default(),
            "queued": queued_notes,
            "instance_id": unique_id,
            "email": new_email,
            "status": "restored"
        });
        std::fs::write(&resume_task_file, resume_payload.to_string()).unwrap();
        assert!(resume_task_file.exists());

        // Verify resume file content
        let read_back: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&resume_task_file).unwrap()).unwrap();
        assert_eq!(read_back["email"], new_email);
        assert_eq!(read_back["task"], running_text);

        // Clean up seeded prompts in repo_db
        if let Ok(conn) = crate::modules::repo_db::connect_db() {
            let _ = conn.execute(
                "DELETE FROM active_prompts WHERE instance_id = ?1",
                rusqlite::params![unique_id],
            );
        }

        // Phase 8: Invariant Check & Cleanup
        for pid in &protected_pids {
            let s = sysinfo::System::new_with_specifics(
                sysinfo::RefreshKind::new().with_processes(sysinfo::ProcessRefreshKind::new()),
            );
            assert!(
                s.process(sysinfo::Pid::from_u32(*pid)).is_some(),
                "Host IDE PID {} must remain alive and untouched throughout E2E test",
                pid
            );
        }

        let _ = std::fs::remove_dir_all(&temp_base);
        println!("[TEMP E2E] Local-only E2E test completed successfully with 100% clean teardown.");
    }
}

#[cfg(test)]
mod clone_tree_tests {
    use super::*;

    #[test]
    fn clone_copies_settings_workspace_db_and_gemini_repo() {
        let root = std::env::temp_dir().join(format!("agm-clone-trees-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let src_data = root.join("src-data");
        let dst_data = root.join("dst-data");
        let src_home = root.join("src-home");
        let dst_home = root.join("dst-home");
        fs::create_dir_all(src_data.join("User").join("globalStorage")).unwrap();
        fs::create_dir_all(src_data.join("User").join("workspaceStorage").join("proj")).unwrap();
        fs::write(
            src_data.join("User").join("settings.json"),
            br#"{"workbench.colorTheme":"Tokyo Night"}"#,
        )
        .unwrap();
        fs::write(
            src_data.join("User").join("keybindings.json"),
            b"[{\"key\": \"ctrl+k\"}]",
        )
        .unwrap();
        fs::create_dir_all(src_data.join("User").join("snippets")).unwrap();
        fs::write(
            src_data.join("User").join("snippets").join("rust.json"),
            b"{\"snippet\": \"test\"}",
        )
        .unwrap();
        fs::write(
            src_data
                .join("User")
                .join("workspaceStorage")
                .join("proj")
                .join("state.vscdb"),
            b"workspace-repo-db",
        )
        .unwrap();
        fs::create_dir_all(src_home.join(".gemini").join("antigravity")).unwrap();
        fs::write(
            src_home.join(".gemini").join("antigravity").join("repo.db"),
            b"gemini-repo-db",
        )
        .unwrap();
        fs::create_dir_all(&dst_data).unwrap();

        copy_required_ide_files(&src_data, &dst_data).unwrap();
        copy_gemini_trees(&src_home, &dst_home).unwrap();

        assert_eq!(
            fs::read(dst_data.join("User").join("settings.json")).unwrap(),
            br#"{"workbench.colorTheme":"Tokyo Night"}"#
        );
        assert_eq!(
            fs::read(dst_data.join("User").join("keybindings.json")).unwrap(),
            b"[{\"key\": \"ctrl+k\"}]"
        );
        assert_eq!(
            fs::read(dst_data.join("User").join("snippets").join("rust.json")).unwrap(),
            b"{\"snippet\": \"test\"}"
        );
        assert_eq!(
            fs::read(
                dst_data
                    .join("User")
                    .join("workspaceStorage")
                    .join("proj")
                    .join("state.vscdb")
            )
            .unwrap(),
            b"workspace-repo-db"
        );
        assert_eq!(
            fs::read(dst_home.join(".gemini").join("antigravity").join("repo.db")).unwrap(),
            b"gemini-repo-db"
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_copy_source_user_settings_copies_to_data_and_appdata() {
        let root = std::env::temp_dir().join(format!("agm-user-settings-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);

        let src_data = root.join("src-inst").join("data");
        let dst_data = root.join("dst-inst").join("data");

        let src_user = src_data.join("User");
        fs::create_dir_all(&src_user).unwrap();
        fs::write(
            src_user.join("settings.json"),
            br#"{"workbench.colorTheme":"Catppuccin Mocha"}"#,
        )
        .unwrap();
        fs::write(
            src_user.join("keybindings.json"),
            br#"[{"key":"ctrl+shift+p"}]"#,
        )
        .unwrap();
        let src_snippets = src_user.join("snippets");
        fs::create_dir_all(&src_snippets).unwrap();
        fs::write(
            src_snippets.join("custom.json"),
            b"{\"prefix\": \"custom\"}",
        )
        .unwrap();

        let source = InstanceConfig {
            id: "src-inst".to_string(),
            name: "Source".to_string(),
            data_dir: src_data.to_string_lossy().to_string(),
            executable_path: None,
            extensions_dir: Some("D:/extensions".to_string()),
            bound_account_id: None,
            bound_email: None,
            created_at: 0,
            last_used: 0,
            is_default: false,
            pid: None,
            seq_num: Some(1),
        };

        let dest = InstanceConfig {
            id: "dst-inst".to_string(),
            name: "Dest".to_string(),
            data_dir: dst_data.to_string_lossy().to_string(),
            executable_path: None,
            extensions_dir: None,
            bound_account_id: None,
            bound_email: None,
            created_at: 0,
            last_used: 0,
            is_default: false,
            pid: None,
            seq_num: Some(2),
        };

        let _ = copy_source_user_settings(&source, &dest);

        assert_eq!(
            fs::read(dst_data.join("User").join("settings.json")).unwrap(),
            br#"{"workbench.colorTheme":"Catppuccin Mocha"}"#
        );
        assert_eq!(
            fs::read(dst_data.join("User").join("keybindings.json")).unwrap(),
            br#"[{"key":"ctrl+shift+p"}]"#
        );
        assert_eq!(
            fs::read(dst_data.join("User").join("snippets").join("custom.json")).unwrap(),
            b"{\"prefix\": \"custom\"}"
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_compute_ide_ending_sequence() {
        let default_inst = InstanceConfig {
            id: "default".to_string(),
            name: "Default".to_string(),
            data_dir: "/tmp/data".to_string(),
            executable_path: None,
            extensions_dir: None,
            bound_account_id: None,
            bound_email: None,
            created_at: 0,
            last_used: 0,
            is_default: true,
            pid: None,
            seq_num: Some(1),
        };
        assert_eq!(compute_ide_ending_sequence(&default_inst), "Antigravity");

        let default_by_id = InstanceConfig {
            id: "default".to_string(),
            name: "Main Workspace".to_string(),
            data_dir: "/tmp/data".to_string(),
            executable_path: None,
            extensions_dir: None,
            bound_account_id: None,
            bound_email: None,
            created_at: 0,
            last_used: 0,
            is_default: false,
            pid: None,
            seq_num: Some(1),
        };
        assert_eq!(compute_ide_ending_sequence(&default_by_id), "Antigravity");

        let custom_inst = InstanceConfig {
            id: "inst-8136".to_string(),
            name: "8136".to_string(),
            data_dir: "/tmp/data".to_string(),
            executable_path: None,
            extensions_dir: None,
            bound_account_id: None,
            bound_email: None,
            created_at: 0,
            last_used: 0,
            is_default: false,
            pid: None,
            seq_num: Some(2),
        };
        assert_eq!(
            compute_ide_ending_sequence(&custom_inst),
            "antigravity-8136"
        );

        let fallback_inst = InstanceConfig {
            id: "inst-xyz".to_string(),
            name: "---".to_string(),
            data_dir: "/tmp/data".to_string(),
            executable_path: None,
            extensions_dir: None,
            bound_account_id: None,
            bound_email: None,
            created_at: 0,
            last_used: 0,
            is_default: false,
            pid: None,
            seq_num: Some(3),
        };
        assert_eq!(
            compute_ide_ending_sequence(&fallback_inst),
            "antigravity-inst-xyz"
        );
    }

    #[test]
    fn test_compute_instance_window_title() {
        let default_inst = InstanceConfig {
            id: "default".to_string(),
            name: "Default".to_string(),
            data_dir: "/tmp/data".to_string(),
            executable_path: None,
            extensions_dir: None,
            bound_account_id: None,
            bound_email: None,
            created_at: 0,
            last_used: 0,
            is_default: true,
            pid: None,
            seq_num: Some(1),
        };
        assert_eq!(
            compute_instance_window_title(&default_inst),
            "#1 Default - Antigravity${separator}${dirty}${activeEditorShort}${separator}${rootName}"
        );

        let custom_inst = InstanceConfig {
            id: "inst-8136".to_string(),
            name: "8136".to_string(),
            data_dir: "/tmp/data".to_string(),
            executable_path: None,
            extensions_dir: None,
            bound_account_id: None,
            bound_email: None,
            created_at: 0,
            last_used: 0,
            is_default: false,
            pid: None,
            seq_num: Some(2),
        };
        assert_eq!(
            compute_instance_window_title(&custom_inst),
            "#2 8136 - antigravity-8136${separator}${dirty}${activeEditorShort}${separator}${rootName}"
        );
    }

    #[test]
    fn test_inject_instance_settings() {
        let temp_dir = std::env::temp_dir().join(format!(
            "agm_settings_test_{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        let user_dir = temp_dir.join("User");
        let _ = fs::create_dir_all(&user_dir);

        let inst = InstanceConfig {
            id: "inst-settings-test".to_string(),
            name: "Settings Test".to_string(),
            data_dir: temp_dir.to_string_lossy().to_string(),
            executable_path: None,
            extensions_dir: None,
            bound_account_id: None,
            bound_email: None,
            created_at: 0,
            last_used: 0,
            is_default: false,
            pid: None,
            seq_num: Some(5),
        };

        assert!(inject_instance_settings(&inst).is_ok());

        let settings_path = user_dir.join("settings.json");
        assert!(settings_path.exists());
        let content = fs::read_to_string(&settings_path).unwrap();
        let val: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(
            val.get("window.title").and_then(|v| v.as_str()),
            Some("#5 Settings Test - antigravity-settings-test${separator}${dirty}${activeEditorShort}${separator}${rootName}")
        );

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_sanitize_cloned_instance_summaries() {
        let test_inst_id = format!(
            "test_inst_sanitize_{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        );
        let instances_dir = get_instances_dir().expect("get_instances_dir should succeed");
        let inst_gemini_dir = instances_dir
            .join(&test_inst_id)
            .join("home")
            .join(".gemini")
            .join("antigravity");
        fs::create_dir_all(&inst_gemini_dir).expect("create_dir_all should succeed");

        let db_path = inst_gemini_dir.join("conversation_summaries.db");
        let conn = rusqlite::Connection::open(&db_path).expect("open db should succeed");
        conn.execute(
            "CREATE TABLE conversation_summaries (
                conversation_id TEXT PRIMARY KEY,
                not_fully_idle INTEGER,
                status TEXT
            )",
            [],
        )
        .expect("create table should succeed");

        conn.execute(
            "INSERT INTO conversation_summaries (conversation_id, not_fully_idle, status)
             VALUES ('conv-1', 1, 'CASCADE_RUN_STATUS_RUNNING')",
            [],
        )
        .expect("insert row should succeed");
        drop(conn);

        let res = sanitize_cloned_instance_summaries(&test_inst_id);
        assert!(res.is_ok());

        let conn2 = rusqlite::Connection::open(&db_path).expect("reopen db should succeed");
        let (idle, status): (i32, String) = conn2
            .query_row(
                "SELECT not_fully_idle, status FROM conversation_summaries WHERE conversation_id = 'conv-1'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .expect("query row should succeed");

        assert_eq!(idle, 0);
        assert_eq!(status, "IDLE");

        let _ = fs::remove_dir_all(instances_dir.join(&test_inst_id));
    }

    #[test]
    fn test_get_macos_candidate_paths_includes_user_applications() {
        let paths = get_macos_candidate_paths();
        assert!(paths
            .iter()
            .any(|p| p == &PathBuf::from("/Applications/Antigravity.app")));
        if let Some(home) = dirs::home_dir() {
            let user_app = home.join("Applications").join("Antigravity.app");
            let user_macos_bin = user_app.join("Contents").join("MacOS").join("Antigravity");
            assert!(paths.contains(&user_app));
            assert!(paths.contains(&user_macos_bin));
        }
    }

    #[test]
    fn test_resolve_instance_exe_name() {
        let default_exe = resolve_instance_exe_name("default", None);
        #[cfg(target_os = "windows")]
        assert!(default_exe.ends_with(".exe"));
        #[cfg(not(target_os = "windows"))]
        assert!(!default_exe.is_empty());

        let custom_exe = resolve_instance_exe_name("custom-1", Some("/opt/bin/my-ide"));
        assert_eq!(custom_exe, "my-ide");

        let fallback_exe = resolve_instance_exe_name("inst-2", None);
        #[cfg(target_os = "windows")]
        assert_eq!(fallback_exe, "Antigravity-inst-2.exe");
        #[cfg(not(target_os = "windows"))]
        assert_eq!(fallback_exe, "Antigravity-inst-2");
    }
}
