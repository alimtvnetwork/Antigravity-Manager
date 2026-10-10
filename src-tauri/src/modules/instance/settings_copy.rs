//! Settings copy and default enforcement.
use super::*;
use std::fs;
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

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
                        // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
                        crate::error::record_ignored(
                            fs::create_dir_all(parent),
                            "create directory",
                        );
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
                        // Justification: best-effort file write; the target is regenerated or re-derived on the next relevant operation
                        crate::error::record_ignored(fs::write(&dst_path, pretty), "write file");
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
                // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
                crate::error::record_ignored(fs::create_dir_all(dst_dir), "create directory");
                // Justification: best-effort file copy; downstream code regenerates or tolerates the missing copy
                crate::error::record_ignored(
                    fs::copy(&found_src, dst_dir.join(file_name)),
                    "copy file",
                );
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
                // Justification: best-effort tree copy; parity sync completes it on launch
                crate::error::record_ignored(
                    copy_dir_recursive(&src_snippets, &dst_snippets),
                    "copy directory tree",
                );
            }
            break;
        }
    }

    // 6. Ensure target window title is freshly injected
    // Justification: settings injection is best-effort; the instance still launches with defaults
    crate::error::record_ignored(
        inject_instance_settings(&to_inst),
        "inject instance settings",
    );

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
                // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
                crate::error::record_ignored(fs::create_dir_all(parent), "create directory");
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
                // Justification: best-effort file write; the target is regenerated or re-derived on the next relevant operation
                crate::error::record_ignored(fs::write(&path, pretty), "write file");
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
