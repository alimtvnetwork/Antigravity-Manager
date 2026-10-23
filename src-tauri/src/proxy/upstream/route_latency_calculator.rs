//! Proxy Route Latency Weight Calculator.
//! Calculates dynamic route weights using Exponential Moving Average (EMA) latencies,
//! directing traffic away from degrading network paths toward faster upstream nodes.

use std::collections::HashMap;
use std::time::Duration;

/// Configuration for latency weighting.
#[derive(Debug, Clone, Copy)]
pub struct LatencyWeightConfig {
    /// Smoothing factor alpha for EMA calculation (0.0 to 1.0, default 0.25).
    pub alpha: f64,
    /// Baseline expected latency for normalization.
    pub baseline_latency_ms: f64,
}

impl Default for LatencyWeightConfig {
    fn default() -> Self {
        Self {
            alpha: 0.25,
            baseline_latency_ms: 300.0,
        }
    }
}

/// Rolling latency tracker for an upstream endpoint.
#[derive(Debug)]
pub struct RouteLatencyStats {
    pub ema_latency_ms: f64,
    pub total_samples: u64,
}

/// Latency calculator managing rolling statistics per upstream route.
#[derive(Debug, Default)]
pub struct RouteLatencyCalculator {
    routes: HashMap<String, RouteLatencyStats>,
    config: LatencyWeightConfig,
}

impl RouteLatencyCalculator {
    /// Creates a new calculator with default or custom config.
    pub fn new(config: Option<LatencyWeightConfig>) -> Self {
        Self {
            routes: HashMap::new(),
            config: config.unwrap_or_default(),
        }
    }

    /// Records an observed round-trip response duration for an endpoint.
    pub fn record_sample(&mut self, endpoint: &str, sample_duration: Duration) {
        let sample_ms = sample_duration.as_millis() as f64;
        let entry = self
            .routes
            .entry(endpoint.to_string())
            .or_insert_with(|| RouteLatencyStats {
                ema_latency_ms: sample_ms,
                total_samples: 0,
            });

        entry.total_samples += 1;
        // EMA: S_t = alpha * Y_t + (1 - alpha) * S_{t-1}
        entry.ema_latency_ms =
            (self.config.alpha * sample_ms) + ((1.0 - self.config.alpha) * entry.ema_latency_ms);
    }

    /// Computes a normalized weight factor (0.1 to 2.0) for routing decisions.
    /// Lower latency yields higher weight.
    pub fn compute_route_weight(&self, endpoint: &str) -> f64 {
        let stats = match self.routes.get(endpoint) {
            Some(s) => s,
            None => return 1.0, // Default baseline weight for unobserved routes
        };

        if stats.ema_latency_ms <= 0.0 {
            return 1.0;
        }

        // Relative performance: baseline / observed
        let ratio = self.config.baseline_latency_ms / stats.ema_latency_ms;
        ratio.clamp(0.1, 2.0)
    }

    /// Queries the current EMA latency in milliseconds.
    pub fn get_ema_latency(&self, endpoint: &str) -> Option<f64> {
        self.routes.get(endpoint).map(|s| s.ema_latency_ms)
    }
}
