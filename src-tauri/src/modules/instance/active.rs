//! Active/default instance selection and account binding.
use super::*;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

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
        // Justification: non-critical sqlite bookkeeping write; read paths tolerate stale data
        crate::error::record_ignored(
            conn.execute(
                "INSERT INTO active_instance_selection (id, instance_id, updated_at)
                 VALUES (1, ?1, ?2)
                 ON CONFLICT(id) DO UPDATE SET instance_id = excluded.instance_id, updated_at = excluded.updated_at",
                rusqlite::params![instance_id, now],
            ),
            "run sqlite statement",
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
