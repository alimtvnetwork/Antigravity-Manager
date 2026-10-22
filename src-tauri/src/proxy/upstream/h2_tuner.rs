//! HTTP/2 Connection Pool Tuner for Upstream AI Gateway.
//! Defines optimized multiplexing window sizes, stream limits, and keepalive intervals.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Http2TunerConfig {
    pub max_concurrent_streams: u32,
    pub initial_stream_window_size: u32,
    pub initial_connection_window_size: u32,
    pub keepalive_interval_secs: u64,
    pub keepalive_timeout_secs: u64,
}

/// Returns production-tuned HTTP/2 multiplexing parameters for upstream streaming clients.
pub fn get_optimal_h2_config() -> Http2TunerConfig {
    Http2TunerConfig {
        max_concurrent_streams: 64,
        initial_stream_window_size: 1024 * 1024,      // 1 MB stream window
        initial_connection_window_size: 4 * 1024 * 1024, // 4 MB connection window
        keepalive_interval_secs: 20,
        keepalive_timeout_secs: 10,
    }
}
