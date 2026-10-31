//! Cross-Instance CDP Port Collision Detector.
//! Scans for occupied debugging ports and allocates non-conflicting ports for parallel profiles.

use std::net::TcpListener;

pub const DEFAULT_CDP_START_PORT: u16 = 9222;

/// Checks if a TCP port is currently free on loopback.
pub fn is_port_available(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_ok()
}

/// Finds the next available loopback port starting from start_port.
pub fn find_available_cdp_port(start_port: u16, max_attempts: u16) -> Option<u16> {
    for i in 0..max_attempts {
        let candidate = start_port + i;
        if is_port_available(candidate) {
            return Some(candidate);
        }
    }
    None
}
