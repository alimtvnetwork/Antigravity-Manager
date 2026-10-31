//! Upstream Circuit Breaker Statistics Recorder
//! Tracks error rates, consecutive failures, and state transitions (Closed -> Open -> HalfOpen)
//! to protect upstream AI endpoints from being hammered during outages.

use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerStats {
    pub state: CircuitState,
    pub consecutive_failures: u32,
    pub failure_threshold: u32,
    pub recovery_timeout: Duration,
    pub last_failure_at: Option<Instant>,
    pub half_open_success_needed: u32,
    pub half_open_successes: u32,
}

impl CircuitBreakerStats {
    pub fn new(failure_threshold: u32, recovery_timeout: Duration) -> Self {
        Self {
            state: CircuitState::Closed,
            consecutive_failures: 0,
            failure_threshold,
            recovery_timeout,
            last_failure_at: None,
            half_open_success_needed: 3,
            half_open_successes: 0,
        }
    }

    /// Evaluates if a request should be allowed through the circuit breaker.
    pub fn can_attempt(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last_fail) = self.last_failure_at {
                    if last_fail.elapsed() >= self.recovery_timeout {
                        self.state = CircuitState::HalfOpen;
                        self.half_open_successes = 0;
                        true
                    } else {
                        false
                    }
                } else {
                    self.state = CircuitState::Closed;
                    true
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Records a successful upstream request.
    pub fn record_success(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.consecutive_failures = 0;
            }
            CircuitState::HalfOpen => {
                self.half_open_successes += 1;
                if self.half_open_successes >= self.half_open_success_needed {
                    self.state = CircuitState::Closed;
                    self.consecutive_failures = 0;
                    self.half_open_successes = 0;
                }
            }
            CircuitState::Open => {}
        }
    }

    /// Records a failed upstream request.
    pub fn record_failure(&mut self) {
        self.consecutive_failures += 1;
        self.last_failure_at = Some(Instant::now());

        match self.state {
            CircuitState::Closed => {
                if self.consecutive_failures >= self.failure_threshold {
                    self.state = CircuitState::Open;
                }
            }
            CircuitState::HalfOpen => {
                self.state = CircuitState::Open;
                self.half_open_successes = 0;
            }
            CircuitState::Open => {}
        }
    }

    /// Explicitly resets the circuit breaker to clean Closed state.
    pub fn reset(&mut self) {
        self.state = CircuitState::Closed;
        self.consecutive_failures = 0;
        self.half_open_successes = 0;
        self.last_failure_at = None;
    }
}
