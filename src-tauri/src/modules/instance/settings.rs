//! Instance settings discovery and injection.
use super::*;
use std::fs;
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

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
            // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
            crate::error::record_ignored(fs::create_dir_all(parent), "create directory");
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
                // Justification: best-effort file write; the target is regenerated or re-derived on the next relevant operation
                crate::error::record_ignored(fs::write(&target, pretty), "write file");
            }
        }
    }

    Ok(())
}
