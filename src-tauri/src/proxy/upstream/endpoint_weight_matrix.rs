//! Upstream Endpoint Weight Matrix
//! Dynamically adjusts selection weights for upstream endpoints based on
//! latency scores, success history, and transient transport errors.

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct EndpointWeightEntry {
    pub url: String,
    pub base_weight: f64,
    pub current_weight: f64,
    pub consecutive_successes: u32,
    pub consecutive_failures: u32,
}

#[derive(Debug, Clone, Default)]
pub struct EndpointWeightMatrix {
    endpoints: HashMap<String, EndpointWeightEntry>,
}

impl EndpointWeightMatrix {
    pub fn new() -> Self {
        Self {
            endpoints: HashMap::new(),
        }
    }

    /// Registers or updates an upstream endpoint with a base priority weight.
    pub fn register_endpoint(&mut self, url: impl Into<String>, base_weight: f64) {
        let u = url.into();
        self.endpoints.insert(
            u.clone(),
            EndpointWeightEntry {
                url: u,
                base_weight: base_weight.max(1.0),
                current_weight: base_weight.max(1.0),
                consecutive_successes: 0,
                consecutive_failures: 0,
            },
        );
    }

    /// Records a success for an endpoint, gradually restoring its weight toward base.
    pub fn record_success(&mut self, url: &str) {
        if let Some(entry) = self.endpoints.get_mut(url) {
            entry.consecutive_successes += 1;
            entry.consecutive_failures = 0;
            entry.current_weight = (entry.current_weight * 1.1).min(entry.base_weight);
        }
    }

    /// Records a failure for an endpoint, degrading its selection weight.
    pub fn record_failure(&mut self, url: &str) {
        if let Some(entry) = self.endpoints.get_mut(url) {
            entry.consecutive_failures += 1;
            entry.consecutive_successes = 0;
            entry.current_weight = (entry.current_weight * 0.6).max(0.1);
        }
    }

    /// Returns the currently highest-weighted healthy endpoint URL.
    pub fn best_endpoint(&self) -> Option<String> {
        self.endpoints
            .values()
            .max_by(|a, b| {
                a.current_weight
                    .partial_cmp(&b.current_weight)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|e| e.url.clone())
    }
}
