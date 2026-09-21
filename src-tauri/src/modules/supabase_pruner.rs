//! Supabase Auto-Pruning Module
//! Guards the 500 MB free-tier boundary by pruning oldest completed logs and expired leases.

#![allow(dead_code)]

use crate::error::AppError;
use crate::modules::supabase_client::{SupabaseClient, SupabaseEndpoint};
use crate::modules::supabase_sync;
use chrono::Utc;
use once_cell::sync::Lazy;
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};

static PRUNER_RUNNING: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(false));

/// Prune completed commands and telemetry from a secondary endpoint
pub async fn prune_secondary_endpoint(
    endpoint: &SupabaseEndpoint,
    max_records_to_keep: usize,
) -> Result<usize, AppError> {
    let client = SupabaseClient::new(endpoint)?;

    // Attempt RPC call first
    let rpc_payload = json!({ "p_keep_limit": max_records_to_keep });
    if let Ok(val) = client.rpc("prune_old_commands", rpc_payload).await {
        if let Some(count) = val.as_u64() {
            return Ok(count as usize);
        }
    }

    // Direct REST fallback: query completed/failed commands in FIFO order
    let query = "status=in.(completed,failed)&order=created_at.asc&limit=50";
    let resp = client.select("command_queue", query).await?;

    let mut deleted_count = 0;
    if let Some(arr) = resp.as_array() {
        for item in arr {
            if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                let del_query = format!("id=eq.{}", id);
                let _ = client.delete("command_queue", &del_query).await;

                let tel_query = format!("command_id=eq.{}", id);
                let _ = client.delete("command_telemetry", &tel_query).await;
                deleted_count += 1;
            }
        }
    }

    Ok(deleted_count)
}

/// Prune expired leases and inactive nodes from Root DB
pub async fn prune_root_endpoint(endpoint: &SupabaseEndpoint) -> Result<(), AppError> {
    let client = SupabaseClient::new(endpoint)?;
    let now = Utc::now().timestamp();

    // 1. Delete expired leases
    let lease_query = format!("expires_at=lt.{}", now);
    let _ = client.delete("workspace_leases", &lease_query).await;

    // 2. Mark nodes inactive if no heartbeat in 10 minutes
    let stale_time = now - 600;
    let stale_query = format!("last_heartbeat_at=lt.{}&status=eq.online", stale_time);
    let status_payload = json!({ "status": "offline" });
    let _ = client.update("nodes", &stale_query, status_payload).await;

    Ok(())
}

/// Start background auto-pruning worker
pub fn start_pruner_worker() {
    let is_already_running = PRUNER_RUNNING.swap(true, Ordering::SeqCst);
    if is_already_running {
        return;
    }

    tokio::spawn(async move {
        loop {
            // Run pruner check every 5 minutes
            tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;

            let config = match supabase_sync::load_config() {
                Ok(c) => c,
                Err(_) => continue,
            };

            if !config.is_sync_enabled {
                continue;
            }

            for ep in &config.endpoints {
                if !ep.is_enabled {
                    continue;
                }
                if ep.role == "root" {
                    let _ = prune_root_endpoint(ep).await;
                } else if ep.role == "secondary" {
                    // Approximate records to keep based on threshold: 1000 records ~ 100MB
                    let keep_limit = (ep.prune_threshold_mb * 10) as usize;
                    let _ = prune_secondary_endpoint(ep, keep_limit).await;
                }
            }
        }
    });
}
