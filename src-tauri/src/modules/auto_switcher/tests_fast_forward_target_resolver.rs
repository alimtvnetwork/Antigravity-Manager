//! Unit test suite for Fast-Forward Target Account Resolver.
//!
//! Verifies the 4-tier decision ladder, weekly quota floor, clean 4-hour headroom,
//! fleet usage window verification, closest-to-expire ranking, and degraded fallback scoring.

use crate::modules::auto_switcher::fast_forward_target_resolver::*;

#[test]
fn test_weekly_quota_floor_skips_sub_15_percent() {
    let config = FfResolutionConfig::default();

    // Account with 14.9% weekly quota is skipped as zero credits.
    // Account with 15.0% weekly quota is considered eligible.
    let candidates = vec![
        FfCandidateAccount::new("acc-sub-15", "sub15@test.com")
            .with_weekly_quota(14.9)
            .with_rolling_4h_quota(100.0),
        FfCandidateAccount::new("acc-at-15", "at15@test.com")
            .with_weekly_quota(15.0)
            .with_rolling_4h_quota(100.0),
    ];

    let result = resolve_fast_forward_target(&candidates, None, &config);
    match result {
        FfResolutionOutcome::FoundCandidate {
            account_id,
            selection_tier,
            ..
        } => {
            assert_eq!(account_id, "acc-at-15");
            assert_eq!(
                selection_tier,
                FfSelectionTier::PrimaryFullHeadroomClosestExpiry
            );
        }
        _ => panic!("Expected FoundCandidate with acc-at-15, got {:?}", result),
    }

    // Verify isolated sub-15% account produces NoEligibleCandidates
    let sub_15_only = vec![FfCandidateAccount::new("acc-sub-15", "sub15@test.com")
        .with_weekly_quota(14.9)
        .with_rolling_4h_quota(100.0)];
    let outcome = resolve_fast_forward_target(&sub_15_only, None, &config);
    assert!(!outcome.is_found());
    match outcome {
        FfResolutionOutcome::NoEligibleCandidates { reason } => {
            assert!(reason.contains("weekly quota floor"));
        }
        _ => panic!("Expected NoEligibleCandidates for sub-15% account"),
    }
}

#[test]
fn test_sub_100_percent_4h_quota_skips_primary_tier() {
    let config = FfResolutionConfig::default();

    // Account with 95% 4h quota cannot be selected in Primary tier.
    let candidates = vec![FfCandidateAccount::new("acc-95pct", "user95@test.com")
        .with_weekly_quota(80.0)
        .with_rolling_4h_quota(95.0)];

    let result = resolve_fast_forward_target(&candidates, None, &config);
    match result {
        FfResolutionOutcome::FoundCandidate {
            account_id,
            selection_tier,
            score,
            ..
        } => {
            assert_eq!(account_id, "acc-95pct");
            // Must NOT be selected in Primary tier! Must be Degraded Fallback.
            assert_eq!(
                selection_tier,
                FfSelectionTier::FallbackHighestCombinedQuota
            );
            assert!((score - (80.0 + 95.0)).abs() < 1e-6);
        }
        _ => panic!("Expected FallbackHighestCombinedQuota, got {:?}", result),
    }

    // When competing against a 100% 4h account, 95% 4h account is superseded by Primary tier
    let mixed = vec![
        FfCandidateAccount::new("acc-95pct", "user95@test.com")
            .with_weekly_quota(90.0)
            .with_rolling_4h_quota(95.0)
            .with_tier_multiplier(4.0), // High score in fallback
        FfCandidateAccount::new("acc-full-100", "full100@test.com")
            .with_weekly_quota(50.0)
            .with_rolling_4h_quota(100.0)
            .with_tier_multiplier(1.0), // Lower score, but 100% 4h credit
    ];
    let mixed_result = resolve_fast_forward_target(&mixed, None, &config);
    match mixed_result {
        FfResolutionOutcome::FoundCandidate {
            account_id,
            selection_tier,
            ..
        } => {
            assert_eq!(account_id, "acc-full-100");
            assert_eq!(
                selection_tier,
                FfSelectionTier::PrimaryFullHeadroomClosestExpiry
            );
        }
        _ => panic!(
            "Expected acc-full-100 in Primary tier, got {:?}",
            mixed_result
        ),
    }
}

#[test]
fn test_recent_fleet_usage_in_last_4h_skips_primary_tier() {
    // Current epoch: 20000. 4h window = 14400s.
    let now = 20_000u64;
    let config = FfResolutionConfig::new(now);

    // Account used 2 hours ago (within 14400s: 20000 - 7200 = 12800) is skipped in Primary tier.
    // Account used 5 hours ago (beyond 14400s: 20000 - 18000 = 2000) is eligible for Primary tier.
    let candidates = vec![
        FfCandidateAccount::new("acc-used-2h-ago", "used2h@test.com")
            .with_weekly_quota(90.0)
            .with_rolling_4h_quota(100.0)
            .with_last_used_epoch(now - 7_200), // 2h ago
        FfCandidateAccount::new("acc-used-5h-ago", "used5h@test.com")
            .with_weekly_quota(80.0)
            .with_rolling_4h_quota(100.0)
            .with_last_used_epoch(now - 18_000), // 5h ago
    ];

    let result = resolve_fast_forward_target(&candidates, None, &config);
    match result {
        FfResolutionOutcome::FoundCandidate {
            account_id,
            selection_tier,
            ..
        } => {
            assert_eq!(account_id, "acc-used-5h-ago");
            assert_eq!(
                selection_tier,
                FfSelectionTier::PrimaryFullHeadroomClosestExpiry
            );
        }
        _ => panic!("Expected acc-used-5h-ago, got {:?}", result),
    }

    // Account never used before (last_used == 0) is also eligible for Primary tier
    let candidates_never_used = vec![
        FfCandidateAccount::new("acc-used-recent", "recent@test.com")
            .with_weekly_quota(95.0)
            .with_rolling_4h_quota(100.0)
            .with_last_used_epoch(now - 3_600), // 1h ago
        FfCandidateAccount::new("acc-never-used", "never@test.com")
            .with_weekly_quota(70.0)
            .with_rolling_4h_quota(100.0)
            .with_last_used_epoch(0), // Never used
    ];
    let outcome_never_used = resolve_fast_forward_target(&candidates_never_used, None, &config);
    match outcome_never_used {
        FfResolutionOutcome::FoundCandidate {
            account_id,
            selection_tier,
            ..
        } => {
            assert_eq!(account_id, "acc-never-used");
            assert_eq!(
                selection_tier,
                FfSelectionTier::PrimaryFullHeadroomClosestExpiry
            );
        }
        _ => panic!("Expected acc-never-used, got {:?}", outcome_never_used),
    }

    // If only recent-usage account is available, it degrades to Fallback tier
    let only_recent = vec![
        FfCandidateAccount::new("acc-used-recent", "recent@test.com")
            .with_weekly_quota(95.0)
            .with_rolling_4h_quota(100.0)
            .with_last_used_epoch(now - 3_600),
    ];
    let outcome_recent = resolve_fast_forward_target(&only_recent, None, &config);
    match outcome_recent {
        FfResolutionOutcome::FoundCandidate {
            account_id,
            selection_tier,
            ..
        } => {
            assert_eq!(account_id, "acc-used-recent");
            assert_eq!(
                selection_tier,
                FfSelectionTier::FallbackHighestCombinedQuota
            );
        }
        _ => panic!(
            "Expected fallback tier for recent account, got {:?}",
            outcome_recent
        ),
    }
}

#[test]
fn test_active_on_other_instance_excluded() {
    let config = FfResolutionConfig::default();

    // Account flagged is_active_on_other_instance == true is skipped across all tiers
    let candidates = vec![
        FfCandidateAccount::new("acc-active-other", "active@test.com")
            .with_weekly_quota(100.0)
            .with_rolling_4h_quota(100.0)
            .with_tier_multiplier(4.0) // Ultra tier
            .with_active_on_other_instance(true),
        FfCandidateAccount::new("acc-idle", "idle@test.com")
            .with_weekly_quota(20.0)
            .with_rolling_4h_quota(100.0)
            .with_tier_multiplier(1.0) // Free tier
            .with_active_on_other_instance(false),
    ];

    let result = resolve_fast_forward_target(&candidates, None, &config);
    match result {
        FfResolutionOutcome::FoundCandidate { account_id, .. } => {
            assert_eq!(account_id, "acc-idle");
        }
        _ => panic!("Expected acc-idle, got {:?}", result),
    }

    // When only active accounts exist, resolution must return NoEligibleCandidates
    let all_active = vec![
        FfCandidateAccount::new("acc-active-1", "active1@test.com")
            .with_active_on_other_instance(true),
        FfCandidateAccount::new("acc-active-2", "active2@test.com")
            .with_active_on_other_instance(true),
    ];
    let outcome = resolve_fast_forward_target(&all_active, None, &config);
    assert!(!outcome.is_found());
    match outcome {
        FfResolutionOutcome::NoEligibleCandidates { reason } => {
            assert!(reason.contains("active on other instances"));
        }
        _ => panic!("Expected NoEligibleCandidates for all active accounts"),
    }
}

#[test]
fn test_primary_tier_selects_closest_to_expire() {
    let now = 10_000u64;
    let config = FfResolutionConfig::new(now);

    // Multiple 100% 4h accounts: the one with reset_epoch_seconds closest to now is chosen.
    let candidates = vec![
        FfCandidateAccount::new("acc-resets-late", "late@test.com")
            .with_reset_epoch(18_000) // Resets in 8000s
            .with_weekly_quota(95.0)
            .with_rolling_4h_quota(100.0),
        FfCandidateAccount::new("acc-resets-soonest", "soon@test.com")
            .with_reset_epoch(11_500) // Resets in 1500s (closest!)
            .with_weekly_quota(75.0)
            .with_rolling_4h_quota(100.0),
        FfCandidateAccount::new("acc-resets-medium", "med@test.com")
            .with_reset_epoch(14_000) // Resets in 4000s
            .with_weekly_quota(85.0)
            .with_rolling_4h_quota(100.0),
    ];

    let result = resolve_fast_forward_target(&candidates, None, &config);
    match result {
        FfResolutionOutcome::FoundCandidate {
            account_id,
            selection_tier,
            ..
        } => {
            assert_eq!(account_id, "acc-resets-soonest");
            assert_eq!(
                selection_tier,
                FfSelectionTier::PrimaryFullHeadroomClosestExpiry
            );
        }
        _ => panic!("Expected acc-resets-soonest, got {:?}", result),
    }

    // Tie-breaker: Same reset_epoch, higher weekly quota wins
    let tie_reset = vec![
        FfCandidateAccount::new("acc-tie-low-weekly", "low@test.com")
            .with_reset_epoch(12_000)
            .with_weekly_quota(40.0)
            .with_rolling_4h_quota(100.0),
        FfCandidateAccount::new("acc-tie-high-weekly", "high@test.com")
            .with_reset_epoch(12_000)
            .with_weekly_quota(80.0)
            .with_rolling_4h_quota(100.0),
    ];
    let tie_result = resolve_fast_forward_target(&tie_reset, None, &config);
    match tie_result {
        FfResolutionOutcome::FoundCandidate { account_id, .. } => {
            assert_eq!(account_id, "acc-tie-high-weekly");
        }
        _ => panic!("Expected acc-tie-high-weekly, got {:?}", tie_result),
    }

    // Tie-breaker 2: Same reset_epoch and same weekly quota, higher tier_multiplier wins
    let tie_tier = vec![
        FfCandidateAccount::new("acc-pro", "pro@test.com")
            .with_reset_epoch(12_000)
            .with_weekly_quota(80.0)
            .with_rolling_4h_quota(100.0)
            .with_tier_multiplier(2.0),
        FfCandidateAccount::new("acc-ultra", "ultra@test.com")
            .with_reset_epoch(12_000)
            .with_weekly_quota(80.0)
            .with_rolling_4h_quota(100.0)
            .with_tier_multiplier(4.0),
    ];
    let tier_result = resolve_fast_forward_target(&tie_tier, None, &config);
    match tier_result {
        FfResolutionOutcome::FoundCandidate { account_id, .. } => {
            assert_eq!(account_id, "acc-ultra");
        }
        _ => panic!("Expected acc-ultra, got {:?}", tier_result),
    }
}

#[test]
fn test_degraded_fallback_selects_highest_combined_credits() {
    let config = FfResolutionConfig::default();

    // When no 100% 4h account exists, selects highest combined tier_multiplier * (weekly + 4h).
    let candidates = vec![
        // Free tier (multiplier 1.0): score = 1.0 * (85.0 + 70.0) = 155.0
        FfCandidateAccount::new("acc-free", "free@test.com")
            .with_tier_multiplier(1.0)
            .with_weekly_quota(85.0)
            .with_rolling_4h_quota(70.0),
        // Pro tier (multiplier 2.0): score = 2.0 * (40.0 + 50.0) = 180.0
        FfCandidateAccount::new("acc-pro", "pro@test.com")
            .with_tier_multiplier(2.0)
            .with_weekly_quota(40.0)
            .with_rolling_4h_quota(50.0),
        // Ultra tier (multiplier 4.0): score = 4.0 * (25.0 + 35.0) = 240.0
        FfCandidateAccount::new("acc-ultra", "ultra@test.com")
            .with_tier_multiplier(4.0)
            .with_weekly_quota(25.0)
            .with_rolling_4h_quota(35.0),
    ];

    let result = resolve_fast_forward_target(&candidates, None, &config);
    match result {
        FfResolutionOutcome::FoundCandidate {
            account_id,
            selection_tier,
            score,
            ..
        } => {
            assert_eq!(account_id, "acc-ultra");
            assert_eq!(
                selection_tier,
                FfSelectionTier::FallbackHighestCombinedQuota
            );
            assert!((score - 240.0).abs() < 1e-6);
        }
        _ => panic!("Expected acc-ultra, got {:?}", result),
    }
}

#[test]
fn test_configurable_weekly_threshold() {
    // Changing min_weekly_quota_percent to 25.0% correctly skips 20% accounts.
    let standard_config = FfResolutionConfig::default(); // default 15.0%
    let strict_config = FfResolutionConfig::default().with_min_weekly_quota_percent(25.0);

    let candidates = vec![FfCandidateAccount::new("acc-20pct", "user20@test.com")
        .with_weekly_quota(20.0)
        .with_rolling_4h_quota(100.0)];

    // Under default 15.0% floor, 20% is eligible
    let default_res = resolve_fast_forward_target(&candidates, None, &standard_config);
    assert!(default_res.is_found());
    assert_eq!(default_res.candidate_id(), Some("acc-20pct"));

    // Under strict 25.0% floor, 20% is skipped as zero credits
    let strict_res = resolve_fast_forward_target(&candidates, None, &strict_config);
    assert!(!strict_res.is_found());
    match strict_res {
        FfResolutionOutcome::NoEligibleCandidates { reason } => {
            assert!(reason.contains("25.0%"));
        }
        _ => panic!("Expected NoEligibleCandidates under 25.0% floor"),
    }

    // Account at exactly 25.0% is accepted under strict 25.0% floor
    let exact_candidates = vec![FfCandidateAccount::new("acc-25pct", "user25@test.com")
        .with_weekly_quota(25.0)
        .with_rolling_4h_quota(100.0)];
    let exact_res = resolve_fast_forward_target(&exact_candidates, None, &strict_config);
    assert!(exact_res.is_found());
    assert_eq!(exact_res.candidate_id(), Some("acc-25pct"));
}

#[test]
fn test_current_instance_account_excluded() {
    let config = FfResolutionConfig::default();

    // Account matching current_instance_account_id is never selected.
    let candidates = vec![
        FfCandidateAccount::new("acc-current-active", "active@test.com")
            .with_weekly_quota(100.0)
            .with_rolling_4h_quota(100.0)
            .with_tier_multiplier(4.0),
        FfCandidateAccount::new("acc-alternate", "alt@test.com")
            .with_weekly_quota(40.0)
            .with_rolling_4h_quota(100.0)
            .with_tier_multiplier(1.0),
    ];

    let result = resolve_fast_forward_target(&candidates, Some("acc-current-active"), &config);
    match result {
        FfResolutionOutcome::FoundCandidate { account_id, .. } => {
            assert_eq!(account_id, "acc-alternate");
        }
        _ => panic!("Expected acc-alternate, got {:?}", result),
    }

    // If only current instance account is in the candidate pool, result is NoEligibleCandidates
    let only_current = vec![FfCandidateAccount::new("acc-current-active", "active@test.com")];
    let outcome = resolve_fast_forward_target(&only_current, Some("acc-current-active"), &config);
    assert!(!outcome.is_found());
}

#[test]
fn test_disabled_accounts_excluded() {
    let config = FfResolutionConfig::default();
    let candidates = vec![
        FfCandidateAccount::new("acc-disabled", "disabled@test.com")
            .with_disabled(true)
            .with_weekly_quota(100.0)
            .with_rolling_4h_quota(100.0),
        FfCandidateAccount::new("acc-enabled", "enabled@test.com")
            .with_disabled(false)
            .with_weekly_quota(50.0)
            .with_rolling_4h_quota(100.0),
    ];

    let result = resolve_fast_forward_target(&candidates, None, &config);
    assert_eq!(result.candidate_id(), Some("acc-enabled"));
}

#[test]
fn test_empty_candidates_returns_no_eligible() {
    let config = FfResolutionConfig::default();
    let candidates: Vec<FfCandidateAccount> = vec![];
    let result = resolve_fast_forward_target(&candidates, None, &config);
    assert!(!result.is_found());
    assert_eq!(result.candidate_id(), None);
}

#[test]
fn test_rolling_4h_floating_tolerance_boundary() {
    let config = FfResolutionConfig::default();

    // 99.4% is below floating tolerance (99.5%), so it skips Primary tier and falls to Fallback
    let sub_tol = vec![FfCandidateAccount::new("acc-99-4", "test@test.com")
        .with_weekly_quota(80.0)
        .with_rolling_4h_quota(99.4)];
    let res_sub = resolve_fast_forward_target(&sub_tol, None, &config);
    match res_sub {
        FfResolutionOutcome::FoundCandidate { selection_tier, .. } => {
            assert_eq!(
                selection_tier,
                FfSelectionTier::FallbackHighestCombinedQuota
            );
        }
        _ => panic!("Expected fallback tier for 99.4% quota"),
    }

    // 99.5% meets floating tolerance (>= 99.5%), qualifies for Primary tier
    let at_tol = vec![FfCandidateAccount::new("acc-99-5", "test@test.com")
        .with_weekly_quota(80.0)
        .with_rolling_4h_quota(99.5)];
    let res_at = resolve_fast_forward_target(&at_tol, None, &config);
    match res_at {
        FfResolutionOutcome::FoundCandidate { selection_tier, .. } => {
            assert_eq!(
                selection_tier,
                FfSelectionTier::PrimaryFullHeadroomClosestExpiry
            );
        }
        _ => panic!("Expected Primary tier for 99.5% quota"),
    }
}
