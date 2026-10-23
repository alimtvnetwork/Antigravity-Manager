//! Settings Deep-Merge Synchronization Engine.
//!
//! Enforces deep-merge synchronization per AGENTS.md:
//! Allows deep-merge settings synchronization from both GUI and CLI across instances,
//! preserving target-specific identity tokens (port, id, name, profile directory).

use serde_json::{Map, Value};

pub const DEFAULT_PRESERVED_IDENTITY_KEYS: &[&str] = &[
    "id",
    "instance_id",
    "port",
    "remote_debugging_port",
    "name",
    "profile_dir",
    "created_at",
];

/// Recursively deep-merges `incoming` settings into `base` settings,
/// strictly shielding keys specified in `preserved_keys` from being overwritten.
pub fn deep_merge_settings(
    base: &Value,
    incoming: &Value,
    preserved_keys: &[&str],
) -> Value {
    match (base, incoming) {
        (Value::Object(base_map), Value::Object(incoming_map)) => {
            let mut merged: Map<String, Value> = base_map.clone();

            for (key, inc_val) in incoming_map {
                let is_preserved = preserved_keys.iter().any(|&pk| pk == key);

                if is_preserved && base_map.contains_key(key) {
                    // Identity token collision: retain base instance value
                    continue;
                }

                if let Some(base_val) = base_map.get(key) {
                    if base_val.is_object() && inc_val.is_object() {
                        merged.insert(
                            key.clone(),
                            deep_merge_settings(base_val, inc_val, preserved_keys),
                        );
                    } else {
                        merged.insert(key.clone(), inc_val.clone());
                    }
                } else {
                    merged.insert(key.clone(), inc_val.clone());
                }
            }

            Value::Object(merged)
        }
        // If either is not an object, incoming overwrites base unless preserved
        (_, incoming_val) => incoming_val.clone(),
    }
}
