use crate::modules::email_vault_db;
use std::net::UdpSocket;

use super::*;

/// Detect local machine network IP address
pub fn detect_local_ip() -> String {
    // Non-blocking trick: bind UDP socket and connect to public DNS to inspect routing
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                return addr.ip().to_string();
            }
        }
    }
    "127.0.0.1".to_string()
}

/// Detect local machine hostname or saved custom node name
pub fn detect_machine_name() -> String {
    let local_ip = detect_local_ip();
    let has_valid_local_ip = !local_ip.is_empty() && local_ip != "127.0.0.1";

    // 1. Cluster-IP primacy: canonical GitMap cluster nodes take absolute primacy
    if has_valid_local_ip {
        for &(alias, ip, _, _) in crate::modules::ssh_manager::GITMAP_CLUSTER_NODES {
            if ip == local_ip.trim() {
                return alias.to_string();
            }
        }
        if let Ok(conns) = crate::modules::ssh_manager::load_ssh_connections() {
            if let Some(c) = conns
                .iter()
                .find(|c| c.ip_address.trim() == local_ip.trim() && !c.alias.trim().is_empty())
            {
                return c.alias.trim().to_string();
            }
        }
    }

    // 2. Explicit user setting from email_vault_db (ensuring no collision with a different remote node's IP)
    if let Ok(settings) = email_vault_db::get_notification_settings() {
        let trimmed_alias = settings.local_machine_name.trim();
        if !trimmed_alias.is_empty() {
            let is_remote_conflict = has_valid_local_ip
                && crate::modules::ssh_manager::GITMAP_CLUSTER_NODES
                    .iter()
                    .any(|&(alias, ip, _, _)| {
                        alias.eq_ignore_ascii_case(trimmed_alias) && ip != local_ip.trim()
                    });
            if !is_remote_conflict {
                return trimmed_alias.to_string();
            }
        }
    }

    // 3. Fallback to OS hostname env vars
    if let Ok(name) = std::env::var("COMPUTERNAME") {
        if !name.trim().is_empty() {
            return name.trim().to_string();
        }
    }
    if let Ok(name) = std::env::var("HOSTNAME") {
        if !name.trim().is_empty() {
            return name.trim().to_string();
        }
    }
    "antigravity-node".to_string()
}
