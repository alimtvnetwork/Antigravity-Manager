//! Multi-Endpoint Dynamic Route Balancer
//!
//! Maintains rolling error-rate scores and latency profiles across upstream endpoints,
//! dynamically adjusting weight allocations to shift traffic away from degrading nodes.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;
use tracing::debug;

/// Route health score profile.
#[derive(Debug, Clone)]
pub struct RouteHealthProfile {
    pub endpoint: String,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub consecutive_failures: u32,
    pub last_failure_at: Option<Instant>,
    pub current_weight: u32,
}

/// Dynamic balancer managing route weights.
pub struct DynamicRouteBalancer {
    routes: Mutex<HashMap<String, RouteHealthProfile>>,
    base_weight: u32,
}

impl Default for DynamicRouteBalancer {
    fn default() -> Self {
        Self::new(100)
    }
}

impl DynamicRouteBalancer {
    pub fn new(base_weight: u32) -> Self {
        Self {
            routes: Mutex::new(HashMap::new()),
            base_weight,
        }
    }

    /// Registers a route if not present.
    pub fn register_route(&self, endpoint: &str) {
        let mut map = match self.routes.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };

        map.entry(endpoint.to_string())
            .or_insert_with(|| RouteHealthProfile {
                endpoint: endpoint.to_string(),
                total_requests: 0,
                successful_requests: 0,
                consecutive_failures: 0,
                last_failure_at: None,
                current_weight: self.base_weight,
            });
    }

    /// Records request result for a route and recalculates weight.
    pub fn record_result(&self, endpoint: &str, is_success: bool) -> u32 {
        let mut map = match self.routes.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };

        if let Some(profile) = map.get_mut(endpoint) {
            profile.total_requests += 1;

            if is_success {
                profile.successful_requests += 1;
                profile.consecutive_failures = 0;
                // Gradually restore weight toward base
                profile.current_weight =
                    ((profile.current_weight * 9 + self.base_weight) / 10).max(10);
            } else {
                profile.consecutive_failures += 1;
                profile.last_failure_at = Some(Instant::now());
                // Halve weight on failure down to minimum 5
                profile.current_weight = (profile.current_weight / 2).max(5);
            }

            debug!(
                endpoint = endpoint,
                weight = profile.current_weight,
                consecutive_failures = profile.consecutive_failures,
                "Updated dynamic route weight"
            );

            profile.current_weight
        } else {
            self.base_weight
        }
    }

    /// Returns current effective weight for an endpoint.
    pub fn get_weight(&self, endpoint: &str) -> u32 {
        let map = match self.routes.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };

        map.get(endpoint)
            .map(|p| p.current_weight)
            .unwrap_or(self.base_weight)
    }
}
