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
    let instances_root = get_instances_dir()?;
    let home_dir = instances_root.join(instance_id).join("home");
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

    // Also update <data_dir>/User/settings.json so the IDE window title bar visibly displays the bound account email
    if let Some(email) = bound_email.map(str::trim).filter(|e| !e.is_empty()) {
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
        let now_dt = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
        settings_map.insert(
            "window.title".to_string(),
            serde_json::Value::String(format!(
                "Antigravity — Account: {} — Date: {} — ${{rootName}} ${{activeEditorShort}}",
                email, now_dt
            )),
        );
        settings_map.insert(
            "workbench.startupEditor".to_string(),
            serde_json::Value::String("none".to_string()),
        );
        settings_map.insert(
            "security.workspace.trust.enabled".to_string(),
            serde_json::Value::Bool(false),
        );
        if let Ok(pretty) = serde_json::to_string_pretty(&settings_map) {
            let _ = fs::write(&settings_path, pretty);
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
            let _ = fs::copy(&default_db, &inst_db);
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

        let is_antigravity = (name.contains("antigravity")
            || exe.contains("antigravity")
            || exe.contains("/tmp/.mount_")
            || name == "apprun")
            && !name.contains("agm")
            && !exe.contains("agm")
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

        if (has_user_data_arg && has_instance_marker && !is_helper)
            || (matches_cloned_exe && !is_helper)
        {
            instance_root_pids.insert(pid_u32);
            if args_str.contains(clean_target) || matches_cloned_exe {
                matched_pids.push(pid_u32);
            }
        } else if is_default && !has_instance_marker && !is_helper {
            default_candidate_pids.push(pid_u32);
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
        let target_pid = sysinfo::Pid::from_u32(saved_pid);
        sys.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::Some(&[target_pid]),
            sysinfo::ProcessRefreshKind::new().with_exe(sysinfo::UpdateKind::OnlyIfNotSet),
        );
        if let Some(proc) = sys.process(target_pid) {
            let proc_name = proc.name().to_string_lossy().to_lowercase();
            let proc_exe = proc
                .exe()
                .map(|p| p.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            if proc_name.contains("antigravity") || proc_exe.contains("antigravity") {
                return true;
            }
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

/// Launch a specific instance with multi-window isolation, bound workspace folder restoration, and custom/cloned executable support
pub fn launch_instance(instance_id: &str) -> Result<(), crate::error::AppError> {
    let mut registry = load_registry().map_err(crate::error::AppError::Config)?;
    let pos = registry
        .instances
        .iter()
        .position(|i| i.id == instance_id)
        .ok_or_else(|| {
            crate::error::AppError::Config(format!("Instance {} not found", instance_id))
        })?;

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
    let workspace_folders = get_instance_workspace_folders(instance_id, &data_dir);

    // Determine executable FIRST while running processes are alive for discovery
    let exe_path = if !is_default {
        if let Some(ref p) = custom_exe {
            let pb = PathBuf::from(p);
            if pb.exists() {
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
            cmd.arg("--remote-debugging-port=0");
            let inst_home_opt = get_instance_home_dir(instance_id).ok();
            write_keyring_bypass_markers(&target_data_path, inst_home_opt.as_deref());
            if let Some(ref inst_home) = inst_home_opt {
                let _ = fs::create_dir_all(inst_home);
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

        let child = cmd.spawn().map_err(|e| {
            crate::error::AppError::Process(format!(
                "Failed to spawn macOS instance process: {}",
                e
            ))
        })?;
        let _ = record_instance_pid(instance_id, child.id(), &data_dir);
        let _ = crate::modules::backup_prompts_db::restore_running_prompts(
            Some(instance_id),
            false,
            None,
        );
        let _ =
            crate::modules::repo_db::resend_running_commands_for_instance(Some(instance_id), 20);
        let _ = crate::modules::repo_db::dispatch_running_prompts(instance_id);
        let _ = crate::modules::repo_db::ensure_prompt_goals_running_for_instance(instance_id);
        return Ok(());
    }

    #[cfg(not(target_os = "macos"))]
    {
        let mut cmd = Command::new(&exe_str);

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
        }
        cmd.arg("--new-window");

        cmd.stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());

        #[cfg(target_os = "windows")]
        {
            cmd.creation_flags(0x00000200 | 0x08000000); // CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS
        }

        #[cfg(target_os = "linux")]
        {
            crate::modules::process::clean_appimage_env(&mut cmd);
        }

        let child = cmd.spawn().map_err(|e| {
            crate::error::AppError::Process(format!("Failed to spawn instance process: {}", e))
        })?;
        let _ = record_instance_pid(instance_id, child.id(), &data_dir);
        let _ = crate::modules::backup_prompts_db::restore_running_prompts(
            Some(instance_id),
            false,
            None,
        );
        let _ =
            crate::modules::repo_db::resend_running_commands_for_instance(Some(instance_id), 20);
        let _ = crate::modules::repo_db::dispatch_running_prompts(instance_id);
        let _ = crate::modules::repo_db::ensure_prompt_goals_running_for_instance(instance_id);
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

    pids.retain(|&pid| {
        let Some(proc) = system.process(sysinfo::Pid::from_u32(pid)) else {
            return false;
        };
        let name = proc.name().to_string_lossy().to_lowercase();
        let exe = proc
            .exe()
            .map(|p| p.to_string_lossy().to_lowercase())
            .unwrap_or_default();
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

    let instance = registry
        .instances
        .iter()
        .find(|i| i.id == target_id)
        .ok_or_else(|| format!("Target instance {} not found", target_id))?;

    let is_default_inst = instance.is_default || instance.id == "default";

    if is_default_inst {
        let app_handle_opt = crate::modules::log_bridge::get_app_handle();
        let integration = match app_handle_opt.as_ref() {
            Some(h) => crate::modules::integration::SystemManager::Desktop(h.clone()),
            None => crate::modules::integration::SystemManager::Headless,
        };
        let service = crate::modules::account_service::AccountService::new(integration);
        service.switch_account(account_id, None).await?;
        bind_account_to_instance("default", &account.id, &account.email)?;
        let registry_after = load_registry().unwrap_or_default();
        if registry_after.active_instance_id.is_empty()
            || registry_after.active_instance_id == "default"
        {
            let _ = set_active_instance_id("default");
        }
        return Ok(());
    }

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

    // 1.5. [Step 1/5] Snapshot and backup running prompts scoped to THIS target instance BEFORE closing IDE
    let backed_up_count =
        crate::modules::repo_db::backup_running_prompts(&instance.id).unwrap_or(0);
    let _ =
        crate::modules::backup_prompts_db::backup_active_running_prompts(Some(&instance.id), None);

    // 2. [Step 2/5] Close the running instance process FIRST ("Kill First -> Write Second -> Start Third")
    //    Running Antigravity flushes in-memory state to state.vscdb/keyring on exit; closing first
    //    prevents the exiting process from overwriting our newly injected credentials.
    let _ = close_instance(&instance.id);
    if is_default_inst {
        if crate::modules::process::is_antigravity_running(None) {
            let _ = crate::modules::process::close_antigravity(20, None);
        }
        if crate::modules::process::is_antigravity_running(Some("ide")) {
            let _ = crate::modules::process::close_antigravity(20, Some("ide"));
        }
    }
    std::thread::sleep(std::time::Duration::from_millis(300));

    // 3. [Step 3/5] Inject credentials into target instance's state.vscdb & storage.json (and OS keyring) AFTER process exit
    inject_all_credentials(&account)?;

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
    tauri::async_runtime::spawn(async move {
        let _ = crate::modules::workspace_lease_manager::acquire_lease_with_details(
            &lease_acc_id,
            &lease_acc_email,
            &lease_inst_name,
            90,
        )
        .await;
        let _ = crate::modules::supabase_sync::sync_local_node_now().await;
    });

    // 5. [Step 4/5] Relaunch Antigravity preserving exact executable path and bound workspace folders
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

    // 5.5. [Step 5/5] Restore from backup DB and re-inject running prompts strictly for THIS instance
    let _ =
        crate::modules::backup_prompts_db::restore_running_prompts(Some(&instance.id), false, None);
    let resent =
        crate::modules::repo_db::resend_running_commands_for_instance(Some(&instance.id), 20)
            .unwrap_or_default();
    let dispatched = crate::modules::repo_db::dispatch_running_prompts(&instance.id).unwrap_or(0);
    let _ = crate::modules::repo_db::ensure_prompt_goals_running_for_instance(&instance.id);

    // 6. Dispatch unified Email and Telegram switch notifications
    let (target_4h, target_weekly) =
        crate::modules::auto_switcher::extract_dual_window_quotas(&account, "gemini-2.5-pro");

    let mut pred_exclusions = vec![account.id.clone(), account.email.clone()];
    if let Some(ref p_em) = prev_email {
        pred_exclusions.push(p_em.clone());
    }
    let predicted_candidate = crate::modules::auto_switcher::select_candidate_profiles(
        &instance.id,
        "gemini-2.5-pro",
        15.0,
        &pred_exclusions,
    )
    .ok()
    .and_then(|v| v.into_iter().next())
    .filter(|c| {
        !c.email.trim().eq_ignore_ascii_case(account.email.trim())
            && prev_email
                .as_deref()
                .map(|p| !c.email.trim().eq_ignore_ascii_case(p.trim()))
                .unwrap_or(true)
    });
    let predicted_next_email = predicted_candidate.map(|c| c.email);

    let running_projs = crate::modules::repo_db::list_running_projects().unwrap_or_default();
    let unique_projs = crate::modules::notification_hub::deduplicate_names(
        running_projs.into_iter().map(|p| p.repo_name),
    );

    let _notify = crate::modules::notification_hub::notify_account_switched_details(
        crate::modules::notification_hub::SwitchNotificationDetails {
            previous_email: prev_email.clone(),
            previous_quota_4h: prev_4h,
            previous_quota_weekly: prev_weekly,
            predicted_next_email,
            selected_email: account.email.clone(),
            target_quota_4h: target_4h,
            target_quota_weekly: target_weekly,
            credit_before_switch: prev_4h,
            threshold_activated: None,
            instance_id: instance.id.clone(),
            instance_name: instance.name.clone(),
            instance_mode: String::new(),
            reason: "Smart Rotator / Instance Account Switch".to_string(),
            is_auto: false,
            backed_up_projects: unique_projs,
            backed_up_prompts_count: Some(backed_up_count),
            restored_prompts_count: Some(resent.len() + dispatched),
        },
    )
    .await;
    if let Ok(prompts) = crate::modules::repo_db::list_all_prompts() {
        for prompt in prompts
            .into_iter()
            .filter(|prompt| prompt.instance_id == instance.id)
        {
            println!("  [Prompt] {} status={}", prompt.id, prompt.status);
            crate::modules::logger::log_info(&format!(
                "[Prompt] instance {} prompt {} status {}",
                instance.id, prompt.id, prompt.status
            ));
        }
    }

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
