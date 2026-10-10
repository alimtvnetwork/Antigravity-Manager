//! Persisted PID bookkeeping for instances.
use super::*;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Record an instance PID launch in the SQLite database and in registry
pub fn record_instance_pid(instance_id: &str, pid: u32, data_dir: &str) -> Result<(), String> {
    let canonical_id = resolve_instance_id(instance_id).unwrap_or_else(|_| instance_id.to_string());
    let now = chrono::Utc::now().timestamp();
    if let Ok(conn) = open_instance_db() {
        // Justification: non-critical sqlite bookkeeping write; read paths tolerate stale data
        crate::error::record_ignored(
            conn.execute(
                "INSERT INTO instance_processes (instance_id, pid, data_dir, launched_at, is_active)
                 VALUES (?1, ?2, ?3, ?4, 1)
                 ON CONFLICT(instance_id) DO UPDATE SET
                    pid = excluded.pid,
                    data_dir = excluded.data_dir,
                    launched_at = excluded.launched_at,
                    is_active = 1",
                rusqlite::params![canonical_id, pid as i64, data_dir, now],
            ),
            "run sqlite statement",
        );
    }

    if let Ok(mut registry) = load_registry() {
        if let Some(inst) = registry
            .instances
            .iter_mut()
            .find(|i| i.id == canonical_id || i.id == instance_id)
        {
            inst.pid = Some(pid);
            inst.last_used = now;
            // Justification: registry persistence is a write-through cache of already-updated in-memory state; retried on the next registry touch
            crate::error::record_ignored(save_registry(&registry), "persist instance registry");
        }
    }

    if pid > 0 {
        if let Ok(mut cache) = INSTANCE_PROCESS_CACHE.write() {
            let record = InstanceProcessRecord {
                instance_id: canonical_id.clone(),
                pid,
                pids: vec![pid],
                primary_pid: Some(pid),
                data_dir: data_dir.to_string(),
                launched_at: now,
                last_verified_at: now,
                is_alive: true,
                command_line: None,
            };
            cache.insert(canonical_id.clone(), record.clone());
            if instance_id != canonical_id {
                cache.insert(instance_id.to_string(), record);
            }
        }
    }

    crate::modules::logger::log_info(&format!(
        "[Instance] Saved instance '{}' launch PID {} to SQLite DB and registry",
        canonical_id, pid
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
    let canonical_id = resolve_instance_id(instance_id).unwrap_or_else(|_| instance_id.to_string());
    if let Ok(conn) = open_instance_db() {
        // Justification: non-critical sqlite bookkeeping write; read paths tolerate stale data
        crate::error::record_ignored(
            conn.execute(
                "UPDATE instance_processes SET is_active = 0 WHERE instance_id = ?1 OR instance_id = ?2",
                rusqlite::params![canonical_id, instance_id],
            ),
            "run sqlite statement",
        );
    }

    if let Ok(mut registry) = load_registry() {
        if let Some(inst) = registry
            .instances
            .iter_mut()
            .find(|i| i.id == canonical_id || i.id == instance_id)
        {
            inst.pid = None;
            // Justification: registry persistence is a write-through cache of already-updated in-memory state; retried on the next registry touch
            crate::error::record_ignored(save_registry(&registry), "persist instance registry");
        }
    }

    invalidate_instance_process_cache(instance_id);
    invalidate_instance_process_cache(&canonical_id);

    Ok(())
}
