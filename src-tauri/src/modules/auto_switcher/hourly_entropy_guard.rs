//! Hourly Switch Entropy & Oscillation Guard
//! Calculates normalized Shannon entropy over recent account switch targets
//! to detect and halt pathological rapid switching loops and bounce storms.

use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct SwitchRecord {
    account_id: String,
    timestamp: Instant,
}

#[derive(Debug, Clone)]
pub struct HourlyEntropyGuard {
    history: Vec<SwitchRecord>,
    window_duration: Duration,
    oscillation_threshold: f64,
}

impl Default for HourlyEntropyGuard {
    fn default() -> Self {
        Self::new(Duration::from_secs(3600), 0.35)
    }
}

impl HourlyEntropyGuard {
    pub fn new(window_duration: Duration, oscillation_threshold: f64) -> Self {
        Self {
            history: Vec::new(),
            window_duration,
            oscillation_threshold,
        }
    }

    /// Records an account switch event.
    pub fn record_switch(&mut self, account_id: impl Into<String>) {
        self.sweep_expired();
        self.history.push(SwitchRecord {
            account_id: account_id.into(),
            timestamp: Instant::now(),
        });
    }

    /// Cleans up events older than the rolling window.
    fn sweep_expired(&mut self) {
        let cutoff = Instant::now().checked_sub(self.window_duration);
        if let Some(limit) = cutoff {
            self.history.retain(|r| r.timestamp >= limit);
        }
    }

    /// Calculates normalized Shannon entropy [0.0, 1.0].
    /// A low entropy with high switch counts (>4) indicates pathological oscillation between 2 accounts.
    pub fn calculate_entropy(&mut self) -> f64 {
        self.sweep_expired();
        let total = self.history.len();
        if total <= 1 {
            return 1.0;
        }

        let mut counts: HashMap<&str, usize> = HashMap::new();
        for record in &self.history {
            *counts.entry(&record.account_id).or_insert(0) += 1;
        }

        let distinct_count = counts.len();
        if distinct_count <= 1 {
            return 0.0;
        }

        let total_f = total as f64;
        let mut entropy = 0.0;
        for &count in counts.values() {
            let p = count as f64 / total_f;
            if p > 0.0 {
                entropy -= p * p.log2();
            }
        }

        let max_entropy = (distinct_count as f64).log2();
        if max_entropy > 0.0 {
            entropy / max_entropy
        } else {
            1.0
        }
    }

    /// Returns true if pathological oscillation is detected (frequent switches with low entropy).
    pub fn is_oscillating(&mut self) -> bool {
        self.sweep_expired();
        if self.history.len() < 5 {
            return false;
        }
        let entropy = self.calculate_entropy();
        entropy < self.oscillation_threshold
    }
}
