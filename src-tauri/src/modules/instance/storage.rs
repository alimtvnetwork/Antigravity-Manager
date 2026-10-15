//! Per-instance app storage, session purge, and keyring bypass markers.
use super::*;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

fn app_storage_targets(data_dir: &Path) -> Vec<PathBuf> {
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

    targets
}

/// Clear cached post-onboarding terms acceptance (`lastLoginUsername` / `lastLoginIsGcpTos`)
/// across all `app_storage.json` locations for an instance so that when switching accounts,
/// Antigravity mounts the "Security Notice & Data Use" dialog and our CDP dismisser can
/// uncheck the Interactions data-sharing checkbox and click "Accept and continue".
pub fn clear_instance_onboarding_acceptance(data_dir: &Path) {
    for target in app_storage_targets(data_dir) {
        if let Some(p) = target.parent() {
            // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
            crate::error::record_ignored(fs::create_dir_all(p), "create directory");
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
        map.remove("jetski.onboarding.lastLoginUsername");
        map.remove("jetski.onboarding.lastLoginIsGcpTos");

        if let Ok(content) = serde_json::to_string_pretty(&map) {
            // Justification: best-effort file write; the target is regenerated or re-derived on the next relevant operation
            crate::error::record_ignored(fs::write(&target, content), "write file");
        }
    }
}

/// Synchronize `lastLoginUsername` and string `lastLoginIsGcpTos` across all `app_storage.json`
/// mirrors after CDP dismissal (or as a fallback if CDP is unavailable).
pub fn record_onboarding_acceptance_fallback(data_dir: &Path, bound_email: Option<&str>) {
    let Some(email) = bound_email.map(str::trim).filter(|e| !e.is_empty()) else {
        return;
    };

    let targets = app_storage_targets(data_dir);
    let mut detected_tos: Option<String> = None;
    for target in &targets {
        if let Ok(raw) = fs::read_to_string(target) {
            if let Ok(map) =
                serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&raw)
            {
                if let Some(serde_json::Value::String(s)) =
                    map.get("jetski.onboarding.lastLoginIsGcpTos")
                {
                    if s == "true" || s == "false" {
                        detected_tos = Some(s.clone());
                        break;
                    }
                }
            }
        }
    }
    let tos_str = detected_tos.unwrap_or_else(|| "false".to_string());

    for target in targets {
        if let Some(p) = target.parent() {
            // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
            crate::error::record_ignored(fs::create_dir_all(p), "create directory");
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
        map.insert(
            "jetski.onboarding.lastLoginUsername".to_string(),
            serde_json::Value::String(email.to_string()),
        );
        map.insert(
            "jetski.onboarding.lastLoginIsGcpTos".to_string(),
            serde_json::Value::String(tos_str.clone()),
        );

        if let Ok(content) = serde_json::to_string_pretty(&map) {
            // Justification: best-effort file write; the target is regenerated or re-derived on the next relevant operation
            crate::error::record_ignored(fs::write(&target, content), "write file");
        }
    }
}

/// Helper to synchronize app_storage.json across all instance storage locations
/// ensuring ide-install-wizard-shown, jetski.onboarding.lastLoginUsername, and
/// jetski.onboarding.lastLoginIsGcpTos reflect the bound account.
pub fn update_instance_app_storage(
    data_dir: &Path,
    bound_email: Option<&str>,
    _is_gcp_tos: bool,
) -> Result<(), String> {
    let targets = app_storage_targets(data_dir);

    for target in targets {
        if let Some(p) = target.parent() {
            // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
            crate::error::record_ignored(fs::create_dir_all(p), "create directory");
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
            let existing_email_matches = map
                .get("jetski.onboarding.lastLoginUsername")
                .and_then(|v| v.as_str())
                .map(|existing| existing.trim().eq_ignore_ascii_case(email))
                .unwrap_or(false);
            let existing_tos_is_valid_string = matches!(
                map.get("jetski.onboarding.lastLoginIsGcpTos"),
                Some(serde_json::Value::String(s)) if s == "true" || s == "false"
            );
            if !existing_email_matches || !existing_tos_is_valid_string {
                map.remove("jetski.onboarding.lastLoginUsername");
                map.remove("jetski.onboarding.lastLoginIsGcpTos");
            }
        } else {
            map.remove("jetski.onboarding.lastLoginUsername");
            map.remove("jetski.onboarding.lastLoginIsGcpTos");
        }

        if let Ok(content) = serde_json::to_string_pretty(&map) {
            // Justification: best-effort file write; the target is regenerated or re-derived on the next relevant operation
            crate::error::record_ignored(fs::write(&target, content), "write file");
        }
    }

    // Also update <data_dir>/User/settings.json for initial workbench preferences
    if bound_email
        .map(str::trim)
        .filter(|e| !e.is_empty())
        .is_some()
    {
        let user_dir = data_dir.join("User");
        // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
        crate::error::record_ignored(fs::create_dir_all(&user_dir), "create directory");
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
                // Justification: best-effort file write; the target is regenerated or re-derived on the next relevant operation
                crate::error::record_ignored(fs::write(&settings_path, pretty), "write file");
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
                // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
                crate::error::record_ignored(fs::create_dir_all(p), "create directory");
            }
            // Justification: best-effort db clone; the instance re-derives state if the clone is missing
            crate::error::record_ignored(
                safe_clone_sqlite_db(&default_db, &inst_db),
                "clone sqlite db",
            );
            // Justification: defensive session cleanup; leftover session data is tolerated
            crate::error::record_ignored(
                crate::modules::db::sanitize_session(&inst_db),
                "sanitize session db",
            );
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
                // Justification: cleanup of an optional directory tree; absence is the normal case
                crate::error::record_ignored(fs::remove_dir_all(&target), "remove directory tree");
            }
        }
        for lock in &["lockfile", "code.lock", "DevToolsActivePort"] {
            let p = root.join(lock);
            if p.exists() {
                // Justification: cleanup of an optional file; absence is the normal case
                crate::error::record_ignored(fs::remove_file(&p), "remove file");
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
        // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
        crate::error::record_ignored(fs::create_dir_all(&dir), "create directory");
        for name in &marker_names {
            let marker_path = dir.join(name);
            // Justification: best-effort file write; the target is regenerated or re-derived on the next relevant operation
            crate::error::record_ignored(fs::write(&marker_path, b"1\n"), "write file");
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
