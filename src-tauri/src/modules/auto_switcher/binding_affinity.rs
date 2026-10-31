//! Multi-Instance Binding Affinity Scorer.
//! Calculates sticky affinity bonus scores for accounts currently or recently bound to an instance.

/// Default affinity bonus applied to the currently bound account: 15.0 score points.
pub const DEFAULT_AFFINITY_BONUS_SCORE: f64 = 15.0;

/// Returns affinity score bonus if candidate matches the currently active account on this instance.
pub fn calculate_binding_affinity_bonus(
    candidate_account_id: &str,
    current_instance_account_id: Option<&str>,
    affinity_bonus: Option<f64>,
) -> f64 {
    match current_instance_account_id {
        Some(current_id) if current_id == candidate_account_id => {
            affinity_bonus.unwrap_or(DEFAULT_AFFINITY_BONUS_SCORE)
        }
        _ => 0.0,
    }
}
