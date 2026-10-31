//! Token Bucket Rate Limiter.
//! Thread-safe token bucket algorithm with configurable refill rate and burst capacity.

use std::sync::Mutex;
use std::time::Instant;

/// Token bucket configuration.
#[derive(Debug, Clone, Copy)]
pub struct TokenBucketConfig {
    pub capacity: f64,
    pub refill_rate_per_sec: f64,
}

impl Default for TokenBucketConfig {
    fn default() -> Self {
        Self {
            capacity: 100.0,
            refill_rate_per_sec: 20.0,
        }
    }
}

/// Thread-safe token bucket rate limiter.
pub struct TokenBucket {
    config: TokenBucketConfig,
    state: Mutex<TokenBucketState>,
}

struct TokenBucketState {
    tokens: f64,
    last_refill: Instant,
}

impl TokenBucket {
    pub fn new(config: TokenBucketConfig) -> Self {
        let initial_tokens = config.capacity;
        Self {
            config,
            state: Mutex::new(TokenBucketState {
                tokens: initial_tokens,
                last_refill: Instant::now(),
            }),
        }
    }

    /// Attempts to consume requested tokens. Returns true if allowed, false if insufficient tokens.
    pub fn try_consume(&self, count: f64) -> bool {
        if let Ok(mut state) = self.state.lock() {
            let now = Instant::now();
            let elapsed = now.duration_since(state.last_refill).as_secs_f64();
            state.last_refill = now;

            // Refill tokens up to capacity
            state.tokens = (state.tokens + elapsed * self.config.refill_rate_per_sec)
                .min(self.config.capacity);

            if state.tokens >= count {
                state.tokens -= count;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Returns current estimated token count available.
    pub fn available_tokens(&self) -> f64 {
        if let Ok(state) = self.state.lock() {
            let elapsed = state.last_refill.elapsed().as_secs_f64();
            (state.tokens + elapsed * self.config.refill_rate_per_sec).min(self.config.capacity)
        } else {
            0.0
        }
    }
}
