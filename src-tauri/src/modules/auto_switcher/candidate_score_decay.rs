// Idle Candidate Score Time-Decay Curve (Imp-490)
// Modulates candidate profile scores based on idle duration to ensure balanced
// utilization and prevent prolonged starvation of secondary accounts.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecayParameters {
    /// Half-life in seconds: duration after which score decay reduces bonus by 50%.
    pub half_life_secs: f64,
    /// Maximum boost factor for long-idle accounts (e.g. 1.25 = 25% boost).
    pub max_boost_factor: f64,
    /// Minimum dampening factor for recently used accounts (e.g. 0.85).
    pub min_dampen_factor: f64,
}

impl Default for DecayParameters {
    fn default() -> Self {
        Self {
            half_life_secs: 1800.0, // 30 minutes
            max_boost_factor: 1.20,
            min_dampen_factor: 0.90,
        }
    }
}

pub struct CandidateScoreDecay {
    params: DecayParameters,
}

impl Default for CandidateScoreDecay {
    fn default() -> Self {
        Self::new(DecayParameters::default())
    }
}

impl CandidateScoreDecay {
    pub fn new(params: DecayParameters) -> Self {
        Self { params }
    }

    /// Calculates a decayed or boosted candidate score based on seconds since last selection.
    pub fn calculate_adjusted_score(&self, base_score: f64, idle_secs: f64) -> f64 {
        if base_score <= 0.0 {
            return 0.0;
        }

        // Exponential recovery curve towards max boost:
        // factor = min_dampen + (max_boost - min_dampen) * (1 - 2^(-idle / half_life))
        let decay_exponent = -idle_secs / self.params.half_life_secs.max(1.0);
        let recovery_ratio = 1.0 - 2.0_f64.powf(decay_exponent);
        let factor = self.params.min_dampen_factor
            + (self.params.max_boost_factor - self.params.min_dampen_factor)
                * recovery_ratio.clamp(0.0, 1.0);

        (base_score * factor).clamp(0.0, 1000.0)
    }
}
