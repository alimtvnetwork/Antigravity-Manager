//! Quota Velocity Exponential Moving Average Smoother
//! Applies EMA smoothing to quota burn velocity to filter out transient token spikes
//! and provide stable consumption rates for auto-switch scoring.

#[derive(Debug, Clone)]
pub struct VelocitySmoother {
    alpha: f64,
    current_ema: Option<f64>,
}

impl Default for VelocitySmoother {
    fn default() -> Self {
        Self::new(0.25)
    }
}

impl VelocitySmoother {
    pub fn new(alpha: f64) -> Self {
        let clamped_alpha = alpha.clamp(0.01, 1.0);
        Self {
            alpha: clamped_alpha,
            current_ema: None,
        }
    }

    /// Ingests a new instantaneous velocity sample (e.g. tokens/sec or quota%/hour)
    /// and returns the updated smoothed EMA.
    pub fn update(&mut self, sample: f64) -> f64 {
        let new_ema = match self.current_ema {
            Some(prev) => (self.alpha * sample) + ((1.0 - self.alpha) * prev),
            None => sample,
        };
        self.current_ema = Some(new_ema);
        new_ema
    }

    /// Gets current smoothed velocity, or 0.0 if no samples ingested.
    pub fn current_velocity(&self) -> f64 {
        self.current_ema.unwrap_or(0.0)
    }

    /// Resets the smoother to clean state.
    pub fn reset(&mut self) {
        self.current_ema = None;
    }
}
