//! Instance registry load/save and envelope export.
use super::*;
use std::fs;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

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
        // Justification: settings injection is best-effort; the instance still launches with defaults
        crate::error::record_ignored(
            inject_instance_settings(&registry.instances[0]),
            "inject instance settings",
        );
        // Justification: idempotent ensure step; the next launch or lookup re-runs it if the default is missing
        crate::error::record_ignored(ensure_default_instance_exists(), "ensure default instance");
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
