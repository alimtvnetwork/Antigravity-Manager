//! Remote debugging port allocator for multi-instance CDP isolation.
//! Probes local TCP sockets within range [9222, 9322] to allocate collision-free
//! debugging ports for concurrently running Antigravity IDE instances.

use std::net::TcpListener;

pub const DEFAULT_CDP_PORT_START: u16 = 9222;
pub const DEFAULT_CDP_PORT_END: u16 = 9322;

pub struct RemoteDebuggingPortAllocator;

impl RemoteDebuggingPortAllocator {
    /// Attempts to allocate the first free TCP port in the specified range.
    pub fn allocate_free_port(start: Option<u16>, end: Option<u16>) -> Option<u16> {
        let p_start = start.unwrap_or(DEFAULT_CDP_PORT_START);
        let p_end = end.unwrap_or(DEFAULT_CDP_PORT_END);

        for port in p_start..=p_end {
            if Self::is_port_available(port) {
                return Some(port);
            }
        }
        None
    }

    /// Checks if a specific port is currently available for binding.
    pub fn is_port_available(port: u16) -> bool {
        // Test binding on localhost
        match TcpListener::bind(("127.0.0.1", port)) {
            Ok(listener) => {
                // Drop immediately to release the port
                drop(listener);
                true
            }
            Err(_) => false,
        }
    }
}
