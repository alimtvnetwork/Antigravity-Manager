//! Dynamic Headroom Score Multiplier.
//! Applies a non-linear sigmoid-like response curve to quota headrooms,
//! heavily favoring accounts with ample headroom (>50%) over fragile, near-exhaustion accounts.

/// Multiplier calculation config.
#[derive(Debug, Clone, Copy)]
pub struct HeadroomCurveConfig {
    pub steepness: f64,
    pub midpoint: f64,
}

impl Default for HeadroomCurveConfig {
    fn default() -> Self {
        Self {
            steepness: 6.0,
            midpoint: 0.50, // 50% quota mark is the inflection point
        }
    }
}

/// Computes non-linear headroom multiplier (0.0 to 1.0) given a ratio in [0.0, 1.0].
pub fn calculate_headroom_multiplier(
    remaining_ratio: f64,
    config: Option<HeadroomCurveConfig>,
) -> f64 {
    let cfg = config.unwrap_or_default();
    let clamped_ratio = remaining_ratio.clamp(0.0, 1.0);

    // Logistic function: 1 / (1 + e^(-k * (x - x0)))
    let exponent = -cfg.steepness * (clamped_ratio - cfg.midpoint);
    let logistic = 1.0 / (1.0 + exponent.exp());

    // Normalize output so f(0.0) ≈ 0.0 and f(1.0) ≈ 1.0
    let f0 = 1.0 / (1.0 + (cfg.steepness * cfg.midpoint).exp());
    let f1 = 1.0 / (1.0 + (-cfg.steepness * (1.0 - cfg.midpoint)).exp());

    let normalized = (logistic - f0) / (f1 - f0);
    normalized.clamp(0.0, 1.0)
}

/// Applies headroom multiplier to an existing raw candidate score.
pub fn apply_headroom_weighting(base_score: f64, remaining_ratio: f64) -> f64 {
    let multiplier = calculate_headroom_multiplier(remaining_ratio, None);
    base_score * multiplier
}
