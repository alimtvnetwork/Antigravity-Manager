//! Cloned Profile Telemetry Storage Cleaner
//! Sanitizes telemetry and hardware fingerprint UUIDs from instance storage.json
//! to guarantee clean machine identity and prevent cross-instance telemetry collisions.

use serde_json::Value;
use std::path::Path;
use uuid::Uuid;

pub struct TelemetryStorageCleaner;

impl TelemetryStorageCleaner {
    /// Sanitizes an instance's storage.json file in place, replacing machine IDs with fresh UUIDs.
    pub fn sanitize_storage_file(storage_json_path: &Path) -> std::io::Result<bool> {
        if !storage_json_path.exists() {
            return Ok(false);
        }

        let content = std::fs::read_to_string(storage_json_path)?;
        let mut json: Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => return Ok(false),
        };

        let mut modified = false;

        if let Some(obj) = json.as_object_mut() {
            // Replace machine IDs with fresh randomized UUIDs
            let telemetry_keys = [
                "telemetry.machineId",
                "telemetry.macMachineId",
                "telemetry.devDeviceId",
                "telemetry.sqmId",
                "storage.serviceMachineId",
            ];

            for key in telemetry_keys {
                if obj.contains_key(key) {
                    obj.insert(key.to_string(), Value::String(Uuid::new_v4().to_string()));
                    modified = true;
                }
            }
        }

        if modified {
            let serialized = serde_json::to_string_pretty(&json)?;
            std::fs::write(storage_json_path, serialized)?;
        }

        Ok(modified)
    }
}
