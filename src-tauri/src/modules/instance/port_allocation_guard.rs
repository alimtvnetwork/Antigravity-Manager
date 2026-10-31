// Non-Blocking Port Availability Guard (Imp-503)
// Performs a non-blocking TCP socket bind probe to ensure candidate
// remote debugging ports are free before assigning them to instance profiles.

use std::net::{SocketAddr, TcpListener};

pub struct PortAllocationGuard;

impl PortAllocationGuard {
    /// Tests whether the specified port on 127.0.0.1 is available for binding.
    pub fn is_port_available(port: u16) -> bool {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        match TcpListener::bind(addr) {
            Ok(listener) => {
                // Successfully bound; drop listener to immediately release port
                drop(listener);
                true
            }
            Err(_) => false,
        }
    }

    /// Finds the first available port in a range starting from candidate_port.
    pub fn find_available_port(start_port: u16, max_attempts: u16) -> Option<u16> {
        for offset in 0..max_attempts {
            let port = start_port.saturating_add(offset);
            if Self::is_port_available(port) {
                return Some(port);
            }
        }
        None
    }
}
