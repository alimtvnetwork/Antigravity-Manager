use serde::{Deserialize, Serialize};

/// Instance configuration stored in instances.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceConfig {
    pub id: String,
    pub name: String,
    pub data_dir: String,
    pub extensions_dir: Option<String>,
    pub bound_account_id: Option<String>,
    pub bound_email: Option<String>,
    pub created_at: i64,
    pub last_used: i64,
    pub is_default: bool,
}

/// Runtime instance status exposed to frontend and CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceStatus {
    pub config: InstanceConfig,
    pub is_running: bool,
    pub pid: Option<u32>,
    pub memory_mb: Option<f64>,
}

/// Persistent registry of instances
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InstanceRegistry {
    pub active_instance_id: String,
    pub instances: Vec<InstanceConfig>,
}
