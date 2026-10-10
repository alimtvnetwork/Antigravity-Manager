use crate::error::AppError;
use crate::modules::instance;
use crate::modules::supabase_client::{normalize_supabase_url, SupabaseClient, SupabaseEndpoint};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use super::*;

/// Migration summary for cross-database failover transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataMigrationSummary {
    pub is_success: bool,
    pub nodes_migrated: usize,
    pub profiles_migrated: usize,
    pub leases_migrated: usize,
    pub commands_migrated: usize,
    pub message: String,
}

/// Migrate active and recent state from a source/damaged Supabase endpoint to a target endpoint
pub async fn migrate_database_data(
    source_endpoint_id: &str,
    target_endpoint_id: &str,
) -> Result<DataMigrationSummary, AppError> {
    let config = load_config()?;
    let source_ep = config
        .endpoints
        .iter()
        .find(|e| e.id == source_endpoint_id)
        .ok_or_else(|| {
            AppError::Config(format!(
                "Source endpoint '{}' not found",
                source_endpoint_id
            ))
        })?;
    let target_ep = config
        .endpoints
        .iter()
        .find(|e| e.id == target_endpoint_id)
        .ok_or_else(|| {
            AppError::Config(format!(
                "Target endpoint '{}' not found",
                target_endpoint_id
            ))
        })?;

    let source_client = SupabaseClient::new(source_ep)?;
    let target_client = SupabaseClient::new(target_ep)?;

    let mut nodes_migrated = 0;
    let mut profiles_migrated = 0;
    let mut leases_migrated = 0;
    let mut commands_migrated = 0;

    // 1. Migrate active nodes
    if let Ok(nodes_val) = source_client.select("nodes", "status=eq.online").await {
        if let Some(nodes_arr) = nodes_val.as_array() {
            for node in nodes_arr {
                if target_client
                    .upsert("nodes", node.clone(), "id")
                    .await
                    .is_ok()
                {
                    nodes_migrated += 1;
                }
            }
        }
    }

    // 2. Migrate active instance profiles
    if let Ok(prof_val) = source_client
        .select("instance_profiles", "status=eq.running")
        .await
    {
        if let Some(prof_arr) = prof_val.as_array() {
            for prof in prof_arr {
                if target_client
                    .upsert("instance_profiles", prof.clone(), "id")
                    .await
                    .is_ok()
                {
                    profiles_migrated += 1;
                }
            }
        }
    }

    // 3. Migrate active leases (unexpired)
    let now = Utc::now().timestamp();
    let lease_query = format!("expires_at=gt.{}", now);
    if let Ok(leases_val) = source_client.select("workspace_leases", &lease_query).await {
        if let Some(leases_arr) = leases_val.as_array() {
            for lease in leases_arr {
                if target_client
                    .upsert("workspace_leases", lease.clone(), "account_id")
                    .await
                    .is_ok()
                {
                    leases_migrated += 1;
                }
            }
        }
    }

    // 4. Migrate recent 50 commands
    if let Ok(cmd_val) = source_client
        .select("command_queue", "order=created_at.desc&limit=50")
        .await
    {
        if let Some(cmd_arr) = cmd_val.as_array() {
            for cmd in cmd_arr {
                if target_client
                    .upsert("command_queue", cmd.clone(), "id")
                    .await
                    .is_ok()
                {
                    commands_migrated += 1;
                }
            }
        }
    }

    let total = nodes_migrated + profiles_migrated + leases_migrated + commands_migrated;
    let msg =
        format!(
        "Successfully migrated {} records ({} nodes, {} profiles, {} leases, {} commands) to {}",
        total, nodes_migrated, profiles_migrated, leases_migrated, commands_migrated, target_ep.name
    );

    Ok(DataMigrationSummary {
        is_success: true,
        nodes_migrated,
        profiles_migrated,
        leases_migrated,
        commands_migrated,
        message: msg,
    })
}

/// Build local fallback machine telemetry when sync is disabled or Supabase is unreachable
pub fn build_local_fallback_machine() -> Vec<FleetMachineInfo> {
    let node_id = get_local_node_id();
    let config = load_config().unwrap_or_default();
    let node_alias = config.node_alias;
    let ip_address = get_local_ip();
    let uptime_seconds = get_uptime_seconds();
    let now = Utc::now().timestamp();

    let mut running_prompts_count = 0;
    if let Ok(conn) = crate::modules::repo_db::connect_db() {
        if let Ok(mut stmt) =
            conn.prepare("SELECT count(*) FROM active_prompts WHERE status = 'running'")
        {
            if let Ok(cnt) = stmt.query_row([], |row| row.get::<_, usize>(0)) {
                running_prompts_count = cnt;
            }
        }
    }

    let mut active_instances = Vec::new();
    let mut bound_emails_set = HashSet::new();
    if let Ok(reg) = crate::modules::instance::load_registry() {
        for inst in reg.instances {
            let is_running =
                crate::modules::instance::is_instance_running(&inst.id, &inst.data_dir, inst.pid);
            if let Some(ref em) = inst.bound_email {
                let trimmed = em.trim();
                if !trimmed.is_empty() {
                    bound_emails_set.insert(trimmed.to_string());
                }
            }
            active_instances.push(FleetInstanceSummary {
                profile_id: format!("{}_{}", node_id, inst.id),
                profile_name: inst.name,
                is_running,
                bound_account_id: inst.bound_account_id,
                bound_account_email: inst.bound_email,
            });
        }
    }

    let mut bound_emails: Vec<String> = bound_emails_set.into_iter().collect();
    bound_emails.sort();

    let local_machine = FleetMachineInfo {
        node_id,
        node_alias: node_alias.clone(),
        alias: node_alias,
        os_info: Some(std::env::consts::OS.to_string()),
        ip_address,
        is_online: true,
        last_heartbeat_timestamp: now,
        last_heartbeat_at: now,
        uptime_seconds,
        status: "online".to_string(),
        source: "local".to_string(),
        in_flight_prompts_count: running_prompts_count,
        total_running_prompts: running_prompts_count,
        total_instances: active_instances.len(),
        active_instances,
        bound_emails: bound_emails.clone(),
        active_accounts: bound_emails,
        leases: Vec::new(),
        is_local: true,
        ..Default::default()
    };

    vec![local_machine]
}
