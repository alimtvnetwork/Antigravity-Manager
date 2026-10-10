use crate::modules::{account, config, instance, logger};
use std::fs;
use std::path::PathBuf;

use super::*;

/// Path to recovery snapshots folder
pub fn get_recovery_dir() -> Result<PathBuf, String> {
    let data_dir = account::get_data_dir()?;
    let recovery_dir = data_dir.join("task_recovery");
    if !recovery_dir.exists() {
        fs::create_dir_all(&recovery_dir)
            .map_err(|e| format!("Failed to create recovery dir: {}", e))?;
    }
    Ok(recovery_dir)
}

/// Snapshot current task and active instance before switching
pub fn snapshot_task_state(
    instance_id: &str,
    account_id: &str,
    reason: &str,
) -> Result<(), String> {
    let dir = get_recovery_dir()?;
    let now = chrono::Utc::now().timestamp();
    let snapshot = TaskRecoverySnapshot {
        instance_id: instance_id.to_string(),
        account_id: account_id.to_string(),
        timestamp: now,
        reason: reason.to_string(),
        is_recovered: false,
    };

    let path = dir.join(format!("snapshot_{}.json", instance_id));
    let content = serde_json::to_string_pretty(&snapshot)
        .map_err(|e| format!("Failed to serialize snapshot: {}", e))?;
    fs::write(path, content).map_err(|e| format!("Failed to write snapshot: {}", e))?;

    logger::log_info(&format!(
        "[AutoSwitcher] Saved task recovery snapshot for instance '{}'",
        instance_id
    ));
    Ok(())
}
