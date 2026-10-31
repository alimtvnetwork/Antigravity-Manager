//! Upstream Connection Pool Health Probe (Imp-545)
//! Monitors active upstream connection latency, consecutive failures, and state transitions
//! to dynamically route traffic away from failing or degraded proxy upstreams.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionHealthState {
    Healthy,
    Degraded,
    Dead,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionHealthProbe {
    pub endpoint_url: String,
    pub consecutive_failures: u32,
    pub consecutive_successes: u32,
    pub latency_history_ms: Vec<u64>,
    pub max_history_size: usize,
    pub degraded_failure_threshold: u32,
    pub dead_failure_threshold: u32,
    pub degraded_latency_threshold_ms: u64,
}

impl ConnectionHealthProbe {
    pub fn new(endpoint_url: impl Into<String>) -> Self {
        Self {
            endpoint_url: endpoint_url.into(),
            consecutive_failures: 0,
            consecutive_successes: 0,
            latency_history_ms: Vec::with_capacity(10),
            max_history_size: 10,
            degraded_failure_threshold: 2,
            dead_failure_threshold: 5,
            degraded_latency_threshold_ms: 2500,
        }
    }

    /// Records a successful connection probe and its round-trip latency.
    pub fn record_success(&mut self, latency_ms: u64) {
        self.consecutive_failures = 0;
        self.consecutive_successes = self.consecutive_successes.saturating_add(1);
        if self.latency_history_ms.len() >= self.max_history_size {
            self.latency_history_ms.remove(0);
        }
        self.latency_history_ms.push(latency_ms);
    }

    /// Records a failed connection attempt.
    pub fn record_failure(&mut self) {
        self.consecutive_successes = 0;
        self.consecutive_failures = self.consecutive_failures.saturating_add(1);
    }

    /// Computes the moving average latency over recent probes.
    pub fn average_latency_ms(&self) -> Option<u64> {
        if self.latency_history_ms.is_empty() {
            None
        } else {
            let sum: u64 = self.latency_history_ms.iter().sum();
            Some(sum / self.latency_history_ms.len() as u64)
        }
    }

    /// Evaluates current health state based on failure counts and latency thresholds.
    pub fn health_state(&self) -> ConnectionHealthState {
        if self.consecutive_failures >= self.dead_failure_threshold {
            ConnectionHealthState::Dead
        } else if self.consecutive_failures >= self.degraded_failure_threshold {
            ConnectionHealthState::Degraded
        } else if let Some(avg_lat) = self.average_latency_ms() {
            if avg_lat >= self.degraded_latency_threshold_ms {
                ConnectionHealthState::Degraded
            } else {
                ConnectionHealthState::Healthy
            }
        } else {
            ConnectionHealthState::Healthy
        }
    }

    /// Determines if the endpoint is viable for routing proxy requests.
    pub fn is_viable(&self) -> bool {
        self.health_state() != ConnectionHealthState::Dead
    }

    /// Resets health metrics to initial baseline.
    pub fn reset(&mut self) {
        self.consecutive_failures = 0;
        self.consecutive_successes = 0;
        self.latency_history_ms.clear();
    }
}
