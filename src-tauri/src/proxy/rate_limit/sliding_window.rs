//! Sliding Window Token & Request Rate Limiter.
//! Calculates per-minute and per-hour request and token volume sliding windows, enforcing adaptive throttling.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Single observation of request timestamp and tokens consumed.
#[derive(Debug, Clone, Copy)]
pub struct UsageEntry {
    pub timestamp: Instant,
    pub tokens: u64,
}

/// Status report indicating sliding window volumes and limit status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlidingWindowStatus {
    pub is_allowed: bool,
    pub tokens_past_minute: u64,
    pub tokens_past_hour: u64,
    pub requests_past_minute: usize,
    pub requests_past_hour: usize,
}

/// Sliding window rate limiter tracking past 1 hour of traffic.
pub struct SlidingWindowLimiter {
    entries: Mutex<VecDeque<UsageEntry>>,
    max_tokens_per_minute: u64,
    max_tokens_per_hour: u64,
}

impl SlidingWindowLimiter {
    pub fn new(max_tokens_per_minute: u64, max_tokens_per_hour: u64) -> Self {
        Self {
            entries: Mutex::new(VecDeque::new()),
            max_tokens_per_minute,
            max_tokens_per_hour,
        }
    }

    /// Records token and request usage, evicting observations older than 1 hour.
    pub fn record(&self, tokens: u64) {
        let now = Instant::now();
        let one_hour = Duration::from_secs(3600);

        if let Ok(mut lock) = self.entries.lock() {
            lock.push_back(UsageEntry {
                timestamp: now,
                tokens,
            });

            // Evict old entries
            while let Some(front) = lock.front() {
                if now.duration_since(front.timestamp) > one_hour {
                    lock.pop_front();
                } else {
                    break;
                }
            }
        }
    }

    /// Checks if current sliding window volumes are within limits.
    pub fn check_limits(&self) -> SlidingWindowStatus {
        let now = Instant::now();
        let one_minute = Duration::from_secs(60);
        let one_hour = Duration::from_secs(3600);

        let mut tokens_minute = 0u64;
        let mut tokens_hour = 0u64;
        let mut reqs_minute = 0usize;
        let mut reqs_hour = 0usize;

        if let Ok(lock) = self.entries.lock() {
            for entry in lock.iter() {
                let age = now.duration_since(entry.timestamp);
                if age <= one_hour {
                    tokens_hour = tokens_hour.saturating_add(entry.tokens);
                    reqs_hour += 1;
                }
                if age <= one_minute {
                    tokens_minute = tokens_minute.saturating_add(entry.tokens);
                    reqs_minute += 1;
                }
            }
        }

        let is_allowed = (self.max_tokens_per_minute == 0
            || tokens_minute <= self.max_tokens_per_minute)
            && (self.max_tokens_per_hour == 0 || tokens_hour <= self.max_tokens_per_hour);

        SlidingWindowStatus {
            is_allowed,
            tokens_past_minute: tokens_minute,
            tokens_past_hour: tokens_hour,
            requests_past_minute: reqs_minute,
            requests_past_hour: reqs_hour,
        }
    }
}
