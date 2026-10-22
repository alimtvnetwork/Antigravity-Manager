//! Account Cooldown Exponential Decay.
//! Computes gradual penalty decay for accounts in temporary cooldown, transitioning them smoothly back to candidate eligibility.

use serde::{Deserialize, Serialize};

/// Status report indicating cooldown progress and decay factors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CooldownDecayStatus {
    pub is_in_cooldown: bool,
    pub remaining_secs: i64,
    pub penalty_factor: f64,     // 1.0 = full penalty, 0.0 = no penalty
    pub eligibility_factor: f64, // 0.0 = ineligible, 1.0 = fully eligible
}

/// Calculates exponential decay of cooldown penalty as time approaches expiration.
pub fn calculate_cooldown_decay(
    cooldown_until_epoch: Option<i64>,
    total_cooldown_secs: i64,
    now_epoch: i64,
) -> CooldownDecayStatus {
    let until = match cooldown_until_epoch {
        Some(ts) => ts,
        None => {
            return CooldownDecayStatus {
                is_in_cooldown: false,
                remaining_secs: 0,
                penalty_factor: 0.0,
                eligibility_factor: 1.0,
            };
        }
    };

    let remaining_secs = until - now_epoch;
    if remaining_secs <= 0 {
        return CooldownDecayStatus {
            is_in_cooldown: false,
            remaining_secs: 0,
            penalty_factor: 0.0,
            eligibility_factor: 1.0,
        };
    }

    let total = total_cooldown_secs.max(1) as f64;
    let ratio = (remaining_secs as f64 / total).clamp(0.0, 1.0);

    // Quadratic / exponential decay curve: ratio^1.5
    let penalty_factor = (ratio.powf(1.5) * 1000.0).round() / 1000.0;
    let eligibility_factor = ((1.0 - penalty_factor) * 1000.0).round() / 1000.0;

    CooldownDecayStatus {
        is_in_cooldown: true,
        remaining_secs,
        penalty_factor,
        eligibility_factor,
    }
}
