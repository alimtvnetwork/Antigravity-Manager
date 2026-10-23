//! Instance storage JSON migration guard.
//! Audits and repairs `storage.json` across IDE upgrades, backfilling required
//! telemetry and hardware isolation keys while stripping corrupted state blocks.

use crate::error::AppError;
use serde_json::{json, Value};
use std::path::Path;

pub struct StorageMigrationGuard;

impl StorageMigrationGuard {
    /// Audits the given `storage.json` file, backfilling missing telemetry GUIDs if necessary.
    pub fn ensure_valid_storage_json(storage_path: &Path) -> Result<bool, AppError> {
        if !storage_path.is_file() {
            // Nothing to migrate if file doesn't exist yet
            return Ok(false);
        }

        let content = std::fs::read_to_string(storage_path)
            .map_err(AppError::Io)?;

        let mut root: Value = serde_json::from_str(&content)
            .unwrap_or_else(|_| json!({}));

        let mut modified = false;

        // Verify and backfill critical telemetry keys
        let required_keys = [
            "telemetry.machineId",
            "telemetry.macMachineId",
            "telemetry.devDeviceId",
            "telemetry.sqmId",
        ];

        for key in required_keys {
            if root.get(key).and_then(|v| v.as_str()).map(|s| s.trim().is_empty()).unwrap_or(true) {
                let synthetic_guid = generate_synthetic_guid();
                root[key] = json!(synthetic_guid);
                modified = true;
            }
        }

        if modified {
            let serialized = serde_json::to_string_pretty(&root)
                .map_err(|e| AppError::Config(format!("Failed to serialize migrated storage.json: {}", e)))?;

            std::fs::write(storage_path, serialized)
                .map_err(AppError::Io)?;
        }

        Ok(modified)
    }
}

fn generate_synthetic_guid() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(123456789);

    let h1 = (now & 0xFFFFFFFF) as u32;
    let h2 = ((now >> 32) & 0xFFFF) as u16;
    let h3 = 0x4000 | (((now >> 48) & 0x0FFF) as u16);
    let h4 = 0x8000 | (((now >> 64) & 0x3FFF) as u16);
    let h5 = ((now >> 80) & 0xFFFFFFFFFFFF) as u64;

    format!("{:08x}-{:04x}-{:04x}-{:04x}-{:012x}", h1, h2, h3, h4, h5)
}
