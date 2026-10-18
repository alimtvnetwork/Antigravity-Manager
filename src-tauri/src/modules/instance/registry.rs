//! Instance registry load/save and envelope export.
use super::*;
use std::fs;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Helper to construct and persist a pristine default registry
fn create_default_registry() -> Result<InstanceRegistry, String> {
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
    // Justification: settings injection is best-effort; the instance still launches with defaults
    crate::error::record_ignored(
        inject_instance_settings(&registry.instances[0]),
        "inject instance settings",
    );
    // Justification: idempotent ensure step; the next launch or lookup re-runs it if the default is missing
    crate::error::record_ignored(ensure_default_instance_exists(), "ensure default instance");
    Ok(registry)
}

/// Helper to parse registry payload supporting both json envelope and plain JSON
pub(crate) fn parse_registry_payload(content: &str) -> Result<InstanceRegistry, String> {
    crate::modules::json_envelope::extract_payload::<InstanceRegistry>(content)
        .map(|(data, _)| data)
        .or_else(|_| serde_json::from_str(content))
        .map_err(|e| format!("Failed to parse instances registry: {}", e))
}

/// Load instance registry or initialize with default instance
pub fn load_registry() -> Result<InstanceRegistry, String> {
    let registry_path = get_registry_path()?;
    if !registry_path.exists() {
        return create_default_registry();
    }

    let backup_path = registry_path.with_extension("json.bak");

    let raw_content = match fs::read_to_string(&registry_path) {
        Ok(c) => c,
        Err(read_err) => {
            if backup_path.exists() {
                fs::read_to_string(&backup_path)
                    .map_err(|_| format!("Failed to read instances registry: {}", read_err))?
            } else {
                return Err(format!("Failed to read instances registry: {}", read_err));
            }
        }
    };

    let trimmed = raw_content.trim();
    let mut registry: InstanceRegistry = if trimmed.is_empty() {
        tracing::warn!("instances.json was empty; attempting backup recovery or creating default");
        if backup_path.exists() {
            if let Ok(bak_content) = fs::read_to_string(&backup_path) {
                if !bak_content.trim().is_empty() {
                    if let Ok(reg) = parse_registry_payload(&bak_content) {
                        let _ = save_registry(&reg);
                        reg
                    } else {
                        create_default_registry()?
                    }
                } else {
                    create_default_registry()?
                }
            } else {
                create_default_registry()?
            }
        } else {
            create_default_registry()?
        }
    } else {
        match parse_registry_payload(trimmed) {
            Ok(reg) => reg,
            Err(parse_err) => {
                tracing::warn!(
                    "Failed to parse instances.json: {}; trying backup",
                    parse_err
                );
                if backup_path.exists() {
                    if let Ok(bak_content) = fs::read_to_string(&backup_path) {
                        if let Ok(reg) = parse_registry_payload(bak_content.trim()) {
                            let _ = save_registry(&reg);
                            reg
                        } else {
                            tracing::error!(
                                "Backup registry corrupted too; re-creating default: {}",
                                parse_err
                            );
                            create_default_registry()?
                        }
                    } else {
                        create_default_registry()?
                    }
                } else {
                    tracing::error!(
                        "No backup registry found; re-creating default: {}",
                        parse_err
                    );
                    create_default_registry()?
                }
            }
        }
    };

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
        // Justification: registry persistence is a write-through cache of already-updated in-memory state; retried on the next registry touch
        crate::error::record_ignored(save_registry(&registry), "persist instance registry");
        // Justification: idempotent ensure step; the next launch or lookup re-runs it if the default is missing
        crate::error::record_ignored(ensure_default_instance_exists(), "ensure default instance");
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
        // Justification: registry persistence is a write-through cache of already-updated in-memory state; retried on the next registry touch
        crate::error::record_ignored(save_registry(&registry), "persist instance registry");
    }

    static SYNCED_TITLES_ONCE: std::sync::atomic::AtomicBool =
        std::sync::atomic::AtomicBool::new(false);
    if !SYNCED_TITLES_ONCE.swap(true, std::sync::atomic::Ordering::Relaxed) {
        for inst in &registry.instances {
            // Justification: settings injection is best-effort; the instance still launches with defaults
            crate::error::record_ignored(
                inject_instance_settings(inst),
                "inject instance settings",
            );
        }
    }

    Ok(registry)
}

/// Save instance registry atomically with backup retention
pub fn save_registry(registry: &InstanceRegistry) -> Result<(), String> {
    let registry_path = get_registry_path()?;
    let json = serde_json::to_string_pretty(registry)
        .map_err(|e| format!("Failed to serialize instances registry: {}", e))?;

    // Create backup of current registry before overwriting if non-empty
    if registry_path.exists() {
        if let Ok(current_meta) = fs::metadata(&registry_path) {
            if current_meta.len() > 0 {
                let backup_path = registry_path.with_extension("json.bak");
                // Justification: best-effort backup; failure to backup must not block saving valid configuration
                crate::error::record_ignored(
                    fs::copy(&registry_path, &backup_path),
                    "backup instances registry before atomic write",
                );
            }
        }
    }

    crate::utils::fs::write_atomic(&registry_path, json.as_bytes())
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
