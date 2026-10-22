//! Route latency exponential smoother.
//! Computes exponential moving averages (EMA) over upstream latency samples
//! to dampen transient network jitter and provide stable routing weights.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencySmootherConfig {
    /// Smoothing factor alpha (typically 0.1 to 0.3). Higher values react faster to recent changes.
    pub alpha: f64,
    /// Default baseline latency in milliseconds for new routes.
    pub initial_latency_ms: f64,
}

impl Default for LatencySmootherConfig {
    fn default() -> Self {
        Self {
            alpha: 0.2,
            initial_latency_ms: 150.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LatencyExponentialSmoother {
    config: LatencySmootherConfig,
    smoothed_value_ms: f64,
    sample_count: usize,
}

impl LatencyExponentialSmoother {
    pub fn new(config: Option<LatencySmootherConfig>) -> Self {
        let cfg = config.unwrap_or_default();
        let init = cfg.initial_latency_ms;
        Self {
            config: cfg,
            smoothed_value_ms: init,
            sample_count: 0,
        }
    }

    /// Records a newly observed round-trip latency sample.
    pub fn record_sample(&mut self, sample_ms: f64) {
        let sample = sample_ms.max(1.0);
        if self.sample_count == 0 {
            self.smoothed_value_ms = sample;
        } else {
            self.smoothed_value_ms =
                (self.config.alpha * sample) + ((1.0 - self.config.alpha) * self.smoothed_value_ms);
        }
        self.sample_count += 1;
    }

    /// Returns the current smoothed latency in milliseconds.
    pub fn smoothed_latency(&self) -> f64 {
        self.smoothed_value_ms
    }

    /// Returns the total number of samples processed.
    pub fn samples_processed(&self) -> usize {
        self.sample_count
    }
}
