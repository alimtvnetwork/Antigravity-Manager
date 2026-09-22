//! Tauri IPC Commands for Supabase Cross-Node Synchronization
//! Exposes configuration, endpoint testing, schema migrations, and leases.

#![allow(dead_code)]

use crate::error::AppResult;
use crate::modules::iterative_codec;
use crate::modules::supabase_client::{SupabaseClient, SupabaseEndpoint, TableVerificationResult};
use crate::modules::supabase_schema;
use crate::modules::supabase_sync::{self, DataMigrationSummary, SupabaseConfig};
use crate::modules::workspace_lease_manager::{self, LeaseResult, WorkspaceLease};
use serde::{Deserialize, Serialize};

/// Local node runtime info for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalNodeInfo {
    pub node_id: String,
    pub node_alias: String,
    pub ip_address: String,
    pub uptime_seconds: u64,
}

#[tauri::command]
pub async fn get_supabase_config() -> AppResult<SupabaseConfig> {
    supabase_sync::load_config()
}

#[tauri::command]
pub async fn save_supabase_config(config: SupabaseConfig) -> AppResult<()> {
    let is_enabled = config.is_sync_enabled;
    supabase_sync::save_config(&config)?;

    if is_enabled {
        supabase_sync::start_sync_worker();
        crate::modules::supabase_pruner::start_pruner_worker();
    }
    Ok(())
}

#[tauri::command]
pub async fn test_supabase_endpoint(endpoint: SupabaseEndpoint) -> AppResult<bool> {
    let client = SupabaseClient::new(&endpoint)?;
    client.test_connection().await
}

#[tauri::command]
pub async fn check_supabase_endpoint_tables(
    endpoint: SupabaseEndpoint,
) -> AppResult<TableVerificationResult> {
    let client = SupabaseClient::new(&endpoint)?;
    Ok(client
        .verify_expected_tables(&endpoint.id, &endpoint.role)
        .await)
}

#[tauri::command]
pub async fn migrate_supabase_data(
    source_id: String,
    target_id: String,
) -> AppResult<DataMigrationSummary> {
    supabase_sync::migrate_database_data(&source_id, &target_id).await
}

#[tauri::command]
pub async fn get_supabase_schema_sql(role: String) -> AppResult<String> {
    Ok(supabase_schema::get_schema_sql(&role).to_string())
}

#[tauri::command]
pub async fn export_supabase_config(format_type: String, rounds: u32) -> AppResult<String> {
    let config = supabase_sync::load_config()?;
    iterative_codec::export_config_string(&config, &format_type, rounds)
}

#[tauri::command]
pub async fn import_supabase_config(content: String) -> AppResult<SupabaseConfig> {
    let config = iterative_codec::import_config_string(&content)?;
    supabase_sync::save_config(&config)?;
    Ok(config)
}

#[tauri::command]
pub async fn acquire_account_lease(
    account_id: String,
    profile_name: String,
    ttl_secs: i64,
) -> AppResult<LeaseResult> {
    workspace_lease_manager::acquire_lease(&account_id, &profile_name, ttl_secs).await
}

#[tauri::command]
pub async fn release_account_lease(account_id: String) -> AppResult<()> {
    workspace_lease_manager::release_lease(&account_id).await
}

#[tauri::command]
pub async fn list_active_account_leases() -> AppResult<Vec<WorkspaceLease>> {
    workspace_lease_manager::list_active_leases().await
}

#[tauri::command]
pub async fn get_local_node_info() -> AppResult<LocalNodeInfo> {
    let config = supabase_sync::load_config().unwrap_or_default();
    Ok(LocalNodeInfo {
        node_id: supabase_sync::get_local_node_id(),
        node_alias: config.node_alias,
        ip_address: supabase_sync::get_local_ip(),
        uptime_seconds: supabase_sync::get_uptime_seconds(),
    })
}
