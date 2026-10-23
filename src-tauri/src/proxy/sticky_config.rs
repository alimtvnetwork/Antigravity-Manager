use serde::{Deserialize, Serialize};

/// Scheduling mode enumeration
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SchedulingMode {
    /// Cache-first: Lock the same account as much as possible, and wait first when the current is limited, which greatly improves the Prompt Caching hit rate
    CacheFirst,
    /// Balance mode (Balance): Lock the same account and immediately switch to an alternative account when the current is limited, taking into account the success rate and performance
    Balance,
    /// Performance-first: Pure polling mode (Round-robin), the account load is the most balanced, but does not use cache
    PerformanceFirst,
}

impl Default for SchedulingMode {
    fn default() -> Self {
        Self::Balance
    }
}

/// Sticky session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StickySessionConfig {
    /// Current scheduling mode
    pub mode: SchedulingMode,
    /// Maximum wait time in cache-first mode (seconds)
    pub max_wait_seconds: u64,
}

impl Default for StickySessionConfig {
    fn default() -> Self {
        Self {
            mode: SchedulingMode::Balance,
            max_wait_seconds: 60,
        }
    }
}
