//! Auto-Switcher Rapid Quota Drain Circuit Breaker (Imp-547)
//! Monitors sudden quota drops over a sliding window. Trips safety protection when an account
//! quota drains abnormally fast (> 25% drop within 60s), shielding the account from runaways.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitBreakerState {
    Closed,   // Normal operation
    Tripped,  // Safety gate active, account suppressed
    HalfOpen, // Cooldown elapsed, probing with limited requests
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrainSnapshot {
    pub timestamp_sec: i64,
    pub quota_percentage: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrainCircuitBreaker {
    pub account_id: String,
    pub state: CircuitBreakerState,
    pub max_drop_percent: i32,
    pub window_duration_sec: i64,
    pub cooldown_duration_sec: i64,
    pub tripped_at_sec: Option<i64>,
    pub history: Vec<DrainSnapshot>,
}

impl DrainCircuitBreaker {
    pub fn new(account_id: impl Into<String>) -> Self {
        Self {
            account_id: account_id.into(),
            state: CircuitBreakerState::Closed,
            max_drop_percent: 25,
            window_duration_sec: 60,
            cooldown_duration_sec: 180,
            tripped_at_sec: None,
            history: Vec::with_capacity(16),
        }
    }

    /// Records a new quota observation and evaluates drain velocity.
    pub fn record_observation(&mut self, percentage: i32, now_sec: i64) -> CircuitBreakerState {
        // Prune snapshots older than sliding window
        let cutoff = now_sec.saturating_sub(self.window_duration_sec);
        self.history.retain(|s| s.timestamp_sec >= cutoff);

        // Check if tripped and cooldown has elapsed
        if self.state == CircuitBreakerState::Tripped {
            if let Some(tripped_at) = self.tripped_at_sec {
                if now_sec.saturating_sub(tripped_at) >= self.cooldown_duration_sec {
                    self.state = CircuitBreakerState::HalfOpen;
                }
            }
        }

        // Evaluate drop across sliding window if closed or half-open
        if let Some(oldest) = self.history.first() {
            let drop = oldest.quota_percentage.saturating_sub(percentage);
            if drop >= self.max_drop_percent {
                self.state = CircuitBreakerState::Tripped;
                self.tripped_at_sec = Some(now_sec);
            } else if self.state == CircuitBreakerState::HalfOpen && drop <= 5 {
                // Stabilized after half-open probe
                self.state = CircuitBreakerState::Closed;
                self.tripped_at_sec = None;
            }
        }

        self.history.push(DrainSnapshot {
            timestamp_sec: now_sec,
            quota_percentage: percentage,
        });

        self.state
    }

    /// Evaluates whether the account is eligible for candidate selection.
    pub fn is_eligible(&self, now_sec: i64) -> bool {
        match self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::HalfOpen => true,
            CircuitBreakerState::Tripped => {
                if let Some(tripped_at) = self.tripped_at_sec {
                    now_sec.saturating_sub(tripped_at) >= self.cooldown_duration_sec
                } else {
                    false
                }
            }
        }
    }

    /// Resets circuit breaker to closed baseline.
    pub fn reset(&mut self) {
        self.state = CircuitBreakerState::Closed;
        self.tripped_at_sec = None;
        self.history.clear();
    }
}
