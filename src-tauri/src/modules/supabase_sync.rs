//! Supabase Synchronization & Node Registry Module
//! Manages local node identity, heartbeat pings, and instance profile state sync.

#![allow(dead_code)]

use crate::error::AppError;
use crate::modules::account;
use crate::modules::instance;
use crate::modules::supabase_client::{normalize_supabase_url, SupabaseClient, SupabaseEndpoint};
use chrono::Utc;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

static PROCESS_START_TIME: Lazy<AtomicU64> =
    Lazy::new(|| AtomicU64::new(Utc::now().timestamp() as u64));

static SYNC_RUNNING: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(false));

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
    fn default() -> Self {
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

/// Global shared configuration state
static GLOBAL_CONFIG: Lazy<Arc<RwLock<Option<SupabaseConfig>>>> =
    Lazy::new(|| Arc::new(RwLock::new(None)));

/// Path to supabase_config.json
pub fn get_config_path() -> Result<PathBuf, AppError> {
    let data_dir = account::get_data_dir()
        .map_err(|e| AppError::Config(format!("Failed to get data directory: {}", e)))?;
    Ok(data_dir.join("supabase_config.json"))
}

/// Load configuration from disk
pub fn load_config() -> Result<SupabaseConfig, AppError> {
    let path = get_config_path()?;
    if !path.exists() {
        #[cfg(target_os = "windows")]
        if let Ok(appdata) = std::env::var("APPDATA") {
            let alt_path = PathBuf::from(appdata)
                .join("antigravity-manager")
                .join("supabase_config.json");
            if alt_path.exists() {
                if let Ok(data) = fs::read_to_string(&alt_path) {
                    let clean = data.trim_start_matches('\u{feff}');
                    if let Ok((mut config, _)) =
                        crate::modules::json_envelope::extract_payload::<SupabaseConfig>(clean)
                    {
                        for ep in &mut config.endpoints {
                            ep.url = normalize_supabase_url(&ep.url);
                        }
                        let _ = save_config(&config);
                        return Ok(config);
                    }
                }
            }
        }
        let def = SupabaseConfig::default();
        save_config(&def)?;
        return Ok(def);
    }
    let data = fs::read_to_string(&path).map_err(|e| AppError::Io(e))?;
    let clean = data.trim_start_matches('\u{feff}');
    let mut config: SupabaseConfig =
        match crate::modules::json_envelope::extract_payload::<SupabaseConfig>(clean) {
            Ok((cfg, _)) => cfg,
            Err(_) => serde_json::from_str(clean)
                .map_err(|e| AppError::Config(format!("Failed to parse Supabase config: {}", e)))?,
        };
    for ep in &mut config.endpoints {
        ep.url = normalize_supabase_url(&ep.url);
    }
    Ok(config)
}

/// Export configuration as standard portable JSON envelope
pub fn export_config_json(config: &SupabaseConfig) -> Result<String, AppError> {
    let envelope =
        crate::modules::json_envelope::JsonEnvelope::new("agm/supabase-endpoints", config.clone());
    serde_json::to_string_pretty(&envelope)
        .map_err(|e| AppError::Config(format!("Failed to serialize Supabase config: {}", e)))
}

/// Save configuration to disk
pub fn save_config(config: &SupabaseConfig) -> Result<(), AppError> {
    let mut clean_config = config.clone();
    for ep in &mut clean_config.endpoints {
        ep.url = normalize_supabase_url(&ep.url);
    }
    let path = get_config_path()?;
    let data = serde_json::to_string_pretty(&clean_config)
        .map_err(|e| AppError::Config(format!("Failed to serialize Supabase config: {}", e)))?;
    fs::write(&path, &data).map_err(|e| AppError::Io(e))?;

    #[cfg(target_os = "windows")]
    if let Ok(appdata) = std::env::var("APPDATA") {
        let alt_dir = PathBuf::from(appdata).join("antigravity-manager");
        if alt_dir.exists() {
            let _ = fs::write(alt_dir.join("supabase_config.json"), &data);
        }
    }

    Ok(())
}

/// Retrieve stable local node ID
pub fn get_local_node_id() -> String {
    machine_uid::get().unwrap_or_else(|_| "agm-node-local".to_string())
}

/// Retrieve local machine IP address
pub fn get_local_ip() -> String {
    if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                return addr.ip().to_string();
            }
        }
    }
    "127.0.0.1".to_string()
}

/// Uptime of this node in seconds
pub fn get_uptime_seconds() -> u64 {
    let start = PROCESS_START_TIME.load(Ordering::Relaxed);
    let now = Utc::now().timestamp() as u64;
    if now > start {
        now - start
    } else {
        0
    }
}

/// Send single heartbeat to the Root DB endpoint
pub async fn send_heartbeat(endpoint: &SupabaseEndpoint, node_alias: &str) -> Result<(), AppError> {
    let client = SupabaseClient::new(endpoint)?;
    let node_id = get_local_node_id();
    let local_ip = get_local_ip();
    let uptime = get_uptime_seconds();
    let now = Utc::now().timestamp();

    // Calculate active project count from instance registry
    let mut project_count = 1;
    let mut instances_to_sync = Vec::new();
    if let Ok(reg) = instance::load_registry() {
        project_count = reg.instances.len() as i32;
        instances_to_sync = reg.instances;
    }

    // 1. Upsert node record
    let node_payload = json!({
        "id": node_id,
        "alias": node_alias,
        "ip_address": local_ip,
        "uptime_seconds": uptime,
        "project_count": project_count,
        "last_heartbeat_at": now,
        "status": "online"
    });

    client.upsert("nodes", node_payload, "id").await?;

    // 2. Upsert instance profiles (Child of this node)
    for inst in instances_to_sync {
        let is_active = inst.is_default;
        let profile_id = format!("{}_{}", node_id, inst.id);
        let active_acc_id = inst.bound_account_id.clone().unwrap_or_default();
        let active_acc_email = inst.bound_email.clone().unwrap_or_default();
        let profile_payload = json!({
            "id": profile_id,
            "node_id": node_id,
            "profile_name": inst.name,
            "active_account_id": active_acc_id,
            "active_account_email": active_acc_email,
            "is_active": is_active,
            "quota_percent": 100,
            "status": if is_active { "running" } else { "idle" },
            "updated_at": now
        });
        let _ = client
            .upsert("instance_profiles", profile_payload, "id")
            .await;
    }

    Ok(())
}

/// Upsert one instance profile to the root Supabase endpoint and read the stored email back.
pub async fn push_and_read_instance_email(instance_id: &str) -> Result<String, AppError> {
    let config = load_config()?;
    let endpoint = config
        .endpoints
        .iter()
        .find(|ep| ep.is_enabled && ep.role == "root")
        .ok_or_else(|| AppError::Config("No enabled root Supabase endpoint".to_string()))?;
    let registry = instance::load_registry().map_err(|e| AppError::Config(e.to_string()))?;
    let inst = registry
        .instances
        .into_iter()
        .find(|item| item.id == instance_id)
        .ok_or_else(|| AppError::Config(format!("Instance '{}' is not registered", instance_id)))?;
    let client = SupabaseClient::new(endpoint)?;
    let profile_id = format!("{}_{}", get_local_node_id(), inst.id);
    let email = inst.bound_email.clone().unwrap_or_default();
    let now = Utc::now().timestamp();
    let payload = json!({
        "id": profile_id,
        "node_id": get_local_node_id(),
        "profile_name": inst.name,
        "active_account_id": inst.bound_account_id.clone().unwrap_or_default(),
        "active_account_email": email,
        "is_active": inst.is_default,
        "quota_percent": 100,
        "status": if inst.is_default { "running" } else { "idle" },
        "updated_at": now
    });
    let node_id = get_local_node_id();
    let node_payload = json!({
        "id": node_id,
        "alias": config.node_alias,
        "ip_address": get_local_ip(),
        "uptime_seconds": get_uptime_seconds(),
        "project_count": 1,
        "last_heartbeat_at": now,
        "status": "online"
    });
    let node_written = client.upsert("nodes", node_payload, "id").await?;
    if let Some(message) = postgrest_error_message(&node_written) {
        return Err(AppError::Network(
            format!("Supabase node upsert failed: {}", message),
            None,
        ));
    }
    let written = client
        .upsert("instance_profiles", payload, "id")
        .await
        .map_err(|e| AppError::Network(format!("Supabase upsert failed: {}", e), None))?;
    if let Some(message) = postgrest_error_message(&written) {
        return Err(AppError::Network(
            format!("Supabase profile upsert failed: {}", message),
            None,
        ));
    }
    let rows = client
        .select(
            "instance_profiles",
            &format!("id=eq.{}&select=active_account_email", profile_id),
        )
        .await?;
    if let Some(message) = postgrest_error_message(&rows) {
        return Err(AppError::Network(
            format!("Supabase profile read failed: {}", message),
            None,
        ));
    }
    let stored = email_from_postgrest(&written)
        .or_else(|| email_from_postgrest(&rows))
        .unwrap_or_default();
    if !stored.eq_ignore_ascii_case(&email) {
        return Err(AppError::Config(format!(
            "Supabase stored '{}' for profile '{}' but the local instance is bound to '{}'",
            stored, profile_id, email
        )));
    }
    Ok(format!("profile {} email {} confirmed", profile_id, stored))
}

fn postgrest_error_message(value: &serde_json::Value) -> Option<String> {
    let message = value.get("message").and_then(|item| item.as_str())?;
    let code = value
        .get("code")
        .and_then(|item| item.as_str())
        .unwrap_or("");
    if code.is_empty() {
        Some(message.to_string())
    } else {
        Some(format!("{} ({})", message, code))
    }
}

fn email_from_postgrest(value: &serde_json::Value) -> Option<String> {
    let row = value
        .as_array()
        .and_then(|items| items.first())
        .unwrap_or(value);
    let email = row
        .get("active_account_email")
        .and_then(|item| item.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if email.is_empty() {
        None
    } else {
        Some(email)
    }
}

/// Force an immediate heartbeat and instance registry sync to all root endpoints
pub async fn sync_local_node_now() -> Result<(), AppError> {
    let config = load_config()?;
    for ep in &config.endpoints {
        if ep.is_enabled && ep.role == "root" {
            let _ = send_heartbeat(ep, &config.node_alias).await;
        }
    }
    Ok(())
}

/// Start background synchronization daemon
pub fn start_sync_worker() {
    let is_already_running = SYNC_RUNNING.swap(true, Ordering::SeqCst);
    if is_already_running {
        return;
    }

    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

            let config = match load_config() {
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
                    let _ = send_heartbeat(ep, &config.node_alias).await;
                }
            }
        }
    });
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_supabase_config() {
        let def = SupabaseConfig::default();
        assert!(!def.is_sync_enabled);
        assert_eq!(def.heartbeat_interval_secs, 30);
        assert_eq!(def.auto_prune_root_mb, 400);
        assert_eq!(def.auto_prune_secondary_mb, 200);
        assert!(def.node_alias.starts_with("Node-"));
        assert!(def.endpoints.is_empty());
    }

    #[test]
    fn test_supabase_config_serde_and_url_normalization() {
        let json_data = r#"{
            "endpoints": [
                {
                    "id": "ep1",
                    "name": "Endpoint 1",
                    "url": "https://sample.supabase.co/rest/v1/",
                    "api_key": "sample-key",
                    "role": "root",
                    "is_enabled": true,
                    "prune_threshold_mb": 400,
                    "priority": 1,
                    "notes": null,
                    "tags": []
                }
            ],
            "node_alias": "Node-Test",
            "is_sync_enabled": true,
            "auto_prune_root_mb": 400,
            "auto_prune_secondary_mb": 200,
            "heartbeat_interval_secs": 30
        }"#;

        let mut config: SupabaseConfig = serde_json::from_str(json_data).expect("valid json");
        assert_eq!(
            config.endpoints[0].url,
            "https://sample.supabase.co/rest/v1/"
        );

        // Normalize endpoints as done in load_config
        for ep in &mut config.endpoints {
            ep.url = normalize_supabase_url(&ep.url);
        }
        assert_eq!(config.endpoints[0].url, "https://sample.supabase.co");

        // Verify re-serialization maintains normalized URL
        let serialized = serde_json::to_string(&config).expect("serialization works");
        assert!(serialized.contains("\"https://sample.supabase.co\""));
        assert!(!serialized.contains("/rest/v1/\""));
    }
}
