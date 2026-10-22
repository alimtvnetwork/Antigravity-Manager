//! Upstream Endpoint Health Probe Cache.
//! Caches reachability and response latency with 60-second TTL to avoid redundant probe connections.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

pub const DEFAULT_PROBE_TTL_SECONDS: i64 = 60;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointProbeResult {
    pub endpoint_url: String,
    pub is_healthy: bool,
    pub latency_ms: u64,
    pub last_checked_sec: i64,
}

#[derive(Debug)]
pub struct ProbeCache {
    ttl_seconds: i64,
    entries: RwLock<HashMap<String, EndpointProbeResult>>,
}

impl Default for ProbeCache {
    fn default() -> Self {
        Self::new(None)
    }
}

impl ProbeCache {
    pub fn new(ttl: Option<i64>) -> Self {
        Self {
            ttl_seconds: ttl.unwrap_or(DEFAULT_PROBE_TTL_SECONDS),
            entries: RwLock::new(HashMap::new()),
        }
    }

    /// Queries health state if present and not expired by TTL.
    pub fn get_healthy_state(&self, endpoint_url: &str, now_sec: i64) -> Option<bool> {
        let guard = self.entries.read().ok()?;
        let entry = guard.get(endpoint_url)?;
        if now_sec - entry.last_checked_sec <= self.ttl_seconds {
            Some(entry.is_healthy)
        } else {
            None // Expired
        }
    }

    /// Records or updates probe telemetry for an endpoint.
    pub fn record_probe(&self, result: EndpointProbeResult) {
        if let Ok(mut guard) = self.entries.write() {
            guard.insert(result.endpoint_url.clone(), result);
        }
    }

    /// Clears expired entries from the cache.
    pub fn purge_expired(&self, now_sec: i64) -> usize {
        if let Ok(mut guard) = self.entries.write() {
            let before = guard.len();
            guard.retain(|_, entry| now_sec - entry.last_checked_sec <= self.ttl_seconds);
            before.saturating_sub(guard.len())
        } else {
            0
        }
    }
}
