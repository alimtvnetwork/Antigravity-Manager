//! Filesystem sync: recursive copy, IDE parity, default instance.
use super::*;
use crate::error::AppError;
use crate::error::AppResult;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

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
                    // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
                    crate::error::record_ignored(fs::create_dir_all(parent), "create directory");
                }
                // Justification: best-effort file copy; downstream code regenerates or tolerates the missing copy
                crate::error::record_ignored(fs::copy(entry.path(), &child_dst), "copy file");
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
                    // Justification: best-effort file write; the target is regenerated or re-derived on the next relevant operation
                    crate::error::record_ignored(
                        fs::write(&target_config_json, formatted),
                        "write file",
                    );
                }
            } else {
                // Justification: best-effort file copy; downstream code regenerates or tolerates the missing copy
                crate::error::record_ignored(
                    fs::copy(&src_cfg_path, &target_config_json),
                    "copy file",
                );
            }
        } else if !target_config_json.exists() {
            // Justification: best-effort file copy; downstream code regenerates or tolerates the missing copy
            crate::error::record_ignored(fs::copy(&src_cfg_path, &target_config_json), "copy file");
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
            // Justification: best-effort file write; the target is regenerated or re-derived on the next relevant operation
            crate::error::record_ignored(fs::write(&target_config_json, formatted), "write file");
        }
    }

    // 2. Synchronize .gemini/config/plugins/
    let target_plugins_dir = target_config_dir.join("plugins");
    let source_plugins_dir = candidate_sources
        .iter()
        .map(|p| p.join("config").join("plugins"))
        .find(|p| p.is_dir());

    if let Some(src_plugins) = source_plugins_dir {
        // Justification: best-effort tree sync; re-synced on the next parity pass
        crate::error::record_ignored(
            sync_directory_recursive(&src_plugins, &target_plugins_dir),
            "sync directory tree",
        );
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
        // Justification: best-effort tree sync; re-synced on the next parity pass
        crate::error::record_ignored(
            sync_directory_recursive(&src_skills, &target_skills_dir),
            "sync directory tree",
        );
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
            // Justification: best-effort file copy; downstream code regenerates or tolerates the missing copy
            crate::error::record_ignored(fs::copy(&src_marker, &target_marker), "copy file");
        } else if !target_marker.exists() {
            // Justification: best-effort file write; the target is regenerated or re-derived on the next relevant operation
            crate::error::record_ignored(fs::write(&target_marker, b"1\n"), "write file");
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
        // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
        crate::error::record_ignored(fs::create_dir_all(parent), "create directory");
    }
    if let Ok(formatted) = serde_json::to_string_pretty(&target_settings_val) {
        // Justification: best-effort file write; the target is regenerated or re-derived on the next relevant operation
        crate::error::record_ignored(fs::write(&target_settings_path, &formatted), "write file");

        // Also write to data/User/settings.json if data dir exists
        let target_data_user = target_data.join("User");
        if target_data_user.exists() {
            // Justification: best-effort file write; the target is regenerated or re-derived on the next relevant operation
            crate::error::record_ignored(
                fs::write(target_data_user.join("settings.json"), &formatted),
                "write file",
            );
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
        // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
        crate::error::record_ignored(fs::create_dir_all(&default_home), "create directory");
    }
    if !default_data.exists() {
        // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
        crate::error::record_ignored(fs::create_dir_all(&default_data), "create directory");
    }

    sync_instance_ide_parity("default")?;
    Ok(())
}
