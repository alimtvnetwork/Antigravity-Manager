//! Account Quota History Ring Buffer.
//! Tracks recent rolling quota samples per account to detect accelerated burn velocities.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

pub const DEFAULT_RING_CAPACITY: usize = 20;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaSample {
    pub timestamp: i64,
    pub quota_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaRingBuffer {
    pub account_id: String,
    pub capacity: usize,
    pub samples: VecDeque<QuotaSample>,
}

impl QuotaRingBuffer {
    pub fn new(account_id: &str, capacity: Option<usize>) -> Self {
        let cap = capacity.unwrap_or(DEFAULT_RING_CAPACITY).clamp(5, 100);
        Self {
            account_id: account_id.to_string(),
            capacity: cap,
            samples: VecDeque::with_capacity(cap),
        }
    }

    /// Records a new quota sample into the circular buffer.
    pub fn push_sample(&mut self, timestamp: i64, quota_percent: f64) {
        if self.samples.len() >= self.capacity {
            self.samples.pop_front();
        }
        self.samples.push_back(QuotaSample {
            timestamp,
            quota_percent: quota_percent.clamp(0.0, 100.0),
        });
    }

    /// Computes consumption rate in percentage points per hour based on stored samples.
    pub fn compute_burn_velocity_per_hour(&self) -> f64 {
        if self.samples.len() < 2 {
            return 0.0;
        }

        let oldest = self.samples.front().unwrap();
        let newest = self.samples.back().unwrap();

        let time_delta_sec = newest.timestamp - oldest.timestamp;
        if time_delta_sec <= 0 {
            return 0.0;
        }

        let quota_delta = oldest.quota_percent - newest.quota_percent; // Positive if decreasing
        if quota_delta <= 0.0 {
            return 0.0;
        }

        let hours = time_delta_sec as f64 / 3600.0;
        (quota_delta / hours).max(0.0)
    }
}
