//! CDP Remote Port Reclaim Watchdog
//!
//! Actively probes allocated CDP remote debugging ports (e.g. 9222) after an instance process
//! exits, confirming the TCP socket has fully entered closed state before releasing the port lease
//! for reuse by subsequent instances.

use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// Probes whether a port is currently listening.
pub fn is_port_listening(port: u16) -> bool {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    TcpStream::connect_timeout(&addr, Duration::from_millis(50)).is_ok()
}

/// Waits for a port to become free with a maximum timeout.
pub fn wait_for_port_release(port: u16, max_wait: Duration) -> bool {
    let start = Instant::now();

    while start.elapsed() < max_wait {
        if !is_port_listening(port) {
            debug!(
                port = port,
                elapsed_ms = start.elapsed().as_millis(),
                "CDP port confirmed released"
            );
            return true;
        }

        std::thread::sleep(Duration::from_millis(50));
    }

    if is_port_listening(port) {
        warn!(
            port = port,
            wait_ms = max_wait.as_millis(),
            "CDP port still bound after wait timeout"
        );
        false
    } else {
        true
    }
}
