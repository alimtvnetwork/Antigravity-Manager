//! Upstream Endpoint Round-Robin Health Balancer.
//! Maintains round-robin distribution across healthy upstream endpoints with circuit-breaker penalty backoff.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Single upstream endpoint target with health and circuit breaker state.
#[derive(Debug, Clone)]
pub struct UpstreamEndpoint {
    pub url: String,
    pub consecutive_failures: u32,
    pub circuit_open_until: Option<Instant>,
}

/// Thread-safe round-robin balancer with circuit breaker penalty logic.
pub struct RoundRobinBalancer {
    endpoints: Mutex<Vec<UpstreamEndpoint>>,
    cursor: AtomicUsize,
    failure_threshold: u32,
    default_cooldown: Duration,
}

impl RoundRobinBalancer {
    pub fn new(urls: Vec<String>) -> Self {
        let endpoints = urls
            .into_iter()
            .map(|url| UpstreamEndpoint {
                url,
                consecutive_failures: 0,
                circuit_open_until: None,
            })
            .collect();

        Self {
            endpoints: Mutex::new(endpoints),
            cursor: AtomicUsize::new(0),
            failure_threshold: 3,
            default_cooldown: Duration::from_secs(30),
        }
    }

    /// Selects the next available healthy upstream endpoint in round-robin sequence.
    pub fn select_endpoint(&self) -> Option<String> {
        let now = Instant::now();
        let endpoints = self.endpoints.lock().ok()?;
        if endpoints.is_empty() {
            return None;
        }

        let total = endpoints.len();
        let start_idx = self.cursor.fetch_add(1, Ordering::Relaxed) % total;

        // Search for first endpoint that is not in circuit-open state
        for i in 0..total {
            let idx = (start_idx + i) % total;
            let ep = &endpoints[idx];

            let is_circuit_open = match ep.circuit_open_until {
                Some(until) => now < until,
                None => false,
            };

            if !is_circuit_open {
                return Some(ep.url.clone());
            }
        }

        // Fallback: if all circuit breakers are open, return lowest failure endpoint
        endpoints.iter().min_by_key(|e| e.consecutive_failures).map(|e| e.url.clone())
    }

    /// Marks an endpoint request as successful, resetting its failure counter and circuit breaker.
    pub fn record_success(&self, url: &str) {
        if let Ok(mut endpoints) = self.endpoints.lock() {
            if let Some(ep) = endpoints.iter_mut().find(|e| e.url == url) {
                ep.consecutive_failures = 0;
                ep.circuit_open_until = None;
            }
        }
    }

    /// Marks an endpoint request as failed, incrementing consecutive failures and potentially opening circuit.
    pub fn record_failure(&self, url: &str, penalty: Option<Duration>) {
        if let Ok(mut endpoints) = self.endpoints.lock() {
            if let Some(ep) = endpoints.iter_mut().find(|e| e.url == url) {
                ep.consecutive_failures += 1;
                if ep.consecutive_failures >= self.failure_threshold {
                    let cooldown = penalty.unwrap_or(self.default_cooldown);
                    ep.circuit_open_until = Some(Instant::now() + cooldown);
                }
            }
        }
    }
}
