//! Stream burst rate limiter for token-bucket chunk pacing.
//! Prevents downstream network congestion and client buffer exhaustion
//! during high-throughput SSE token delivery.

use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Configuration for the streaming burst rate limiter.
#[derive(Debug, Clone)]
pub struct StreamBurstConfig {
    /// Maximum burst capacity in tokens / chunks.
    pub capacity: f64,
    /// Token refill rate per second.
    pub refill_rate_per_sec: f64,
}

impl Default for StreamBurstConfig {
    fn default() -> Self {
        Self {
            capacity: 50.0,
            refill_rate_per_sec: 25.0,
        }
    }
}

/// Token bucket limiter tuned for streaming event streams.
pub struct StreamBurstLimiter {
    config: StreamBurstConfig,
    state: Mutex<BucketState>,
}

#[derive(Debug)]
struct BucketState {
    tokens: f64,
    last_update: Instant,
}

impl StreamBurstLimiter {
    /// Creates a new burst limiter with specified configuration.
    pub fn new(config: StreamBurstConfig) -> Self {
        let initial_tokens = config.capacity;
        Self {
            config,
            state: Mutex::new(BucketState {
                tokens: initial_tokens,
                last_update: Instant::now(),
            }),
        }
    }

    /// Attempts to consume the specified number of tokens immediately.
    /// Returns true if tokens were successfully consumed, false if limited.
    pub fn try_consume(&self, tokens: f64) -> bool {
        let mut guard = match self.state.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };

        self.refill_locked(&mut guard);

        if guard.tokens >= tokens {
            guard.tokens -= tokens;
            true
        } else {
            false
        }
    }

    /// Computes the duration to wait until the required tokens become available.
    pub fn wait_duration_for(&self, tokens: f64) -> Duration {
        let mut guard = match self.state.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };

        self.refill_locked(&mut guard);

        if guard.tokens >= tokens {
            Duration::ZERO
        } else {
            let deficit = tokens - guard.tokens;
            let seconds = deficit / self.config.refill_rate_per_sec.max(0.001);
            Duration::from_secs_f64(seconds)
        }
    }

    /// Internal refill helper called while lock is held.
    fn refill_locked(&self, state: &mut BucketState) {
        let now = Instant::now();
        let elapsed = now.duration_since(state.last_update).as_secs_f64();
        let added = elapsed * self.config.refill_rate_per_sec;
        state.tokens = (state.tokens + added).min(self.config.capacity);
        state.last_update = now;
    }

    /// Returns current available tokens.
    pub fn available_tokens(&self) -> f64 {
        let mut guard = match self.state.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        self.refill_locked(&mut guard);
        guard.tokens
    }
}
