use crate::models::DeviceProfile;
use crate::modules::{account, logger, process};
use chrono::Local;
use rusqlite::Connection;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

use super::*;

/// Backup storage.json, returns backup file path
#[allow(dead_code)]
pub fn backup_storage(storage_path: &Path) -> Result<PathBuf, String> {
    if !storage_path.exists() {
        return Err(format!("storage_json_missing: {:?}", storage_path));
    }
    let dir = storage_path
        .parent()
        .ok_or_else(|| "failed_to_get_storage_parent_dir".to_string())?;
    let backup_path = dir.join(format!(
        "storage.json.backup_{}",
        Local::now().format("%Y%m%d_%H%M%S")
    ));
    fs::copy(storage_path, &backup_path).map_err(|e| format!("backup_failed: {}", e))?;
    Ok(backup_path)
}

/// Read current device profile from storage.json
#[allow(dead_code)]
pub fn read_profile(storage_path: &Path) -> Result<DeviceProfile, String> {
    let content = fs::read_to_string(storage_path)
        .map_err(|e| format!("read_failed ({:?}): {}", storage_path, e))?;
    let json: Value = serde_json::from_str(&content)
        .map_err(|e| format!("parse_failed ({:?}): {}", storage_path, e))?;

    // Supports nested telemetry or flat telemetry.xxx
    let get_field = |key: &str| -> Option<String> {
        if let Some(obj) = json.get("telemetry").and_then(|v| v.as_object()) {
            if let Some(v) = obj.get(key).and_then(|v| v.as_str()) {
                return Some(v.to_string());
            }
        }
        if let Some(v) = json
            .get(format!("telemetry.{key}"))
            .and_then(|v| v.as_str())
        {
            return Some(v.to_string());
        }
        None
    };

    Ok(DeviceProfile {
        machine_id: get_field("machineId").ok_or("missing_machine_id")?,
        mac_machine_id: get_field("macMachineId").ok_or("missing_mac_machine_id")?,
        dev_device_id: get_field("devDeviceId").ok_or("missing_dev_device_id")?,
        sqm_id: get_field("sqmId").ok_or("missing_sqm_id")?,
    })
}

/// Write device profile to storage.json
pub fn write_profile(storage_path: &Path, profile: &DeviceProfile) -> Result<(), String> {
    if !storage_path.exists() {
        return Err(format!("storage_json_missing: {:?}", storage_path));
    }

    let content = fs::read_to_string(storage_path).map_err(|e| format!("read_failed: {}", e))?;
    let mut json: Value =
        serde_json::from_str(&content).map_err(|e| format!("parse_failed: {}", e))?;

    // Ensure telemetry is an object
    if !json.get("telemetry").map_or(false, |v| v.is_object()) {
        if json.as_object_mut().is_some() {
            json["telemetry"] = serde_json::json!({});
        } else {
            return Err("json_top_level_not_object".to_string());
        }
    }

    if let Some(telemetry) = json.get_mut("telemetry").and_then(|v| v.as_object_mut()) {
        telemetry.insert(
            "machineId".to_string(),
            Value::String(profile.machine_id.clone()),
        );
        telemetry.insert(
            "macMachineId".to_string(),
            Value::String(profile.mac_machine_id.clone()),
        );
        telemetry.insert(
            "devDeviceId".to_string(),
            Value::String(profile.dev_device_id.clone()),
        );
        telemetry.insert("sqmId".to_string(), Value::String(profile.sqm_id.clone()));
    } else {
        return Err("telemetry_not_object".to_string());
    }

    // Write flat keys as well, compatible with old formats
    if let Some(map) = json.as_object_mut() {
        map.insert(
            "telemetry.machineId".to_string(),
            Value::String(profile.machine_id.clone()),
        );
        map.insert(
            "telemetry.macMachineId".to_string(),
            Value::String(profile.mac_machine_id.clone()),
        );
        map.insert(
            "telemetry.devDeviceId".to_string(),
            Value::String(profile.dev_device_id.clone()),
        );
        map.insert(
            "telemetry.sqmId".to_string(),
            Value::String(profile.sqm_id.clone()),
        );
    }

    // Sync storage.serviceMachineId (match with devDeviceId), place at root level
    if let Some(map) = json.as_object_mut() {
        map.insert(
            "storage.serviceMachineId".to_string(),
            Value::String(profile.dev_device_id.clone()),
        );
    }

    let updated =
        serde_json::to_string_pretty(&json).map_err(|e| format!("serialize_failed: {}", e))?;
    fs::write(storage_path, updated)
        .map_err(|e| format!("write_failed ({:?}): {}", storage_path, e))?;
    logger::log_info(&format!("device_profile_written to {:?}", storage_path));

    // Sync ItemTable.storage.serviceMachineId in state.vscdb
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(
        sync_state_service_machine_id_value(&profile.dev_device_id),
        "sync_state_service_machine_id_value",
    );
    Ok(())
}

/// Only sync serviceMachineId, don't change other fields
#[allow(dead_code)]
pub fn sync_service_machine_id(storage_path: &Path, service_id: &str) -> Result<(), String> {
    let content = fs::read_to_string(storage_path).map_err(|e| format!("read_failed: {}", e))?;
    let mut json: Value =
        serde_json::from_str(&content).map_err(|e| format!("parse_failed: {}", e))?;

    if let Some(map) = json.as_object_mut() {
        map.insert(
            "storage.serviceMachineId".to_string(),
            Value::String(service_id.to_string()),
        );
    }

    let updated =
        serde_json::to_string_pretty(&json).map_err(|e| format!("serialize_failed: {}", e))?;
    fs::write(storage_path, updated).map_err(|e| format!("write_failed: {}", e))?;
    logger::log_info("service_machine_id_synced");

    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(
        sync_state_service_machine_id_value(service_id),
        "sync_state_service_machine_id_value",
    );
    Ok(())
}

/// Read serviceMachineId from storage.json (fallback to devDeviceId), sync back if missing and sync state.vscdb
#[allow(dead_code)]
pub fn sync_service_machine_id_from_storage(storage_path: &Path) -> Result<(), String> {
    if !storage_path.exists() {
        return Err("storage_json_missing".to_string());
    }
    let content = fs::read_to_string(storage_path).map_err(|e| format!("read_failed: {}", e))?;
    let mut json: Value =
        serde_json::from_str(&content).map_err(|e| format!("parse_failed: {}", e))?;

    let service_id = json
        .get("storage.serviceMachineId")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            json.get("telemetry")
                .and_then(|t| t.get("devDeviceId"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .or_else(|| {
            json.get("telemetry.devDeviceId")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .ok_or("missing_ids_in_storage")?;

    let mut dirty = false;
    if json
        .get("storage.serviceMachineId")
        .and_then(|v| v.as_str())
        .is_none()
    {
        if let Some(map) = json.as_object_mut() {
            map.insert(
                "storage.serviceMachineId".to_string(),
                Value::String(service_id.clone()),
            );
            dirty = true;
        }
    }

    if dirty {
        let updated =
            serde_json::to_string_pretty(&json).map_err(|e| format!("serialize_failed: {}", e))?;
        fs::write(storage_path, updated).map_err(|e| format!("write_failed: {}", e))?;
        logger::log_info("service_machine_id_added");
    }

    sync_state_service_machine_id_value(&service_id)
}

pub(crate) fn sync_state_service_machine_id_value(service_id: &str) -> Result<(), String> {
    let db_path = get_state_db_path()?;
    if !db_path.exists() {
        logger::log_warn(&format!("state_db_missing: {:?}", db_path));
        return Ok(());
    }

    let conn = Connection::open(&db_path).map_err(|e| format!("db_open_failed: {}", e))?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS ItemTable (key TEXT PRIMARY KEY, value TEXT);",
        [],
    )
    .map_err(|e| format!("failed_to_create_item_table: {}", e))?;
    conn.execute(
        "INSERT OR REPLACE INTO ItemTable (key, value) VALUES ('storage.serviceMachineId', ?1);",
        [service_id],
    )
    .map_err(|e| format!("failed_to_write_to_db: {}", e))?;
    logger::log_info("service_machine_id_synced_to_db");
    Ok(())
}
