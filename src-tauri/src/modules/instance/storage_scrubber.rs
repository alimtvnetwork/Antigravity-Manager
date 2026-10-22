//! Multi-Instance Storage JSON Scrubber.
//! Strips telemetry tracking IDs, machine GUIDs, and session tokens from cloned storage.json files.

use serde_json::{Map, Value};
use uuid::Uuid;

pub const SCRUBBED_TELEMETRY_KEYS: &[&str] = &[
    "telemetry.machineId",
    "telemetry.macMachineId",
    "telemetry.sqmId",
    "telemetry.devDeviceId",
    "storage.serviceMachineId",
    "crashReporter.id",
];

/// Deeply scrubs a raw storage.json string, regenerating unique synthetic telemetry identifiers.
pub fn scrub_storage_json(
    raw_json: &str,
    synthetic_machine_id: Option<&str>,
) -> Result<String, serde_json::Error> {
    let mut val: Value = serde_json::from_str(raw_json)?;

    if let Value::Object(ref mut map) = val {
        // 1. Remove blacklisted telemetry markers
        for key in SCRUBBED_TELEMETRY_KEYS {
            map.remove(*key);
        }

        // 2. Assign unique synthetic machine identifier
        let new_id = synthetic_machine_id
            .map(|s| s.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        map.insert(
            "telemetry.machineId".to_string(),
            Value::String(new_id.clone()),
        );
        map.insert(
            "telemetry.devDeviceId".to_string(),
            Value::String(Uuid::new_v4().to_string()),
        );

        // Strip auth tokens if present
        strip_auth_tokens(map);
    }

    serde_json::to_string_pretty(&val)
}

fn strip_auth_tokens(map: &mut Map<String, Value>) {
    let keys_to_remove: Vec<String> = map
        .keys()
        .filter(|k| {
            let lower = k.to_ascii_lowercase();
            lower.contains("token") || lower.contains("oauth") || lower.contains("secret")
        })
        .cloned()
        .collect();

    for k in keys_to_remove {
        map.remove(&k);
    }
}
