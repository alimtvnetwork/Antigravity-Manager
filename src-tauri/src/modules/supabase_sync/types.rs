use crate::modules::account;
use crate::modules::instance;
use crate::modules::supabase_client::{normalize_supabase_url, SupabaseClient, SupabaseEndpoint};
use chrono::Utc;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use super::*;

pub(crate) static PROCESS_START_TIME: Lazy<AtomicU64> =
    Lazy::new(|| AtomicU64::new(Utc::now().timestamp() as u64));

pub(crate) static SYNC_RUNNING: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(false));

/// Supabase sync global configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupabaseConfig {
    pub endpoints: Vec<SupabaseEndpoint>,
    pub node_alias: String,
    pub is_sync_enabled: bool,
    pub auto_prune_root_mb: u64,
    pub auto_prune_secondary_mb: u64,
    pub heartbeat_interval_secs: u64,
}

impl Default for SupabaseConfig {
    pub(crate) fn default() -> Self {
        let node_id = get_local_node_id();
        let short_id = if node_id.len() > 6 {
            &node_id[..6]
        } else {
            &node_id
        };
        Self {
            endpoints: Vec::new(),
            node_alias: format!("Node-{}", short_id),
            is_sync_enabled: false,
            auto_prune_root_mb: 400,
            auto_prune_secondary_mb: 200,
            heartbeat_interval_secs: 30,
        }
    }
}

/// Instance profile data model exposed for fleet management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetInstanceItem {
    pub instance_id: String,
    pub profile_name: String,
    pub bound_account_id: String,
    pub bound_account_email: String,
    pub is_active: bool,
    pub status: String,
    pub running_prompts_count: usize,
    pub updated_at: i64,
    pub is_leased: bool,
    pub lease_expires_at: Option<i64>,
}

/// Active child instance profile summary running under a specific node
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FleetInstanceSummary {
    pub profile_id: String,
    pub profile_name: String,
    pub is_running: bool,
    pub bound_account_id: Option<String>,
    pub bound_account_email: Option<String>,
}

/// Active account lease held by a node in the cluster
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FleetLeaseInfo {
    pub account_id: String,
    pub account_email: String,
    pub profile_name: String,
    pub leased_at: i64,
    pub expires_at: i64,
    pub is_expired: bool,
}

/// Comprehensive machine info aggregating node telemetry, profiles, and leases
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FleetMachineInfo {
    pub node_id: String,
    #[serde(default)]
    pub node_alias: String,
    #[serde(default)]
    pub alias: String,
    #[serde(default)]
    pub os_info: Option<String>,
    pub ip_address: String,
    #[serde(default)]
    pub is_online: bool,
    #[serde(default)]
    pub last_heartbeat_timestamp: i64,
    #[serde(default)]
    pub last_heartbeat_at: i64,
    pub uptime_seconds: u64,
    #[serde(default)]
    pub status: String,
    pub is_local: bool,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub in_flight_prompts_count: usize,
    #[serde(default)]
    pub total_running_prompts: usize,
    #[serde(default)]
    pub total_instances: usize,
    #[serde(default)]
    pub active_instances: Vec<FleetInstanceSummary>,
    #[serde(default)]
    pub instances: Vec<FleetInstanceItem>,
    #[serde(default)]
    pub bound_emails: Vec<String>,
    #[serde(default)]
    pub active_accounts: Vec<String>,
    #[serde(default)]
    pub leases: Vec<FleetLeaseInfo>,
}
