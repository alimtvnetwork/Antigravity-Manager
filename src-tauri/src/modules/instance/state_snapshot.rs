//! Instance State Persistence Snapshotter.
//! Serializes and restores lightweight instance runtime state snapshots for crash recovery.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Lightweight runtime state snapshot of an active instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceStateSnapshot {
    pub instance_id: String,
    pub bound_account_id: Option<String>,
    pub pids: Vec<u32>,
    pub cdp_port: Option<u16>,
    pub timestamp_epoch: i64,
}

/// Atomically persists the instance runtime snapshot to disk.
pub fn write_instance_state_snapshot(
    snapshot: &InstanceStateSnapshot,
    target_file: &Path,
) -> Result<(), String> {
    if let Some(parent) = target_file.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let serialized = serde_json::to_string_pretty(snapshot)
        .map_err(|e| format!("Failed to serialize state snapshot: {}", e))?;

    std::fs::write(target_file, serialized)
        .map_err(|e| format!("Failed to write state snapshot: {}", e))?;

    Ok(())
}

/// Loads a previously persisted runtime snapshot from disk.
pub fn load_instance_state_snapshot(source_file: &Path) -> Result<InstanceStateSnapshot, String> {
    if !source_file.exists() {
        return Err("Snapshot file does not exist".to_string());
    }

    let content = std::fs::read_to_string(source_file)
        .map_err(|e| format!("Failed to read snapshot file: {}", e))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse state snapshot JSON: {}", e))
}
