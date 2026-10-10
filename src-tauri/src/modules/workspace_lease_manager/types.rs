use crate::modules::supabase_client::SupabaseClient;
use crate::modules::supabase_sync::{self, SupabaseConfig};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

use super::*;

pub(crate) static ACTIVE_REMOTE_LEASES: Lazy<RwLock<HashMap<String, WorkspaceLease>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Distributed workspace lease model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceLease {
    pub account_id: String,
    #[serde(default)]
    pub account_email: String,
    pub node_id: String,
    pub node_alias: String,
    #[serde(default)]
    pub ip_address: String,
    pub profile_name: String,
    pub leased_at: i64,
    pub expires_at: i64,
}

/// Result of a lease acquisition attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaseResult {
    pub is_success: bool,
    pub error_message: Option<String>,
    pub owner_node_id: Option<String>,
    pub owner_alias: Option<String>,
    pub expires_at: Option<i64>,
}

/// Find the primary active Root DB client if configured
pub(crate) fn get_root_client(config: &SupabaseConfig) -> Option<SupabaseClient> {
    if !config.is_sync_enabled {
        return None;
    }
    for ep in &config.endpoints {
        if !ep.is_enabled {
            continue;
        }
        if ep.role == "root" {
            if let Ok(client) = SupabaseClient::new(ep) {
                return Some(client);
            }
        }
    }
    None
}

/// Derive lease TTL from app config (derived from account_cooldown_minutes * 60, min 1800s, default 3600s)
pub fn get_default_lease_ttl_secs() -> i64 {
    let cooldown_mins = crate::modules::config::load_app_config()
        .map(|c| c.auto_profile_switcher.account_cooldown_minutes)
        .unwrap_or(60);
    ((cooldown_mins as i64) * 60).max(1800)
}
