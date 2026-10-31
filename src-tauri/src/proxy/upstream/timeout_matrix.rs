//! Multi-Model Upstream Timeout Matrix.
//! Configures model-specific streaming connection and TTFT (time-to-first-token) timeouts.

use std::time::Duration;

pub const DEFAULT_FLASH_TTFT_SECS: u64 = 60;
pub const DEFAULT_PRO_TTFT_SECS: u64 = 120;
pub const DEFAULT_THINKING_TTFT_SECS: u64 = 180;
pub const DEFAULT_GENERAL_TTFT_SECS: u64 = 90;

/// Timeout parameters tailored to specific model execution profiles.
#[derive(Debug, Clone, Copy)]
pub struct ModelTimeoutConfig {
    pub connect_timeout: Duration,
    pub ttft_timeout: Duration,
    pub idle_timeout: Duration,
}

/// Resolves recommended timeout bounds for a model identifier.
pub fn get_timeout_for_model(model_id: &str) -> ModelTimeoutConfig {
    let lower = model_id.trim().to_lowercase();

    let ttft_secs = if lower.contains("thinking")
        || lower.contains("claude-3-7")
        || lower.contains("o1")
        || lower.contains("o3")
    {
        DEFAULT_THINKING_TTFT_SECS
    } else if lower.contains("pro") || lower.contains("opus") {
        DEFAULT_PRO_TTFT_SECS
    } else if lower.contains("flash") || lower.contains("haiku") || lower.contains("mini") {
        DEFAULT_FLASH_TTFT_SECS
    } else {
        DEFAULT_GENERAL_TTFT_SECS
    };

    ModelTimeoutConfig {
        connect_timeout: Duration::from_secs(15),
        ttft_timeout: Duration::from_secs(ttft_secs),
        idle_timeout: Duration::from_secs(45),
    }
}
