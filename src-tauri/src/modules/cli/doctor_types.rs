use serde::{Deserialize, Serialize};

use super::*;

// -----------------------------------------------------------------------------
// Prompts & Doctor Subcommands
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorReport {
    pub status: String,
    pub node_name: String,
    pub local_ip: String,
    pub database_connectivity: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database_path: Option<String>,
    pub accounts_count: usize,
    pub instances_count: usize,
    pub proxy_gateway_status: String,
    pub antigravity_installation: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub antigravity_path: Option<String>,
    pub running_pids: Vec<u32>,
    pub checks: Vec<DoctorCheckItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorCheckItem {
    pub name: String,
    pub is_passed: bool,
    pub details: String,
}
