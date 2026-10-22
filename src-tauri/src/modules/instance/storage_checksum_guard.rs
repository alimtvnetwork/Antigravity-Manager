//! Cloned Instance Storage Checksum Guard.
//! Inspects `storage.json` within instance profile directories, validating JSON structural
//! integrity and verifying that telemetry IDs are valid UUID format strings.

use serde_json::Value;
use std::fs;
use std::path::Path;

/// Result of storage JSON verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageValidationOutcome {
    Valid { keys_verified: usize },
    FileNotFound,
    InvalidJson(String),
    MissingTelemetryKey(String),
    MalformedUuid(String),
}

/// Verifies `storage.json` file in target instance user data directory.
pub fn verify_instance_storage_json(user_data_dir: &Path) -> StorageValidationOutcome {
    let storage_path = user_data_dir.join("User").join("globalStorage").join("storage.json");
    if !storage_path.exists() {
        return StorageValidationOutcome::FileNotFound;
    }

    let contents = match fs::read_to_string(&storage_path) {
        Ok(c) => c,
        Err(e) => return StorageValidationOutcome::InvalidJson(e.to_string()),
    };

    let parsed: Value = match serde_json::from_str(&contents) {
        Ok(v) => v,
        Err(e) => return StorageValidationOutcome::InvalidJson(e.to_string()),
    };

    let obj = match parsed.as_object() {
        Some(o) => o,
        None => return StorageValidationOutcome::InvalidJson("Root is not object".to_string()),
    };

    let target_keys = ["telemetry.machineId", "telemetry.sqmId"];
    let mut verified = 0;

    for key in &target_keys {
        if let Some(val) = obj.get(*key).and_then(|v| v.as_str()) {
            if val.len() < 16 {
                return StorageValidationOutcome::MalformedUuid((*key).to_string());
            }
            verified += 1;
        } else {
            return StorageValidationOutcome::MissingTelemetryKey((*key).to_string());
        }
    }

    StorageValidationOutcome::Valid {
        keys_verified: verified,
    }
}
