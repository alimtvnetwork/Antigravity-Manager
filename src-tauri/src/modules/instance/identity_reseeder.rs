//! Cloned Profile Machine Identity Reseeder.
//! Re-seeds `storage.json` telemetry identities with unique machine and device UUIDs following cloning.

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::Path;
use uuid::Uuid;

/// Computes SHA-256 hex string for a seed.
fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Re-seeds `storage.json` telemetry identity fields with unique synthetic values.
pub fn reseed_profile_telemetry_identity(storage_json_path: &Path, instance_id: &str) -> Result<(), String> {
    if !storage_json_path.exists() {
        return Ok(());
    }

    let raw = std::fs::read_to_string(storage_json_path)
        .map_err(|e| format!("Failed to read storage.json: {}", e))?;

    let mut json_val: Value = serde_json::from_str(&raw)
        .map_err(|e| format!("Invalid JSON in storage.json: {}", e))?;

    let machine_uuid = Uuid::new_v4().to_string();
    let mac_uuid = Uuid::new_v4().to_string();
    let dev_device_id = Uuid::new_v4().to_string();
    let sqm_id = format!("{{{}}}", Uuid::new_v4().to_string().to_uppercase());

    let machine_hash = sha256_hex(&format!("{}-{}", instance_id, machine_uuid));
    let mac_hash = sha256_hex(&format!("{}-{}", instance_id, mac_uuid));

    if let Some(obj) = json_val.as_object_mut() {
        obj.insert("telemetry.machineId".to_string(), Value::String(machine_hash));
        obj.insert("telemetry.macMachineId".to_string(), Value::String(mac_hash));
        obj.insert("telemetry.devDeviceId".to_string(), Value::String(dev_device_id.clone()));
        obj.insert("telemetry.sqmId".to_string(), Value::String(sqm_id));
        obj.insert("storage.serviceMachineId".to_string(), Value::String(dev_device_id));
    }

    let serialized = serde_json::to_string_pretty(&json_val)
        .map_err(|e| format!("Failed to serialize storage.json: {}", e))?;

    std::fs::write(storage_json_path, serialized)
        .map_err(|e| format!("Failed to write storage.json: {}", e))?;

    Ok(())
}
