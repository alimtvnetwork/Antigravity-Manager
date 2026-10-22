//! HTTP/2 Window Size & Concurrency Flow Controller
//!
//! Configures adaptive HTTP/2 flow control parameters, initial stream window sizes,
//! connection window sizes, and max concurrent stream limits to prevent buffer stalls
//! and backpressure deadlocks during large streaming responses.

use tracing::debug;

/// Default initial stream-level window size (1MB).
pub const DEFAULT_INITIAL_STREAM_WINDOW_SIZE: u32 = 1024 * 1024;

/// Default initial connection-level window size (8MB).
pub const DEFAULT_INITIAL_CONNECTION_WINDOW_SIZE: u32 = 8 * 1024 * 1024;

/// Default maximum concurrent streams limit per connection.
pub const DEFAULT_MAX_CONCURRENT_STREAMS: u32 = 128;

/// Minimum allowable stream window size (64KB).
pub const MIN_STREAM_WINDOW_SIZE: u32 = 64 * 1024;

/// Maximum allowable stream window size (16MB).
pub const MAX_STREAM_WINDOW_SIZE: u32 = 16 * 1024 * 1024;

/// Configuration options for HTTP/2 upstream client flow control.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Http2FlowControlConfig {
    /// Initial window size for individual HTTP/2 streams.
    pub stream_window_size: u32,
    /// Initial window size for the entire HTTP/2 connection.
    pub connection_window_size: u32,
    /// Maximum concurrent active streams allowed per connection.
    pub max_concurrent_streams: u32,
    /// Adaptive auto-tuning enabled flag.
    pub adaptive_tuning: bool,
}

impl Default for Http2FlowControlConfig {
    fn default() -> Self {
        Self {
            stream_window_size: DEFAULT_INITIAL_STREAM_WINDOW_SIZE,
            connection_window_size: DEFAULT_INITIAL_CONNECTION_WINDOW_SIZE,
            max_concurrent_streams: DEFAULT_MAX_CONCURRENT_STREAMS,
            adaptive_tuning: true,
        }
    }
}

/// Evaluates and tunes HTTP/2 flow control parameters based on expected payload sizes
/// or concurrency demands.
pub fn tune_http2_flow_control(
    estimated_payload_bytes: Option<usize>,
    concurrency_hint: Option<u32>,
) -> Http2FlowControlConfig {
    let mut config = Http2FlowControlConfig::default();

    // Adapt stream window size based on estimated payload
    if let Some(payload_bytes) = estimated_payload_bytes {
        if payload_bytes > 4 * 1024 * 1024 {
            // For large streaming responses (>4MB), expand stream window to 4MB or max
            config.stream_window_size = (payload_bytes as u32).clamp(MIN_STREAM_WINDOW_SIZE, MAX_STREAM_WINDOW_SIZE);
            config.connection_window_size = (config.stream_window_size * 4).max(DEFAULT_INITIAL_CONNECTION_WINDOW_SIZE);
        } else if payload_bytes < 128 * 1024 {
            // For lightweight calls (<128KB), keep stream window conservative
            config.stream_window_size = (128 * 1024).clamp(MIN_STREAM_WINDOW_SIZE, MAX_STREAM_WINDOW_SIZE);
        }
    }

    // Adapt concurrent stream capacity
    if let Some(concurrent_count) = concurrency_hint {
        config.max_concurrent_streams = concurrent_count.clamp(16, 512);
        // Ensure connection window size is scaled proportionally
        let required_conn_window = config.stream_window_size.saturating_mul(config.max_concurrent_streams.min(8));
        config.connection_window_size = config.connection_window_size.max(required_conn_window);
    }

    debug!(
        stream_window = config.stream_window_size,
        connection_window = config.connection_window_size,
        max_concurrent = config.max_concurrent_streams,
        "Configured HTTP/2 flow control profile"
    );

    config
}
