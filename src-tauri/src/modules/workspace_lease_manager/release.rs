use crate::error::AppError;
use chrono::Utc;
use serde_json::json;

use super::*;
use crate::modules::supabase_sync;

/// Renew an active exclusive lease owned by this node (heartbeat renewal)
pub async fn renew_lease(account_id: &str, ttl_secs: i64) -> Result<bool, AppError> {
    let config = supabase_sync::load_config()?;
    let client = match get_root_client(&config) {
        Some(c) => c,
        None => return Ok(true),
    };

    let node_id = supabase_sync::get_local_node_id();
    let now = Utc::now().timestamp();
    let effective_ttl = if ttl_secs <= 90 {
        get_default_lease_ttl_secs()
    } else {
        ttl_secs.max(1800)
    };
    let new_expires = now + effective_ttl;

    let payload = json!({
        "expires_at": new_expires,
        "leased_at": now
    });

    let query = format!("account_id=eq.{}&node_id=eq.{}", account_id, node_id);
    let patch_res = client.patch("workspace_leases", &query, payload).await;
    match patch_res {
        Ok(_) => {
            tracing::debug!("Successfully renewed lease for {account_id} until {new_expires}");
            Ok(true)
        }
        Err(e) => {
            tracing::warn!("Failed to renew lease for {account_id}: {e}");
            Ok(false)
        }
    }
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
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(client.delete("workspace_leases", &query).await, "delete");
    Ok(())
}

/// Retrieve all currently active leases across all nodes
pub async fn list_active_leases() -> Result<Vec<WorkspaceLease>, AppError> {
    if supabase_sync::is_supabase_circuit_open() {
        if let Ok(cache) = ACTIVE_REMOTE_LEASES.read() {
            return Ok(cache.values().cloned().collect());
        }
        return Ok(Vec::new());
    }

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
    let resp = match client.select("workspace_leases", &query).await {
        Ok(r) => r,
        Err(e) => {
            if let Ok(cache) = ACTIVE_REMOTE_LEASES.read() {
                return Ok(cache.values().cloned().collect());
            }
            return Err(e);
        }
    };

    let leases: Vec<WorkspaceLease> = serde_json::from_value(resp).unwrap_or_default();
    if let Ok(mut cache) = ACTIVE_REMOTE_LEASES.write() {
        cache.clear();
        for l in &leases {
            cache.insert(l.account_id.clone(), l.clone());
        }
    }
    Ok(leases)
}
