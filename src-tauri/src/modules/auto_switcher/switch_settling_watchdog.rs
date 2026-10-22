//! Profile Switch Settling Watchdog.
//! Enforces the mandatory 10-second settling period after IDE account switching,
//! guaranteeing that the language server and extension host have fully initialized before prompt dispatch.

use std::time::{Duration, Instant};

/// Mandatory default settling period across all instance switches.
pub const DEFAULT_SWITCH_SETTLING_DURATION: Duration = Duration::from_secs(10);

/// Active state tracker for an instance's post-switch settling phase.
#[derive(Debug)]
pub struct SwitchSettlingWatchdog {
    instance_id: String,
    switched_at: Instant,
    settling_duration: Duration,
}

impl SwitchSettlingWatchdog {
    /// Starts a settling period timer for an instance.
    pub fn start(instance_id: &str, custom_duration: Option<Duration>) -> Self {
        Self {
            instance_id: instance_id.to_string(),
            switched_at: Instant::now(),
            settling_duration: custom_duration.unwrap_or(DEFAULT_SWITCH_SETTLING_DURATION),
        }
    }

    /// Checks whether the settling period has successfully concluded.
    pub fn is_settled(&self) -> bool {
        self.switched_at.elapsed() >= self.settling_duration
    }

    /// Returns remaining settling duration.
    pub fn remaining_delay(&self) -> Duration {
        let elapsed = self.switched_at.elapsed();
        if elapsed >= self.settling_duration {
            Duration::ZERO
        } else {
            self.settling_duration - elapsed
        }
    }

    /// Returns percentage (0 to 100) of settling completed.
    pub fn progress_percent(&self) -> f64 {
        let elapsed_secs = self.switched_at.elapsed().as_secs_f64();
        let total_secs = self.settling_duration.as_secs_f64();
        (elapsed_secs / total_secs * 100.0).clamp(0.0, 100.0)
    }

    /// Associated instance identifier.
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }
}
