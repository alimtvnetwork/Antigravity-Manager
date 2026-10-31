use chrono::Utc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use super::*;

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

/// Authoritative local node alias resolution using Cluster-IP primacy and GitMap configuration
pub fn resolve_local_node_alias() -> String {
    // 0. Checks GitMap variables (~/.gitmap/variables.json) for explicit machine.alias / machine.name
    if let Some(home) = dirs::home_dir() {
        let var_path = home.join(".gitmap").join("variables.json");
        if var_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&var_path) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(alias) = val
                        .get("global")
                        .and_then(|g| g.get("machine.alias").or_else(|| g.get("machine.name")))
                        .and_then(|v| v.as_str())
                    {
                        let trimmed = alias.trim();
                        if !trimmed.is_empty() && !trimmed.starts_with("Node-") {
                            return trimmed.to_string();
                        }
                    }
                }
            }
        }
    }

    let local_ip = get_local_ip();
    let has_valid_ip = !local_ip.is_empty() && local_ip != "127.0.0.1";

    if has_valid_ip {
        // 1. Matches against canonical GitMap cluster nodes FIRST (Cluster-IP primacy)
        for &(alias, ip, _, _) in crate::modules::ssh_manager::GITMAP_CLUSTER_NODES {
            if ip == local_ip.trim() {
                return alias.to_string();
            }
        }

        // 2. Matches against stored SSH connections only after canonical check
        if let Ok(conns) = crate::modules::ssh_manager::load_ssh_connections() {
            if let Some(c) = conns
                .iter()
                .find(|c| c.ip_address.trim() == local_ip.trim() && !c.alias.trim().is_empty())
            {
                return c.alias.trim().to_string();
            }
        }
    }

    // 3. Falls back to user configured node_alias in Supabase config (ignoring generated Node-<digits>)
    if let Ok(cfg) = super::load_config() {
        let trimmed = cfg.node_alias.trim();
        if !trimmed.is_empty() && !trimmed.starts_with("Node-") {
            return trimmed.to_string();
        }
    }

    // 4. Falls back to email_watcher machine name detection
    let email_name = crate::modules::email_watcher::detect_machine_name();
    if !email_name.is_empty()
        && email_name != "antigravity-node"
        && !email_name.starts_with("Node-")
    {
        return email_name;
    }

    // 5. Final fallback
    if has_valid_ip && local_ip.trim() == "192.168.1.12" {
        "w3".to_string()
    } else if has_valid_ip && local_ip.trim() == "192.168.1.7" {
        "w2".to_string()
    } else if has_valid_ip && local_ip.trim() == "192.168.1.3" {
        "w1".to_string()
    } else {
        "w2".to_string()
    }
}
