//! Instance Health Check Probe Runner.
//! Performs lightweight loopback checks against active CDP debugging ports to verify instance readiness.

use serde::{Deserialize, Serialize};
use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, Instant};

/// Outcome of an instance loopback readiness probe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceHealthCheck {
    pub is_ready: bool,
    pub port: u16,
    pub latency_ms: u64,
}

/// Probes whether the instance's CDP debugging port is listening and responsive.
pub fn probe_instance_cdp_readiness(port: u16, timeout_ms: Option<u64>) -> InstanceHealthCheck {
    let start = Instant::now();
    let timeout = Duration::from_millis(timeout_ms.unwrap_or(200));
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let is_ready = match TcpStream::connect_timeout(&addr, timeout) {
        Ok(stream) => {
            let _ = stream.set_read_timeout(Some(timeout));
            let _ = stream.set_write_timeout(Some(timeout));
            true
        }
        Err(_) => false,
    };

    InstanceHealthCheck {
        is_ready,
        port,
        latency_ms: start.elapsed().as_millis() as u64,
    }
}
