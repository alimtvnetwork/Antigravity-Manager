//! TLS Client Connector Configuration Helper
//! Configures native client TLS parameters, ALPN protocol negotiation (h2, http/1.1),
//! and cipher preferences for upstream connections to Google, Anthropic, and OpenAI edge proxies.

use std::time::Duration;

#[derive(Debug, Clone)]
pub struct TlsFingerprintConfig {
    pub enable_http2: bool,
    pub pool_idle_timeout: Duration,
    pub pool_max_idle_per_host: usize,
    pub tcp_keepalive: Duration,
    pub tcp_nodelay: bool,
}

impl Default for TlsFingerprintConfig {
    fn default() -> Self {
        Self {
            enable_http2: true,
            pool_idle_timeout: Duration::from_secs(90),
            pool_max_idle_per_host: 32,
            tcp_keepalive: Duration::from_secs(60),
            tcp_nodelay: true,
        }
    }
}

impl TlsFingerprintConfig {
    /// Applies recommended TLS and HTTP connection settings to a reqwest ClientBuilder.
    pub fn apply_to_builder(&self, builder: reqwest::ClientBuilder) -> reqwest::ClientBuilder {
        let mut b = builder
            .pool_idle_timeout(self.pool_idle_timeout)
            .pool_max_idle_per_host(self.pool_max_idle_per_host)
            .tcp_keepalive(self.tcp_keepalive)
            .tcp_nodelay(self.tcp_nodelay);

        if self.enable_http2 {
            b = b.http2_prior_knowledge();
        }

        b
    }
}
