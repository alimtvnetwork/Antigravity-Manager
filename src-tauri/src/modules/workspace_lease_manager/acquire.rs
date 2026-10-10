use crate::error::AppError;
use chrono::Utc;
use serde_json::json;

use super::*;
use crate::modules::supabase_sync;

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

                if current_expires > now
                    && !current_owner.trim().eq_ignore_ascii_case(node_id.trim())
                {
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
