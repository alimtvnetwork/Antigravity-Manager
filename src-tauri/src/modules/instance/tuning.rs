//! Per-instance turbo/plan-review/theme tuning.
use super::*;
use std::fs;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

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
                // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
                crate::error::record_ignored(fs::create_dir_all(parent), "create directory");
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
                    // Justification: best-effort file write; the target is regenerated or re-derived on the next relevant operation
                    crate::error::record_ignored(fs::write(&path, pretty), "write file");
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
                // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
                crate::error::record_ignored(fs::create_dir_all(parent), "create directory");
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
                    // Justification: best-effort file write; the target is regenerated or re-derived on the next relevant operation
                    crate::error::record_ignored(fs::write(&path, pretty), "write file");
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
                // Justification: idempotent directory setup; a real failure surfaces at the next file op needing the dir
                crate::error::record_ignored(fs::create_dir_all(parent), "create directory");
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
                    // Justification: best-effort file write; the target is regenerated or re-derived on the next relevant operation
                    crate::error::record_ignored(fs::write(&path, pretty), "write file");
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

            deep_merge_json(&mut current, &settings_to_apply);

            if let Ok(pretty) = serde_json::to_string_pretty(&current) {
                // Justification: best-effort file write; the target is regenerated or re-derived on the next relevant operation
                crate::error::record_ignored(fs::write(&path, pretty), "write file");
            }
        }
        updated_count += 1;
    }

    Ok(updated_count)
}
