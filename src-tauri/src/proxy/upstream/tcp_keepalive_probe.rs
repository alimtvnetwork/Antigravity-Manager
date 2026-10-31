//! Upstream TCP Keep-Alive Probe Configuration
//!
//! Provides platform-level TCP keep-alive socket options (idle timeout, probe intervals,
//! probe counts) for upstream connections, detecting silently dropped TCP connections
//! across intermediate stateful NAT firewalls and cloud proxies.

use std::time::Duration;
use tracing::debug;

/// Default TCP keep-alive idle time before first probe (30 seconds).
pub const DEFAULT_TCP_KEEPALIVE_IDLE: Duration = Duration::from_secs(30);

/// Default interval between unacknowledged keep-alive probes (10 seconds).
pub const DEFAULT_TCP_KEEPALIVE_INTERVAL: Duration = Duration::from_secs(10);

/// Configuration parameters for TCP keep-alive probing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TcpKeepaliveProbeConfig {
    pub idle_time: Duration,
    pub probe_interval: Duration,
    pub probe_count: u32,
    pub enabled: bool,
}

impl Default for TcpKeepaliveProbeConfig {
    fn default() -> Self {
        Self {
            idle_time: DEFAULT_TCP_KEEPALIVE_IDLE,
            probe_interval: DEFAULT_TCP_KEEPALIVE_INTERVAL,
            probe_count: 3,
            enabled: true,
        }
    }
}

/// Applies or generates TCP keepalive parameters for reqwest / socket configuration.
pub fn configure_tcp_keepalive(
    custom_idle: Option<Duration>,
    custom_interval: Option<Duration>,
) -> TcpKeepaliveProbeConfig {
    let config = TcpKeepaliveProbeConfig {
        idle_time: custom_idle.unwrap_or(DEFAULT_TCP_KEEPALIVE_IDLE),
        probe_interval: custom_interval.unwrap_or(DEFAULT_TCP_KEEPALIVE_INTERVAL),
        probe_count: 3,
        enabled: true,
    };

    debug!(
        idle_secs = config.idle_time.as_secs(),
        interval_secs = config.probe_interval.as_secs(),
        "Configured upstream TCP keep-alive probe profile"
    );

    config
}
