use crate::error::AppError;
use chrono::Utc;

use super::*;
use crate::modules::supabase_sync;

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
            supabase_sync::record_supabase_network_error();
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
