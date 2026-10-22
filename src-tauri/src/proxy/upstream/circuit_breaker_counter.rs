//! Upstream Circuit Breaker Trip Counter.
//! Tracks rolling failure counts and consecutive network dropouts per upstream endpoint,
//! automatically tripping circuit states to prevent cascading downstream delays.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Operational state of a circuit breaker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,  // Operating normally
    Open,    // Tripped, fast-failing traffic
    HalfOpen,// Probing upstream recovery
}

/// Configuration thresholds for circuit breakers.
#[derive(Debug, Clone, Copy)]
pub struct CircuitBreakerConfig {
    /// Consecutive failures required to trip the breaker.
    pub failure_threshold: u32,
    /// Duration an open breaker remains open before transitioning to HalfOpen.
    pub open_cooldown: Duration,
    /// Time window for tracking rolling failure frequency.
    pub failure_window: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            open_cooldown: Duration::from_secs(30),
            failure_window: Duration::from_secs(60),
        }
    }
}

/// Tracking record for an individual upstream endpoint.
#[derive(Debug)]
pub struct EndpointCircuitTracker {
    state: CircuitState,
    consecutive_failures: u32,
    last_failure_at: Option<Instant>,
    tripped_at: Option<Instant>,
    successes_in_half_open: u32,
}

impl Default for EndpointCircuitTracker {
    fn default() -> Self {
        Self {
            state: CircuitState::Closed,
            consecutive_failures: 0,
            last_failure_at: None,
            tripped_at: None,
            successes_in_half_open: 0,
        }
    }
}

/// Centralized manager tracking circuit breakers across all known upstream hosts.
#[derive(Debug, Default)]
pub struct CircuitBreakerCounter {
    endpoints: HashMap<String, EndpointCircuitTracker>,
    config: CircuitBreakerConfig,
}

impl CircuitBreakerCounter {
    /// Creates a new counter with given or default configuration.
    pub fn new(config: Option<CircuitBreakerConfig>) -> Self {
        Self {
            endpoints: HashMap::new(),
            config: config.unwrap_or_default(),
        }
    }

    /// Evaluates if traffic is permitted to route to an endpoint.
    pub fn can_route(&mut self, endpoint: &str) -> bool {
        let entry = self.endpoints.entry(endpoint.to_string()).or_default();

        match entry.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(tripped) = entry.tripped_at {
                    if tripped.elapsed() >= self.config.open_cooldown {
                        entry.state = CircuitState::HalfOpen;
                        entry.successes_in_half_open = 0;
                        return true;
                    }
                }
                false
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Records a successful upstream request.
    pub fn record_success(&mut self, endpoint: &str) {
        let entry = self.endpoints.entry(endpoint.to_string()).or_default();
        match entry.state {
            CircuitState::HalfOpen => {
                entry.successes_in_half_open += 1;
                if entry.successes_in_half_open >= 2 {
                    entry.state = CircuitState::Closed;
                    entry.consecutive_failures = 0;
                    entry.tripped_at = None;
                }
            }
            CircuitState::Closed => {
                entry.consecutive_failures = 0;
            }
            CircuitState::Open => {}
        }
    }

    /// Records an upstream failure (e.g. 5xx or transport timeout).
    pub fn record_failure(&mut self, endpoint: &str) -> CircuitState {
        let entry = self.endpoints.entry(endpoint.to_string()).or_default();
        let now = Instant::now();

        entry.last_failure_at = Some(now);
        entry.consecutive_failures += 1;

        if entry.consecutive_failures >= self.config.failure_threshold || entry.state == CircuitState::HalfOpen {
            entry.state = CircuitState::Open;
            entry.tripped_at = Some(now);
        }

        entry.state
    }

    /// Queries the current state of an endpoint.
    pub fn state(&self, endpoint: &str) -> CircuitState {
        self.endpoints
            .get(endpoint)
            .map(|e| e.state)
            .unwrap_or(CircuitState::Closed)
    }
}
