//! Sliding window quota velocity guard.
//! Tracks rolling 5-minute quota consumption and calculates projected time to exhaustion.
//! Blocks rotation into candidate accounts that are burning quota too fast.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct ConsumptionEvent {
    timestamp: Instant,
    tokens: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VelocityGuardVerdict {
    pub is_safe_for_switch: bool,
    pub rolling_velocity_tokens_per_sec: f64,
    pub projected_exhaustion_secs: Option<f64>,
    pub warning_message: Option<String>,
}

pub struct SlidingVelocityGuard {
    window_duration: Duration,
    minimum_exhaustion_safety_secs: f64,
    events: VecDeque<ConsumptionEvent>,
}

impl SlidingVelocityGuard {
    pub fn new(window_duration: Option<Duration>, min_safety_secs: Option<f64>) -> Self {
        Self {
            window_duration: window_duration.unwrap_or(Duration::from_secs(300)), // 5 minutes
            minimum_exhaustion_safety_secs: min_safety_secs.unwrap_or(60.0), // 60 seconds minimum runway
            events: VecDeque::new(),
        }
    }

    /// Records token consumption for a completed request.
    pub fn record_consumption(&mut self, tokens: f64) {
        let now = Instant::now();
        self.events.push_back(ConsumptionEvent {
            timestamp: now,
            tokens,
        });
        self.prune_stale(now);
    }

    /// Evaluates if an account with the given remaining tokens is safe to switch into.
    pub fn evaluate_safety(&mut self, remaining_tokens: f64) -> VelocityGuardVerdict {
        let now = Instant::now();
        self.prune_stale(now);

        let total_tokens: f64 = self.events.iter().map(|e| e.tokens).sum();
        let elapsed_secs = if let Some(oldest) = self.events.front() {
            now.duration_since(oldest.timestamp).as_secs_f64().max(1.0)
        } else {
            1.0
        };

        let velocity = total_tokens / elapsed_secs;

        if velocity <= 0.001 {
            return VelocityGuardVerdict {
                is_safe_for_switch: true,
                rolling_velocity_tokens_per_sec: 0.0,
                projected_exhaustion_secs: None,
                warning_message: None,
            };
        }

        let projected_secs = remaining_tokens / velocity;
        let is_safe = projected_secs >= self.minimum_exhaustion_safety_secs;

        let warning = if !is_safe {
            Some(format!(
                "High consumption velocity ({:.1} tok/s) would exhaust remaining tokens in {:.1}s (< {:.0}s threshold)",
                velocity, projected_secs, self.minimum_exhaustion_safety_secs
            ))
        } else {
            None
        };

        VelocityGuardVerdict {
            is_safe_for_switch: is_safe,
            rolling_velocity_tokens_per_sec: velocity,
            projected_exhaustion_secs: Some(projected_secs),
            warning_message: warning,
        }
    }

    fn prune_stale(&mut self, now: Instant) {
        while let Some(front) = self.events.front() {
            if now.duration_since(front.timestamp) > self.window_duration {
                self.events.pop_front();
            } else {
                break;
            }
        }
    }
}
