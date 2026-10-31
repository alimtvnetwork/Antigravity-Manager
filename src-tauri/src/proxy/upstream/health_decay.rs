//! Upstream Endpoint Health Decay Scorer.
//! Calculates decaying health scores (0.0 to 1.0) based on error frequency and latency percentiles.

use serde::{Deserialize, Serialize};

/// Metrics snapshot for an upstream endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointHealthMetrics {
    pub total_requests: u64,
    pub failure_count: u64,
    pub avg_latency_ms: f64,
}

/// Computes a composite health score in range [0.0, 1.0] where 1.0 is optimal.
pub fn calculate_endpoint_health(metrics: &EndpointHealthMetrics) -> f64 {
    if metrics.total_requests == 0 {
        return 1.0;
    }

    // Success rate factor [0.0, 1.0]
    let success_count = metrics.total_requests.saturating_sub(metrics.failure_count);
    let success_rate = success_count as f64 / metrics.total_requests as f64;

    // Latency penalty factor: 100ms or lower = 1.0, 2000ms or higher = 0.0
    let latency_factor = (1.0 - (metrics.avg_latency_ms - 100.0) / 1900.0).clamp(0.0, 1.0);

    // Weighted composite: 75% success rate, 25% latency
    let composite = (0.75 * success_rate) + (0.25 * latency_factor);
    (composite * 1000.0).round() / 1000.0
}
