//! Auto-Switcher Quota Reset Epoch Drift Compensator (Imp-551)
//! Tracks clock skew between local system time and upstream quota reset epochs,
//! applying a sliding safety buffer to prevent premature account switches.

use serde::{Deserialize, Serialize};

pub const DEFAULT_SAFETY_BUFFER_SEC: i64 = 30;
pub const MAX_PERMISSIBLE_DRIFT_SEC: i64 = 300;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochDriftCompensator {
    pub safety_buffer_sec: i64,
    pub observed_drift_sec: i64,
    pub sample_count: u32,
}

impl Default for EpochDriftCompensator {
    fn default() -> Self {
        Self::new(None)
    }
}

impl EpochDriftCompensator {
    pub fn new(safety_buffer: Option<i64>) -> Self {
        Self {
            safety_buffer_sec: safety_buffer.unwrap_or(DEFAULT_SAFETY_BUFFER_SEC),
            observed_drift_sec: 0,
            sample_count: 0,
        }
    }

    /// Records an upstream server timestamp and updates the estimated drift.
    pub fn record_server_timestamp(&mut self, server_time_sec: i64, local_time_sec: i64) {
        let raw_drift = server_time_sec.saturating_sub(local_time_sec);
        // Clamp extreme outliers to avoid NTP sync anomalies
        let clamped_drift = raw_drift.clamp(-MAX_PERMISSIBLE_DRIFT_SEC, MAX_PERMISSIBLE_DRIFT_SEC);

        if self.sample_count == 0 {
            self.observed_drift_sec = clamped_drift;
        } else {
            // Exponential smoothing (80% previous, 20% new)
            self.observed_drift_sec = ((self.observed_drift_sec * 4) + clamped_drift) / 5;
        }
        self.sample_count = self.sample_count.saturating_add(1);
    }

    /// Calculates the safe, compensated reset epoch timestamp.
    /// Incorporates the safety buffer and compensates for server clock skew.
    pub fn effective_reset_epoch(&self, reported_reset_epoch: i64) -> i64 {
        // If server clock is ahead (positive drift), reset occurs earlier locally
        // Safety buffer adds an intentional delay to guarantee upstream has completed reset
        reported_reset_epoch
            .saturating_add(self.safety_buffer_sec)
            .saturating_sub(self.observed_drift_sec)
    }

    /// Evaluates if the compensated reset epoch has elapsed.
    pub fn has_quota_reset_elapsed(&self, reported_reset_epoch: i64, now_sec: i64) -> bool {
        now_sec >= self.effective_reset_epoch(reported_reset_epoch)
    }

    /// Resets drift samples to zero.
    pub fn reset(&mut self) {
        self.observed_drift_sec = 0;
        self.sample_count = 0;
    }
}
