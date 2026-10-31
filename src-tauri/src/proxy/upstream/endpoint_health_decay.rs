//! Upstream endpoint health score decay calculator.
//! Uses half-life exponential decay to allow recovered upstream endpoints
//! to gradually re-enter the active routing pool without sudden traffic spikes.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Configuration for health penalty decay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthDecayConfig {
    /// Time in seconds for penalty score to decrease by 50%.
    pub half_life_seconds: f64,
    /// Minimum penalty threshold below which penalty is completely cleared (0.0).
    pub clear_threshold: f64,
}

impl Default for HealthDecayConfig {
    fn default() -> Self {
        Self {
            half_life_seconds: 60.0,
            clear_threshold: 0.05,
        }
    }
}

/// Endpoint health tracker with time-decayed penalty score.
#[derive(Debug)]
pub struct EndpointHealthTracker {
    config: HealthDecayConfig,
    endpoint_url: String,
    accumulated_penalty: f64,
    last_penalty_time: Instant,
}

impl EndpointHealthTracker {
    pub fn new(endpoint_url: impl Into<String>, config: Option<HealthDecayConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
            endpoint_url: endpoint_url.into(),
            accumulated_penalty: 0.0,
            last_penalty_time: Instant::now(),
        }
    }

    /// Adds penalty score (e.g., +1.0 for HTTP 500, +2.0 for connection timeout).
    pub fn record_failure(&mut self, penalty: f64) {
        self.decay_in_place();
        self.accumulated_penalty += penalty;
        self.last_penalty_time = Instant::now();
    }

    /// Records a successful request, accelerating penalty recovery.
    pub fn record_success(&mut self) {
        self.decay_in_place();
        self.accumulated_penalty = (self.accumulated_penalty - 0.5).max(0.0);
    }

    /// Calculates current decayed penalty.
    pub fn current_penalty(&self) -> f64 {
        let elapsed = self.last_penalty_time.elapsed().as_secs_f64();
        let factor = 0.5f64.powf(elapsed / self.config.half_life_seconds);
        let decayed = self.accumulated_penalty * factor;
        if decayed < self.config.clear_threshold {
            0.0
        } else {
            decayed
        }
    }

    /// Health score between 0.0 (unhealthy) and 1.0 (perfectly healthy).
    pub fn health_score(&self) -> f64 {
        let penalty = self.current_penalty();
        1.0 / (1.0 + penalty)
    }

    /// Internal decay application.
    fn decay_in_place(&mut self) {
        self.accumulated_penalty = self.current_penalty();
        self.last_penalty_time = Instant::now();
    }

    pub fn endpoint_url(&self) -> &str {
        &self.endpoint_url
    }
}
