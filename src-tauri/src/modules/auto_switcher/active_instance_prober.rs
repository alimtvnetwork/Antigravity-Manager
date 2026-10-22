// Active Instance Readiness Prober (Imp-492)
// Verifies whether a running IDE instance is receptive to command and prompt injection
// by performing non-blocking TCP reachability checks on its listening ports.

use serde::{Deserialize, Serialize};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

pub const DEFAULT_PROBE_TIMEOUT_MS: u64 = 400;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceProbeResult {
    pub instance_id: String,
    pub port: u16,
    pub is_reachable: bool,
    pub latency_ms: u64,
    pub error: Option<String>,
}

pub struct ActiveInstanceProber;

impl ActiveInstanceProber {
    /// Non-blocking probe to verify if the instance's CDP/HTTP port is accepting connections.
    pub fn probe_port(instance_id: &str, port: u16, timeout: Duration) -> InstanceProbeResult {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let start = std::time::Instant::now();

        match TcpStream::connect_timeout(&addr, timeout) {
            Ok(_) => {
                let latency_ms = start.elapsed().as_millis() as u64;
                InstanceProbeResult {
                    instance_id: instance_id.to_string(),
                    port,
                    is_reachable: true,
                    latency_ms,
                    error: None,
                }
            }
            Err(e) => {
                let latency_ms = start.elapsed().as_millis() as u64;
                InstanceProbeResult {
                    instance_id: instance_id.to_string(),
                    port,
                    is_reachable: false,
                    latency_ms,
                    error: Some(e.to_string()),
                }
            }
        }
    }
}
