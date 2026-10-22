//! Upstream Connection Keepalive Configurator.
//! Standardizes TCP keepalive, connection pool pooling boundaries, and idle socket timeouts
//! across all outbound HTTP/1.1 and HTTP/2 proxy clients.

use std::time::Duration;

/// Configuration matrix for proxy upstream connection keepalives.
#[derive(Debug, Clone, Copy)]
pub struct UpstreamKeepaliveConfig {
    /// TCP keepalive probe interval.
    pub tcp_keepalive_interval: Duration,
    /// Connection pool max idle time before closing.
    pub pool_idle_timeout: Duration,
    /// Maximum idle connections allowed per upstream host.
    pub pool_max_idle_per_host: usize,
    /// Whether HTTP/2 adaptive window sizing is enabled.
    pub http2_adaptive_window: bool,
    /// HTTP/2 ping interval to keep firewall conduits open.
    pub http2_keep_alive_interval: Duration,
    /// HTTP/2 ping timeout before considering upstream dead.
    pub http2_keep_alive_timeout: Duration,
}

impl Default for UpstreamKeepaliveConfig {
    fn default() -> Self {
        Self {
            tcp_keepalive_interval: Duration::from_secs(60),
            pool_idle_timeout: Duration::from_secs(90),
            pool_max_idle_per_host: 32,
            http2_adaptive_window: true,
            http2_keep_alive_interval: Duration::from_secs(30),
            http2_keep_alive_timeout: Duration::from_secs(10),
        }
    }
}

impl UpstreamKeepaliveConfig {
    /// Builds a hardened keepalive profile suitable for high-concurrency streaming.
    pub fn for_streaming() -> Self {
        Self {
            tcp_keepalive_interval: Duration::from_secs(30),
            pool_idle_timeout: Duration::from_secs(120),
            pool_max_idle_per_host: 64,
            http2_adaptive_window: true,
            http2_keep_alive_interval: Duration::from_secs(20),
            http2_keep_alive_timeout: Duration::from_secs(8),
        }
    }

    /// Builds a low-memory profile for lightweight or headless server deployments.
    pub fn for_low_memory() -> Self {
        Self {
            tcp_keepalive_interval: Duration::from_secs(90),
            pool_idle_timeout: Duration::from_secs(45),
            pool_max_idle_per_host: 8,
            http2_adaptive_window: false,
            http2_keep_alive_interval: Duration::from_secs(60),
            http2_keep_alive_timeout: Duration::from_secs(15),
        }
    }
}
