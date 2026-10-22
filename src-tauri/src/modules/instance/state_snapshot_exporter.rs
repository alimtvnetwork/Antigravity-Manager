//! Instance State Health Snapshot Exporter.
//! Exports structured JSON snapshots of instance health, allocated CDP ports,
//! process lifecycles, and bound accounts for system diagnostics and CLI status.

use serde::{Deserialize, Serialize};

/// High-level health snapshot of a single running or registered instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceHealthSnapshot {
    pub instance_id: String,
    pub name: String,
    pub is_running: bool,
    pub pid: Option<u32>,
    pub cdp_port: Option<u16>,
    pub bound_account_email: Option<String>,
    pub uptime_seconds: Option<u64>,
}

/// Global system-level instance snapshot bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalInstanceSnapshot {
    pub exported_at_timestamp: u64,
    pub total_instances: usize,
    pub running_instances: usize,
    pub instances: Vec<InstanceHealthSnapshot>,
}

impl GlobalInstanceSnapshot {
    /// Creates a new global snapshot.
    pub fn new(instances: Vec<InstanceHealthSnapshot>) -> Self {
        let running_instances = instances.iter().filter(|i| i.is_running).count();
        let total_instances = instances.len();
        let exported_at_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            exported_at_timestamp,
            total_instances,
            running_instances,
            instances,
        }
    }

    /// Serializes snapshot into pretty-printed JSON.
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}
