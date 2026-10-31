//! Cloned Profile MachineGuid Reseeder.
//! Generates and injects unique synthetic MachineGuids, telemetry device identifiers,
//! and sqmid tokens into cloned profile `storage.json` files to prevent profile collision.

use serde_json::Value;

/// Generates a deterministic-like or randomized UUID v4 format string.
pub fn generate_synthetic_uuid() -> String {
    format!(
        "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
        fastrand::u32(..),
        fastrand::u16(..),
        fastrand::u16(..) & 0x0FFF,
        (fastrand::u16(..) & 0x3FFF) | 0x8000,
        fastrand::u64(..) & 0xFFFFFFFFFFFF,
    )
}

/// Injects unique device identifiers into a parsed `storage.json` object.
pub fn reseed_storage_json(storage: &mut Value) -> usize {
    let mut modified_keys = 0;

    let target_keys = [
        "telemetry.machineId",
        "telemetry.devDeviceId",
        "telemetry.sqmId",
        "telemetry.macMachineId",
        "storage.machineGuid",
    ];

    if let Some(obj) = storage.as_object_mut() {
        for key in &target_keys {
            obj.insert(key.to_string(), Value::String(generate_synthetic_uuid()));
            modified_keys += 1;
        }
    }

    modified_keys
}
