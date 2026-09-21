//! Supabase Synchronization & Node Registry Module
//! Manages local node identity, heartbeat pings, and instance profile state sync.

#![allow(dead_code)]

use crate::error::AppError;
use crate::modules::account;
use crate::modules::instance;
use crate::modules::supabase_client::{SupabaseClient, SupabaseEndpoint};
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
        let def = SupabaseConfig::default();
        save_config(&def)?;
        return Ok(def);
    }
    let data = fs::read_to_string(&path).map_err(|e| AppError::Io(e))?;
    let config: SupabaseConfig = serde_json::from_str(&data)
        .map_err(|e| AppError::Config(format!("Failed to parse Supabase config: {}", e)))?;
    Ok(config)
}

/// Save configuration to disk
pub fn save_config(config: &SupabaseConfig) -> Result<(), AppError> {
    let path = get_config_path()?;
    let data = serde_json::to_string_pretty(config)
        .map_err(|e| AppError::Config(format!("Failed to serialize Supabase config: {}", e)))?;
    fs::write(&path, data).map_err(|e| AppError::Io(e))?;
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

    // 2. Upsert instance profiles
    for inst in instances_to_sync {
        let is_active = inst.is_default;
        let profile_id = format!("{}_{}", node_id, inst.id);
        let profile_payload = json!({
            "id": profile_id,
            "node_id": node_id,
            "profile_name": inst.name,
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

/// Start background synchronization daemon
pub fn start_sync_worker() {
    let is_already_running = SYNC_RUNNING.swap(true, Ordering::SeqCst);
    if is_already_running {
        return;
    }

    tokio::spawn(async move {
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
