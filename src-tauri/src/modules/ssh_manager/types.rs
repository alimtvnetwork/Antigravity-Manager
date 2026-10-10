use serde::{Deserialize, Serialize};

use super::*;

pub(crate) const SSH_CONFIG_MARKER_START: &str = "# --- agm ssh managed (do not edit) ---";

pub(crate) const SSH_CONFIG_MARKER_END: &str = "# --- end agm ssh managed ---";

pub(crate) fn default_ssh_port() -> u16 {
    22
}

pub(crate) fn default_os_type() -> String {
    "linux".to_string()
}

pub(crate) fn default_auth_method() -> String {
    "key".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshNodeExportItem {
    #[serde(default, alias = "workerId")]
    pub worker_id: String,
    #[serde(rename = "id", default)]
    pub numeric_id: usize,
    pub alias: String,
    #[serde(alias = "ipAddress")]
    pub ip_address: String,
    pub username: String,
    #[serde(default = "default_ssh_port")]
    pub port: u16,
    #[serde(default = "default_os_type")]
    pub os: String,
    #[serde(default = "default_auth_method", alias = "authMethod")]
    pub auth_method: String,
    #[serde(default, alias = "keyPath")]
    pub key_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConnectionRecord {
    pub alias: String,
    #[serde(alias = "ipAddress")]
    pub ip_address: String,
    pub username: String,
    #[serde(default, alias = "encryptedPassword")]
    pub encrypted_password: String,
    #[serde(default, alias = "keyPath")]
    pub key_path: String,
    #[serde(default = "default_os_type")]
    pub os: String,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "osGroup")]
    pub os_group: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "osVersion")]
    pub os_version: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "buildVersion"
    )]
    pub build_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "firstRunAt")]
    pub first_run_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "createdAt")]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshNodesExportEnvelope {
    pub schema_version: String,
    pub exported_at: String,
    pub total_nodes: usize,
    pub nodes: Vec<SshNodeExportItem>,
    pub connections: Vec<SshConnectionRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NodeSyncStats {
    pub total: usize,
    pub matched: usize,
    pub updated: usize,
    pub inserted: usize,
    pub unchanged: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshKeyRecord {
    pub name: String,
    pub key_type: String,
    pub public_key: String,
    pub private_path: String,
    pub public_path: String,
    pub fingerprint: String,
    pub comment: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPublicKey {
    pub source_node: String,
    pub key_name: String,
    pub public_key: String,
    pub key_blob: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyDeployNodeReport {
    pub alias: String,
    pub ip_address: String,
    pub username: String,
    pub os: String,
    pub online: bool,
    pub keys_deployed: usize,
    pub batch_auth_verified: bool,
    pub status: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshDeploySummary {
    pub gathered_keys: Vec<DiscoveredPublicKey>,
    pub node_reports: Vec<KeyDeployNodeReport>,
    pub local_files_updated: usize,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshExecNodeResult {
    pub alias: String,
    pub target: String,
    pub ip_address: String,
    pub username: String,
    pub port: u16,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u128,
    pub success: bool,
}
