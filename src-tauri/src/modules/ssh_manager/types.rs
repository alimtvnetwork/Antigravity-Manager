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

pub(crate) fn default_encryption_algo() -> String {
    "RSA-2048".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshNodeExportItem {
    #[serde(default, alias = "workerId")]
    pub worker_id: String,
    #[serde(rename = "id", default)]
    pub numeric_id: usize,
    pub alias: String,
    #[serde(alias = "ipAddress", alias = "ip")]
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
    #[serde(default, alias = "encryptedPassword")]
    pub encrypted_password: String,
    #[serde(default = "default_encryption_algo", alias = "encryptionAlgo")]
    pub encryption_algo: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConnectionRecord {
    pub alias: String,
    #[serde(alias = "ipAddress", alias = "ip")]
    pub ip_address: String,
    pub username: String,
    #[serde(default = "default_ssh_port")]
    pub port: u16,
    #[serde(default, alias = "encryptedPassword")]
    pub encrypted_password: String,
    #[serde(default = "default_encryption_algo", alias = "encryptionAlgo")]
    pub encryption_algo: String,
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

/// GitMap CLI and RSA-2048 SSH Vault status for UI & CLI inspection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshGitmapStatus {
    pub gitmap_installed: bool,
    pub gitmap_version: Option<String>,
    pub gitmap_path: Option<String>,
    pub rsa_vault_ready: bool,
    pub rsa_public_key_fingerprint: String,
    pub ssh_config_path: String,
    pub suggested_install_cmd: String,
    pub total_nodes: usize,
}

/// Frontend / CLI view of a configured SSH node with RSA-2048 metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshNodeRecord {
    pub id: String,
    pub alias: String,
    #[serde(default)]
    pub host: String,
    #[serde(default)]
    pub ip_address: String,
    pub port: u16,
    pub username: String,
    pub auth_method: String,
    pub key_path: Option<String>,
    pub encrypted_password: Option<String>,
    #[serde(default)]
    pub has_password: bool,
    #[serde(default)]
    pub has_encrypted_password: bool,
    #[serde(default)]
    pub encrypted_password_preview: Option<String>,
    pub encryption_algo: String,
    pub os: String,
    pub source: String,
    pub last_status: String,
}

/// Input payload for saving/updating an SSH node with RSA-2048 password encryption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSshNodeRsaPayload {
    #[serde(default)]
    pub id: Option<String>,
    pub alias: String,
    #[serde(alias = "ip", alias = "ipAddress", alias = "ip_address")]
    pub host: String,
    #[serde(default = "default_ssh_port")]
    pub port: u16,
    pub username: String,
    #[serde(default, alias = "plaintext_password", alias = "password")]
    pub raw_password: Option<String>,
    #[serde(default)]
    pub key_path: Option<String>,
    #[serde(default)]
    pub os: Option<String>,
}

/// Multi-machine instance & bound account balance record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterInstanceBalanceItem {
    pub machine_alias: String,
    pub machine_ip: String,
    #[serde(default, alias = "ip_address")]
    pub ip_address: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    pub is_local: bool,
    pub instance_id: String,
    pub instance_name: String,
    pub running: bool,
    pub bound_email: String,
    pub tier: String,
    pub quota_4h_pct: f64,
    pub quota_weekly_pct: f64,
    pub active_model: String,
    pub status_label: String,
}

/// Payload for sending a prompt to a local or remote SSH machine/instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemotePromptDispatchPayload {
    pub node_alias: String,
    pub instance_name: String,
    pub prompt_text: String,
    #[serde(default)]
    pub conversation_id: Option<String>,
}

/// Result of dispatching a prompt to a local or remote SSH machine/instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemotePromptDispatchResult {
    pub success: bool,
    pub node_alias: String,
    pub instance_name: String,
    pub dispatch_route: String,
    pub message: String,
}

/// Result of exporting system data as CSV or JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvExportResult {
    pub filename: String,
    pub mime_type: String,
    pub content: String,
    pub row_count: usize,
}
