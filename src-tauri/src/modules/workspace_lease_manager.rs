//! Distributed Workspace & Account Lease Manager
//! Enforces exclusive leases on accounts to prevent cross-node concurrency collisions.

#![allow(dead_code)]

use crate::error::AppError;
use crate::modules::supabase_client::SupabaseClient;
use crate::modules::supabase_sync::{self, SupabaseConfig};
use chrono::Utc;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::RwLock;

static ACTIVE_REMOTE_LEASES: Lazy<RwLock<HashMap<String, WorkspaceLease>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Distributed workspace lease model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceLease {
    pub account_id: String,
    pub node_id: String,
    pub node_alias: String,
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
fn get_root_client(config: &SupabaseConfig) -> Option<SupabaseClient> {
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

/// Attempt to acquire an exclusive lease for an account
pub async fn acquire_lease(
    account_id: &str,
    profile_name: &str,
    ttl_secs: i64,
) -> Result<LeaseResult, AppError> {
    let config = supabase_sync::load_config()?;
    let client = match get_root_client(&config) {
        Some(c) => c,
        None => {
            // Standalone mode: Gracefully allow local acquisition without DB
            return Ok(LeaseResult {
                is_success: true,
                error_message: None,
                owner_node_id: None,
                owner_alias: None,
                expires_at: Some(Utc::now().timestamp() + ttl_secs),
            });
        }
    };

    let node_id = supabase_sync::get_local_node_id();
    let node_alias = config.node_alias;
    let now = Utc::now().timestamp();
    let expires = now + ttl_secs;

    // First attempt using atomic RPC stored function
    let rpc_payload = json!({
        "p_account_id": account_id,
        "p_node_id": node_id,
        "p_node_alias": node_alias,
        "p_profile_name": profile_name,
        "p_ttl_seconds": ttl_secs
    });

    if let Ok(rpc_resp) = client.rpc("acquire_workspace_lease", rpc_payload).await {
        let is_ok = rpc_resp
            .get("is_success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if is_ok {
            return Ok(LeaseResult {
                is_success: true,
                error_message: None,
                owner_node_id: Some(node_id),
                owner_alias: Some(node_alias),
                expires_at: Some(expires),
            });
        } else {
            return Ok(LeaseResult {
                is_success: false,
                error_message: rpc_resp
                    .get("error")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                owner_node_id: rpc_resp
                    .get("owner_node_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                owner_alias: rpc_resp
                    .get("owner_alias")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                expires_at: rpc_resp.get("expires_at").and_then(|v| v.as_i64()),
            });
        }
    }

    // Direct REST fallback check
    let query = format!("account_id=eq.{}&select=*", account_id);
    let select_res = client.select("workspace_leases", &query).await;

    if let Ok(records) = select_res {
        if let Some(arr) = records.as_array() {
            if let Some(first) = arr.first() {
                let current_owner = first.get("node_id").and_then(|v| v.as_str()).unwrap_or("");
                let current_alias = first
                    .get("node_alias")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let current_expires = first
                    .get("expires_at")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);

                if current_expires > now && current_owner != node_id {
                    return Ok(LeaseResult {
                        is_success: false,
                        error_message: Some(format!("Account held by {}", current_alias)),
                        owner_node_id: Some(current_owner.to_string()),
                        owner_alias: Some(current_alias.to_string()),
                        expires_at: Some(current_expires),
                    });
                }
            }
        }
    }

    // Upsert lease record
    let lease_payload = json!({
        "account_id": account_id,
        "node_id": node_id,
        "node_alias": node_alias,
        "profile_name": profile_name,
        "leased_at": now,
        "expires_at": expires
    });

    client
        .upsert("workspace_leases", lease_payload, "account_id")
        .await?;

    Ok(LeaseResult {
        is_success: true,
        error_message: None,
        owner_node_id: Some(node_id),
        owner_alias: Some(node_alias),
        expires_at: Some(expires),
    })
}

/// Release a lease when switching account or shutting down
pub async fn release_lease(account_id: &str) -> Result<(), AppError> {
    let config = supabase_sync::load_config()?;
    let client = match get_root_client(&config) {
        Some(c) => c,
        None => return Ok(()),
    };

    let node_id = supabase_sync::get_local_node_id();
    let query = format!("account_id=eq.{}&node_id=eq.{}", account_id, node_id);
    let _ = client.delete("workspace_leases", &query).await;
    Ok(())
}

/// Retrieve all currently active leases across all nodes
pub async fn list_active_leases() -> Result<Vec<WorkspaceLease>, AppError> {
    let config = supabase_sync::load_config()?;
    let client = match get_root_client(&config) {
        Some(c) => c,
        None => return Ok(Vec::new()),
    };

    let now = Utc::now().timestamp();
    let query = format!("expires_at=gt.{}&select=*", now);
    let resp = client.select("workspace_leases", &query).await?;

    let leases: Vec<WorkspaceLease> = serde_json::from_value(resp).unwrap_or_default();
    if let Ok(mut cache) = ACTIVE_REMOTE_LEASES.write() {
        cache.clear();
        for l in &leases {
            cache.insert(l.account_id.clone(), l.clone());
        }
    }
    Ok(leases)
}

/// Synchronously check if an account is currently leased by another active node
pub fn is_account_leased_by_other(account_id: &str) -> bool {
    let local_node = supabase_sync::get_local_node_id();
    let now = Utc::now().timestamp();
    if let Ok(cache) = ACTIVE_REMOTE_LEASES.read() {
        if let Some(lease) = cache.get(account_id) {
            if lease.expires_at > now && lease.node_id != local_node {
                return true;
            }
        }
    }
    false
}
