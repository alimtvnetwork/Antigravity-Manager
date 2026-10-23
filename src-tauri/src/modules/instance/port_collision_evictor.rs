//! Port Collision Immediate Evictor.
//! Tests whether a proposed CDP remote debugging port (e.g. 9222) is actively occupied,
//! automatically evicting collisions and selecting an alternative vacant port in the safe range (9222-9399).

use std::net::{SocketAddr, TcpListener};

/// Range of allowed debugging ports.
pub const CDP_PORT_MIN: u16 = 9222;
pub const CDP_PORT_MAX: u16 = 9399;

/// Tests if a local TCP port is immediately bindable without conflict.
pub fn is_port_available(port: u16) -> bool {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    TcpListener::bind(addr).is_ok()
}

/// Evicts port collisions by probing the requested port and scanning for next vacant port if taken.
pub fn resolve_collision_free_port(requested_port: u16) -> Option<u16> {
    if is_port_available(requested_port) {
        return Some(requested_port);
    }

    // Port collision detected: scan forward within safe range
    for port in requested_port + 1..=CDP_PORT_MAX {
        if is_port_available(port) {
            return Some(port);
        }
    }

    // Wrap around to search from minimum
    for port in CDP_PORT_MIN..requested_port {
        if is_port_available(port) {
            return Some(port);
        }
    }

    None
}
