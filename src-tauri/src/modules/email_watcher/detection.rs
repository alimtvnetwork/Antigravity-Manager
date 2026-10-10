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
    if let Ok(settings) = email_vault_db::get_notification_settings() {
        if !settings.local_machine_name.trim().is_empty() {
            return settings.local_machine_name.trim().to_string();
        }
    }
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
