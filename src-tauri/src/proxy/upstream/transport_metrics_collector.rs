//! Upstream Transport Metrics Collector
//! Records low-level network timing milestones (DNS, TCP, TLS, TTFB)
//! across upstream endpoints to detect edge network degradation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportTimings {
    pub endpoint_url: String,
    pub dns_ms: f64,
    pub tcp_ms: f64,
    pub tls_ms: f64,
    pub ttfb_ms: f64,
    pub total_ms: f64,
}

#[derive(Debug, Clone, Default)]
pub struct TransportMetricsCollector {
    averages: Arc<RwLock<HashMap<String, f64>>>,
}

impl TransportMetricsCollector {
    pub fn new() -> Self {
        Self {
            averages: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Ingests a new transport timing record and updates moving average TTFB.
    pub fn record(&self, timings: &TransportTimings) {
        if let Ok(mut guard) = self.averages.write() {
            let entry = guard
                .entry(timings.endpoint_url.clone())
                .or_insert(timings.ttfb_ms);
            *entry = (*entry * 0.8) + (timings.ttfb_ms * 0.2); // 20% weight to new sample
        }
    }

    /// Gets current average TTFB in milliseconds for an endpoint.
    pub fn average_ttfb(&self, endpoint_url: &str) -> Option<f64> {
        self.averages
            .read()
            .ok()
            .and_then(|g| g.get(endpoint_url).copied())
    }
}
