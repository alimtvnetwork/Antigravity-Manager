//! Environment profile resolver exposing active proxy gateway configuration,
//! loopback ports, Cloudflared tunnel status, and node identity.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentInfo {
    pub gateway_host: String,
    pub gateway_port: u16,
    pub is_tunnel_active: bool,
    pub tunnel_url: Option<String>,
    pub node_id: String,
    pub version: String,
    pub os_family: String,
    pub arch: String,
    pub pid: u32,
    pub timestamp: String,
}

/// Resolves active environment details for the local running instance.
pub fn get_environment_info(port: u16, tunnel_url: Option<String>) -> EnvironmentInfo {
    let node_id = match std::env::var("AGM_NODE_ID") {
        Ok(id) if !id.trim().is_empty() => id,
        _ => match hostname::get() {
            Ok(h) => h.to_string_lossy().to_string(),
            Err(_) => "local-node".to_string(),
        },
    };

    let is_tunnel_active = tunnel_url
        .as_ref()
        .map(|u| !u.trim().is_empty())
        .unwrap_or(false);

    let os_family = std::env::consts::OS.to_string();
    let arch = std::env::consts::ARCH.to_string();
    let pid = std::process::id();
    let version = env!("CARGO_PKG_VERSION").to_string();
    let timestamp = chrono::Utc::now().to_rfc3339();

    EnvironmentInfo {
        gateway_host: "127.0.0.1".to_string(),
        gateway_port: port,
        is_tunnel_active,
        tunnel_url,
        node_id,
        version,
        os_family,
        arch,
        pid,
        timestamp,
    }
}
