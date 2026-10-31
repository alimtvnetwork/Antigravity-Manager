//! Cooldown recovery curve calculator.
//! Computes smooth non-linear sigmoid recovery multipliers for accounts
//! emerging from rate-limit cooldown to prevent sudden rotation shockwaves.

use serde::{Deserialize, Serialize};

/// Configuration parameters for the sigmoid recovery curve.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryCurveConfig {
    /// Steepness parameter (k) of the sigmoid transition.
    pub steepness: f64,
    /// Offset threshold (midpoint) where recovery reaches 50%.
    pub midpoint_ratio: f64,
}

impl Default for RecoveryCurveConfig {
    fn default() -> Self {
        Self {
            steepness: 8.0,
            midpoint_ratio: 0.5,
        }
    }
}

pub struct CooldownRecoveryCurve;

impl CooldownRecoveryCurve {
    /// Computes a smooth multiplier in range [0.0, 1.0] based on progress through cooldown.
    /// - At progress = 0.0 (just tripped), returns ~0.0
    /// - At progress = 0.5 (midpoint), returns ~0.5
    /// - At progress = 1.0 (cooldown elapsed), returns ~1.0
    pub fn compute_multiplier(
        elapsed_secs: f64,
        total_cooldown_secs: f64,
        config: Option<&RecoveryCurveConfig>,
    ) -> f64 {
        if total_cooldown_secs <= 0.0 || elapsed_secs >= total_cooldown_secs {
            return 1.0;
        }
        if elapsed_secs <= 0.0 {
            return 0.0;
        }

        let default_cfg = RecoveryCurveConfig::default();
        let cfg = config.unwrap_or(&default_cfg);

        let progress = (elapsed_secs / total_cooldown_secs).clamp(0.0, 1.0);
        let x = progress - cfg.midpoint_ratio;
        let sigmoid = 1.0 / (1.0 + (-cfg.steepness * x).exp());

        // Normalize so f(0) = 0 and f(1) = 1
        let min_val = 1.0 / (1.0 + (cfg.steepness * cfg.midpoint_ratio).exp());
        let max_val = 1.0 / (1.0 + (-cfg.steepness * (1.0 - cfg.midpoint_ratio)).exp());
        let normalized = (sigmoid - min_val) / (max_val - min_val).max(0.0001);

        normalized.clamp(0.0, 1.0)
    }
}
