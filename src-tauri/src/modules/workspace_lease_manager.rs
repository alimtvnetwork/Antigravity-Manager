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

/// Derive lease TTL from app config (derived from account_cooldown_minutes * 60, min 1800s, default 3600s)
pub fn get_default_lease_ttl_secs() -> i64 {
    let cooldown_mins = crate::modules::config::load_app_config()
        .map(|c| c.auto_profile_switcher.account_cooldown_minutes)
        .unwrap_or(60);
    ((cooldown_mins as i64) * 60).max(1800)
}

/// Attempt to acquire an exclusive lease for an account
pub async fn acquire_lease(
    account_id: &str,
    profile_name: &str,
    ttl_secs: i64,
) -> Result<LeaseResult, AppError> {
    let effective_ttl = if ttl_secs <= 90 {
        get_default_lease_ttl_secs()
    } else {
        ttl_secs.max(1800)
    };
    acquire_lease_with_details(account_id, "", profile_name, effective_ttl).await
}

/// Attempt to acquire an exclusive lease for an account with rich metadata
pub async fn acquire_lease_with_details(
    account_id: &str,
    account_email: &str,
    profile_name: &str,
    ttl_secs: i64,
) -> Result<LeaseResult, AppError> {
    let effective_ttl = if ttl_secs <= 90 {
        get_default_lease_ttl_secs()
    } else {
        ttl_secs.max(1800)
    };

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
                expires_at: Some(Utc::now().timestamp() + effective_ttl),
            });
        }
    };

    let node_id = supabase_sync::get_local_node_id();
    let node_alias = config.node_alias;
    let local_ip = supabase_sync::get_local_ip();
    let email_to_use = if !account_email.is_empty() {
        account_email.to_string()
    } else {
        crate::modules::account::load_account(account_id)
            .map(|a| a.email)
            .unwrap_or_default()
    };
    let now = Utc::now().timestamp();
    let expires = now + effective_ttl;

    // First attempt using atomic RPC stored function
    let rpc_payload = json!({
        "p_account_id": account_id,
        "p_node_id": node_id,
        "p_node_alias": node_alias,
        "p_profile_name": profile_name,
        "p_ttl_seconds": effective_ttl,
        "p_account_email": email_to_use,
        "p_ip_address": local_ip
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
            let owner_node_id = rpc_resp
                .get("owner_node_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let owner_alias = rpc_resp
                .get("owner_alias")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let expires_at = rpc_resp.get("expires_at").and_then(|v| v.as_i64());
            let error_message = rpc_resp
                .get("error")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .or_else(|| {
                    let alias = owner_alias.as_deref().unwrap_or("another machine");
                    let rem = expires_at.map(|e| (e - now).max(0)).unwrap_or(0);
                    Some(format!(
                        "Account is currently held by node '{}' (expires in {}s)",
                        alias, rem
                    ))
                });
            return Ok(LeaseResult {
                is_success: false,
                error_message,
                owner_node_id,
                owner_alias,
                expires_at,
            });
        }
    }

    // Direct REST fallback check
    let query = if !email_to_use.is_empty() {
        format!(
            "or=(account_id.eq.{},account_email.eq.{})&select=*",
            account_id, email_to_use
        )
    } else {
        format!("account_id=eq.{}&select=*", account_id)
    };
    let select_res = client.select("workspace_leases", &query).await;

    if let Ok(records) = select_res {
        if let Some(arr) = records.as_array() {
            for first in arr {
                let current_owner = first.get("node_id").and_then(|v| v.as_str()).unwrap_or("");
                let current_alias = first
                    .get("node_alias")
                    .and_then(|v| v.as_str())
                    .unwrap_or("another node");
                let current_expires = first
                    .get("expires_at")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);

                if current_expires > now && !current_owner.trim().eq_ignore_ascii_case(node_id.trim()) {
                    let remaining = (current_expires - now).max(0);
                    return Ok(LeaseResult {
                        is_success: false,
                        error_message: Some(format!(
                            "Account is currently held by node '{}' (expires in {}s)",
                            current_alias, remaining
                        )),
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
        "account_email": email_to_use,
        "node_id": node_id,
        "node_alias": node_alias,
        "ip_address": local_ip,
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
    let query = format!(
        "or=(expires_at.gt.{},leased_at.gt.{})&select=*",
        now,
        now - 7200
    );
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
    is_account_or_email_leased_by_other(account_id, "")
}

/// Synchronously check if an account (by ID or profile email) is currently leased by another active node.
/// Automatically marks remote leases as inactive if more than `stale_binding_timeout_hours` (default 6h, configurable 6h-10h)
/// have elapsed since `leased_at` with no ping or lease refresh.
pub fn is_account_or_email_leased_by_other(account_id: &str, email: &str) -> bool {
    let local_node = supabase_sync::get_local_node_id();
    let now = Utc::now().timestamp();
    let app_config = crate::modules::config::load_app_config();
    let stale_hours = app_config
        .as_ref()
        .map(|c| {
            c.auto_profile_switcher
                .stale_binding_timeout_hours
                .clamp(1, 24)
        })
        .unwrap_or(6);
    let cooldown_minutes = app_config
        .as_ref()
        .map(|c| {
            c.auto_profile_switcher
                .account_cooldown_minutes
                .max(c.auto_profile_switcher.account_lockout_window_minutes)
        })
        .unwrap_or(60);
    let stale_timeout_secs = (stale_hours as i64) * 3600;
    let lockout_window_secs = (cooldown_minutes as i64) * 60;
    let acc_id_clean = account_id.trim();
    let resolved_email = if !email.trim().is_empty() {
        email.trim().to_lowercase()
    } else if !acc_id_clean.is_empty() {
        crate::modules::account::load_account(acc_id_clean)
            .map(|a| a.email.trim().to_lowercase())
            .unwrap_or_default()
    } else {
        String::new()
    };
    let email_clean = resolved_email.as_str();

    if let Ok(cache) = ACTIVE_REMOTE_LEASES.read() {
        for (k, lease) in cache.iter() {
            let is_match_id = !acc_id_clean.is_empty()
                && (k.eq_ignore_ascii_case(acc_id_clean)
                    || lease.account_id.trim().eq_ignore_ascii_case(acc_id_clean)
                    || lease
                        .account_email
                        .trim()
                        .eq_ignore_ascii_case(acc_id_clean));
            let is_match_email = !email_clean.is_empty()
                && (lease.account_email.trim().to_lowercase() == email_clean
                    || lease.profile_name.trim().to_lowercase() == email_clean
                    || lease.account_id.trim().to_lowercase() == email_clean);
            let is_match = is_match_id || is_match_email;
            let is_stale_without_ping =
                lease.leased_at > 0 && (now - lease.leased_at) > stale_timeout_secs;
            let is_locked_or_unexpired = (lease.leased_at > 0
                && (now - lease.leased_at) < lockout_window_secs)
                || lease.expires_at > now;
            if is_match
                && is_locked_or_unexpired
                && !is_stale_without_ping
                && !lease.node_id.trim().eq_ignore_ascii_case(local_node.trim())
            {
                return true;
            }
        }
    }
    false
}

/// If an account or email is leased by another active node, return (node_alias, profile_name, remaining_seconds)
pub fn get_remote_lease_holder_info(
    account_id: &str,
    email: &str,
) -> Option<(String, String, i64)> {
    let local_node = supabase_sync::get_local_node_id();
    let now = Utc::now().timestamp();
    let app_config = crate::modules::config::load_app_config();
    let stale_hours = app_config
        .as_ref()
        .map(|c| {
            c.auto_profile_switcher
                .stale_binding_timeout_hours
                .clamp(1, 24)
        })
        .unwrap_or(6);
    let cooldown_minutes = app_config
        .as_ref()
        .map(|c| {
            c.auto_profile_switcher
                .account_cooldown_minutes
                .max(c.auto_profile_switcher.account_lockout_window_minutes)
        })
        .unwrap_or(60);
    let stale_timeout_secs = (stale_hours as i64) * 3600;
    let lockout_window_secs = (cooldown_minutes as i64) * 60;
    let acc_id_clean = account_id.trim();
    let resolved_email = if !email.trim().is_empty() {
        email.trim().to_lowercase()
    } else if !acc_id_clean.is_empty() {
        crate::modules::account::load_account(acc_id_clean)
            .map(|a| a.email.trim().to_lowercase())
            .unwrap_or_default()
    } else {
        String::new()
    };
    let email_clean = resolved_email.as_str();

    if let Ok(cache) = ACTIVE_REMOTE_LEASES.read() {
        for (k, lease) in cache.iter() {
            let is_match_id = !acc_id_clean.is_empty()
                && (k.eq_ignore_ascii_case(acc_id_clean)
                    || lease.account_id.trim().eq_ignore_ascii_case(acc_id_clean)
                    || lease
                        .account_email
                        .trim()
                        .eq_ignore_ascii_case(acc_id_clean));
            let is_match_email = !email_clean.is_empty()
                && (lease.account_email.trim().to_lowercase() == email_clean
                    || lease.profile_name.trim().to_lowercase() == email_clean
                    || lease.account_id.trim().to_lowercase() == email_clean);
            let is_match = is_match_id || is_match_email;
            let is_stale_without_ping =
                lease.leased_at > 0 && (now - lease.leased_at) > stale_timeout_secs;
            let is_locked_or_unexpired = (lease.leased_at > 0
                && (now - lease.leased_at) < lockout_window_secs)
                || lease.expires_at > now;
            if is_match
                && is_locked_or_unexpired
                && !is_stale_without_ping
                && !lease.node_id.trim().eq_ignore_ascii_case(local_node.trim())
            {
                let remaining = (lease.expires_at - now).max(0);
                return Some((
                    lease.node_alias.clone(),
                    lease.profile_name.clone(),
                    remaining,
                ));
            }
        }
    }
    None
}

/// Find any cached workspace lease matching account ID or email (case-insensitive)
pub fn find_cached_lease(account_id: &str, email: &str) -> Option<WorkspaceLease> {
    let acc_id_clean = account_id.trim();
    let resolved_email = if !email.trim().is_empty() {
        email.trim().to_lowercase()
    } else if !acc_id_clean.is_empty() {
        crate::modules::account::load_account(acc_id_clean)
            .map(|a| a.email.trim().to_lowercase())
            .unwrap_or_default()
    } else {
        String::new()
    };
    let email_clean = resolved_email.as_str();

    if let Ok(cache) = ACTIVE_REMOTE_LEASES.read() {
        for (k, lease) in cache.iter() {
            let is_match_id = !acc_id_clean.is_empty()
                && (k.eq_ignore_ascii_case(acc_id_clean)
                    || lease.account_id.trim().eq_ignore_ascii_case(acc_id_clean)
                    || lease
                        .account_email
                        .trim()
                        .eq_ignore_ascii_case(acc_id_clean));
            let is_match_email = !email_clean.is_empty()
                && (lease.account_email.trim().to_lowercase() == email_clean
                    || lease.profile_name.trim().to_lowercase() == email_clean
                    || lease.account_id.trim().to_lowercase() == email_clean);
            if is_match_id || is_match_email {
                return Some(lease.clone());
            }
        }
    }
    None
}

/// Check if an account or email has an active or recent lease in the remote cache within the given cooldown window.
pub fn has_active_lease_in_cooldown(account_id: &str, email: &str, cooldown_secs: i64) -> bool {
    let now = Utc::now().timestamp();
    if let Some(lease) = find_cached_lease(account_id, email) {
        if (lease.leased_at > 0 && (now - lease.leased_at) < cooldown_secs)
            || lease.expires_at > now
        {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_account_or_email_leased_by_other() {
        let local_node = supabase_sync::get_local_node_id();
        let now = Utc::now().timestamp();

        let mut cache = ACTIVE_REMOTE_LEASES.write().unwrap();
        cache.clear();

        // 1. Lease owned by local node (should not be considered "leased by other")
        cache.insert(
            "acc_local".to_string(),
            WorkspaceLease {
                account_id: "acc_local".to_string(),
                account_email: "local@example.com".to_string(),
                node_id: local_node.clone(),
                node_alias: "Node-Local".to_string(),
                ip_address: "127.0.0.1".to_string(),
                profile_name: "Profile-Local".to_string(),
                leased_at: now - 60,
                expires_at: now + 3600,
            },
        );

        // 2. Active lease owned by a remote node
        cache.insert(
            "acc_remote_active".to_string(),
            WorkspaceLease {
                account_id: "acc_remote_active".to_string(),
                account_email: "remote@example.com".to_string(),
                node_id: "node-remote-alpha".to_string(),
                node_alias: "Node-Alpha".to_string(),
                ip_address: "192.168.1.100".to_string(),
                profile_name: "Profile-Alpha".to_string(),
                leased_at: now - 60,
                expires_at: now + 3600,
            },
        );

        // 3. Expired lease owned by a remote node
        cache.insert(
            "acc_remote_expired".to_string(),
            WorkspaceLease {
                account_id: "acc_remote_expired".to_string(),
                account_email: "expired@example.com".to_string(),
                node_id: "node-remote-beta".to_string(),
                node_alias: "Node-Beta".to_string(),
                ip_address: "192.168.1.101".to_string(),
                profile_name: "Profile-Beta".to_string(),
                leased_at: now - 7200,
                expires_at: now - 60,
            },
        );

        // 4. Stale lease (> 6 hours since leased_at)
        cache.insert(
            "acc_remote_stale".to_string(),
            WorkspaceLease {
                account_id: "acc_remote_stale".to_string(),
                account_email: "stale@example.com".to_string(),
                node_id: "node-remote-gamma".to_string(),
                node_alias: "Node-Gamma".to_string(),
                ip_address: "192.168.1.102".to_string(),
                profile_name: "Profile-Gamma".to_string(),
                leased_at: now - 25000, // ~7 hours ago
                expires_at: now + 3600,
            },
        );

        // 5. Active lease matched by account_email (even when profile_name differs)
        cache.insert(
            "acc_by_email".to_string(),
            WorkspaceLease {
                account_id: "acc_by_email".to_string(),
                account_email: "team-shared@example.com".to_string(),
                node_id: "node-remote-delta".to_string(),
                node_alias: "Node-Delta".to_string(),
                ip_address: "192.168.1.103".to_string(),
                profile_name: "Profile-Different-Name".to_string(),
                leased_at: now - 100,
                expires_at: now + 3600,
            },
        );

        // 6. Recently leased by remote node within lockout window (even if expires_at has passed)
        cache.insert(
            "acc_remote_locked".to_string(),
            WorkspaceLease {
                account_id: "acc_remote_locked".to_string(),
                account_email: "locked@example.com".to_string(),
                node_id: "node-remote-epsilon".to_string(),
                node_alias: "Node-Epsilon".to_string(),
                ip_address: "192.168.1.104".to_string(),
                profile_name: "Profile-Epsilon".to_string(),
                leased_at: now - 300, // 5 min ago (< 60m lockout window)
                expires_at: now - 10,
            },
        );

        drop(cache);

        // Assertions:
        assert!(!is_account_or_email_leased_by_other("acc_local", ""));
        assert!(is_account_or_email_leased_by_other("acc_remote_active", ""));
        assert!(is_account_or_email_leased_by_other("ACC_REMOTE_ACTIVE", "")); // Case-insensitive ID
        assert!(!is_account_or_email_leased_by_other(
            "acc_remote_expired",
            ""
        ));
        assert!(!is_account_or_email_leased_by_other("acc_remote_stale", ""));
        assert!(is_account_or_email_leased_by_other(
            "some_random_id",
            "team-shared@example.com"
        ));
        assert!(is_account_or_email_leased_by_other(
            "some_random_id",
            "TEAM-SHARED@EXAMPLE.COM" // Case-insensitive email
        ));
        assert!(is_account_or_email_leased_by_other("acc_remote_locked", ""));
        assert!(!is_account_or_email_leased_by_other(
            "acc_non_existent",
            "other@example.com"
        ));
        assert!(has_active_lease_in_cooldown("acc_remote_active", "", 3600));
        assert!(has_active_lease_in_cooldown(
            "",
            "TEAM-SHARED@example.com",
            3600
        ));
        assert!(!has_active_lease_in_cooldown("acc_non_existent", "", 3600));

        // Cleanup
        let mut cache = ACTIVE_REMOTE_LEASES.write().unwrap();
        cache.clear();
    }
}
