//! Unit tests for Auto-Switcher Candidate Dynamic Scorer (`Imp-619`)

use super::candidate_dynamic_scorer::{
    rank_candidates, score_candidate_dynamically, select_best_candidate, CandidateScoringFactors,
    ScoredCandidate, ScoringViolation,
};

#[test]
fn test_inactive_account_disqualified() {
    let mut factors = CandidateScoringFactors::new("acc-1", "user1@example.com");
    factors.is_active = false;

    let res = score_candidate_dynamically(&factors);
    assert!(!res.is_eligible);
    assert_eq!(res.final_score, 0.0);
    assert!(res
        .exclusion_reason
        .as_deref()
        .unwrap()
        .contains("inactive or disabled"));
    assert!(res
        .violations
        .contains(&ScoringViolation::AccountInactiveOrDisabled));
    assert!(res.has_violations());
    assert!(!res.is_selectable());
}

#[test]
fn test_in_use_by_other_instance_disqualified() {
    let mut factors = CandidateScoringFactors::new("acc-2", "user2@example.com");
    factors.is_in_use_by_other_instance = true;

    let res = score_candidate_dynamically(&factors);
    assert!(!res.is_eligible);
    assert_eq!(res.final_score, 0.0);
    assert!(res
        .exclusion_reason
        .as_deref()
        .unwrap()
        .contains("in use by sibling instance"));
    assert!(res
        .violations
        .contains(&ScoringViolation::InUseByOtherInstance));
    assert!(!res.is_selectable());
}

#[test]
fn test_cooldown_active_disqualified() {
    let mut factors = CandidateScoringFactors::new("acc-3", "user3@example.com");
    factors.cooldown_remaining_secs = 45;

    let res = score_candidate_dynamically(&factors);
    assert!(!res.is_eligible);
    assert_eq!(res.final_score, 0.0);
    assert!(res
        .exclusion_reason
        .as_deref()
        .unwrap()
        .contains("in cooldown (45s remaining)"));
    assert!(res.violations.contains(&ScoringViolation::ActiveCooldown));
    assert!(!res.is_selectable());
}

#[test]
fn test_exhausted_quota_disqualified() {
    let mut factors = CandidateScoringFactors::new("acc-4", "user4@example.com");
    factors.remaining_quota_percent = 0.0;

    let res = score_candidate_dynamically(&factors);
    assert!(!res.is_eligible);
    assert_eq!(res.final_score, 0.0);
    assert!(res
        .exclusion_reason
        .as_deref()
        .unwrap()
        .contains("Quota completely exhausted"));
    assert!(res.violations.contains(&ScoringViolation::QuotaExhausted));
    assert!(!res.is_selectable());
}

#[test]
fn test_healthy_candidate_full_score() {
    let factors = CandidateScoringFactors::new("acc-5", "user5@example.com");
    // Default: 100% quota, 0 throttles, no velocity penalty
    let res = score_candidate_dynamically(&factors);

    assert!(res.is_eligible);
    assert_eq!(res.base_score, 1000.0);
    assert_eq!(res.velocity_penalty, 0.0);
    assert_eq!(res.throttle_penalty, 0.0);
    assert_eq!(res.final_score, 1000.0);
    assert!(res.exclusion_reason.is_none());
    assert!(res.violations.is_empty());
    assert!(!res.has_violations());
    assert!(res.is_selectable());
}

#[test]
fn test_velocity_penalties() {
    // Case A: <= 120s remaining -> 400 pt penalty + violation
    let mut f_crit = CandidateScoringFactors::new("acc-crit", "crit@example.com");
    f_crit.estimated_seconds_remaining = Some(90);
    let res_crit = score_candidate_dynamically(&f_crit);
    assert_eq!(res_crit.velocity_penalty, 400.0);
    assert_eq!(res_crit.final_score, 600.0);
    assert!(res_crit
        .violations
        .contains(&ScoringViolation::CriticalVelocityExhaustion));

    // Case B: <= 300s remaining -> 200 pt penalty
    let mut f_warn = CandidateScoringFactors::new("acc-warn", "warn@example.com");
    f_warn.estimated_seconds_remaining = Some(250);
    let res_warn = score_candidate_dynamically(&f_warn);
    assert_eq!(res_warn.velocity_penalty, 200.0);
    assert_eq!(res_warn.final_score, 800.0);

    // Case C: <= 600s remaining -> 50 pt penalty
    let mut f_elev = CandidateScoringFactors::new("acc-elev", "elev@example.com");
    f_elev.estimated_seconds_remaining = Some(500);
    let res_elev = score_candidate_dynamically(&f_elev);
    assert_eq!(res_elev.velocity_penalty, 50.0);
    assert_eq!(res_elev.final_score, 950.0);
}

#[test]
fn test_throttle_penalties() {
    let mut f = CandidateScoringFactors::new("acc-throttled", "throttled@example.com");
    f.consecutive_throttles = 2; // 2 * 150 = 300 pt penalty

    let res = score_candidate_dynamically(&f);
    assert_eq!(res.throttle_penalty, 300.0);
    assert_eq!(res.final_score, 700.0);

    // Excessive throttles (>= 3)
    let mut f_excess =
        CandidateScoringFactors::new("acc-heavy-throttled", "throttled2@example.com");
    f_excess.consecutive_throttles = 3;
    let res_excess = score_candidate_dynamically(&f_excess);
    assert!(res_excess
        .violations
        .contains(&ScoringViolation::ExcessiveThrottling));
}

#[test]
fn test_ranking_candidates_order() {
    let mut c1 = CandidateScoringFactors::new("acc-low", "low@example.com");
    c1.remaining_quota_percent = 40.0; // score: 400.0

    let mut c2 = CandidateScoringFactors::new("acc-high", "high@example.com");
    c2.remaining_quota_percent = 95.0; // score: 950.0

    let mut c3 = CandidateScoringFactors::new("acc-in-use", "inuse@example.com");
    c3.is_in_use_by_other_instance = true; // ineligible, score: 0.0

    let mut c4 = CandidateScoringFactors::new("acc-medium", "medium@example.com");
    c4.remaining_quota_percent = 80.0; // score: 800.0

    let ranked = rank_candidates(&[c1, c2, c3, c4]);

    assert_eq!(ranked.len(), 4);
    assert_eq!(ranked[0].account_id, "acc-high");
    assert_eq!(ranked[1].account_id, "acc-medium");
    assert_eq!(ranked[2].account_id, "acc-low");
    assert_eq!(ranked[3].account_id, "acc-in-use");
    assert!(!ranked[3].is_eligible);
}

#[test]
fn test_select_best_candidate() {
    let mut c_cool = CandidateScoringFactors::new("acc-cool", "cool@example.com");
    c_cool.cooldown_remaining_secs = 60;

    let c_ready = CandidateScoringFactors::new("acc-ready", "ready@example.com");

    let best = select_best_candidate(&[c_cool, c_ready]);
    assert!(best.is_some());
    assert_eq!(best.unwrap().account_id, "acc-ready");

    let empty = select_best_candidate(&[]);
    assert!(empty.is_none());
}

#[test]
fn test_scoring_violation_disqualifying_and_display() {
    assert!(ScoringViolation::AccountInactiveOrDisabled.is_disqualifying());
    assert!(ScoringViolation::InUseByOtherInstance.is_disqualifying());
    assert!(ScoringViolation::ActiveCooldown.is_disqualifying());
    assert!(ScoringViolation::QuotaExhausted.is_disqualifying());
    assert!(!ScoringViolation::CriticalVelocityExhaustion.is_disqualifying());
    assert!(!ScoringViolation::ExcessiveThrottling.is_disqualifying());

    let desc = format!("{}", ScoringViolation::CriticalVelocityExhaustion);
    assert!(desc.contains("under 120 seconds"));
}

#[test]
fn test_candidate_scoring_factors_builder() {
    let f = CandidateScoringFactors::new("acc-b", "builder@example.com")
        .with_remaining_quota_percent(75.0)
        .with_burn_rate(15.0)
        .with_estimated_seconds_remaining(Some(400))
        .with_consecutive_throttles(1)
        .with_cooldown_remaining(0)
        .with_active(true)
        .with_in_use_by_other_instance(false);

    assert_eq!(f.remaining_quota_percent, 75.0);
    assert_eq!(f.burn_rate_tokens_per_sec, 15.0);
    assert_eq!(f.estimated_seconds_remaining, Some(400));
    assert_eq!(f.consecutive_throttles, 1);
}
