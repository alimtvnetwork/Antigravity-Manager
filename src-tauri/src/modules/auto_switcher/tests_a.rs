#![cfg(test)]

use super::*;
use crate::models::config::AutoProfileSwitcherConfig;
use crate::models::token::TokenData;

use super::*;

pub(crate) fn make_test_account(
    id: &str,
    email: &str,
    model: &str,
    pct: i32,
    reset_time: &str,
) -> Account {
    let token = TokenData::new(
        "access_token".to_string(),
        "refresh_token".to_string(),
        3600,
        Some(email.to_string()),
        None,
        None,
        false,
        None,
    );
    let mut acc = Account::new(id.to_string(), email.to_string(), token);
    let model_quota = crate::models::quota::ModelQuota {
        name: model.to_string(),
        percentage: pct,
        reset_time: reset_time.to_string(),
        display_name: None,
        supports_images: None,
        supports_thinking: None,
        thinking_budget: None,
        recommended: None,
        max_tokens: None,
        max_output_tokens: None,
        supported_mime_types: None,
    };
    acc.quota = Some(crate::models::quota::QuotaData {
        models: vec![model_quota],
        last_updated: chrono::Utc::now().timestamp(),
        subscription_tier: Some("pro".to_string()),
        subscription_tier_fetched_at: None,
        is_forbidden: false,
        forbidden_reason: None,
        model_forwarding_rules: std::collections::HashMap::new(),
        quota_groups: None,
    });
    acc
}

#[test]
pub(crate) fn test_calculate_next_interval_seconds_ladder() {
    let cfg = AutoProfileSwitcherConfig::default();
    assert_eq!(cfg.check_interval_seconds, 300);
    assert_eq!(cfg.caution_interval_seconds, 60);
    assert_eq!(cfg.critical_interval_seconds, 40);
    assert_eq!(cfg.low_quota_threshold_percent, 15.0);
    assert_eq!(cfg.critical_threshold_percent, 15.0);

    assert_eq!(calculate_next_interval_seconds(None, &cfg), 120);
    assert_eq!(calculate_next_interval_seconds(Some(85.0), &cfg), 120);
    assert_eq!(calculate_next_interval_seconds(Some(25.0), &cfg), 120);
    assert_eq!(calculate_next_interval_seconds(Some(19.0), &cfg), 120);
    assert_eq!(calculate_next_interval_seconds(Some(17.9), &cfg), 60);
    assert_eq!(calculate_next_interval_seconds(Some(16.0), &cfg), 60);
    assert_eq!(calculate_next_interval_seconds(Some(15.0), &cfg), 40);
    assert_eq!(calculate_next_interval_seconds(Some(5.0), &cfg), 40);
    assert_eq!(calculate_next_interval_seconds(Some(0.0), &cfg), 40);
    assert_eq!(google_quota_interval_seconds(600), 600);
    assert_eq!(google_quota_interval_seconds(40), 40);
}

#[test]
pub(crate) fn test_parse_reset_time_to_unix() {
    let valid_rfc3339 = "2026-09-22T14:30:00Z";
    let parsed = parse_reset_time_to_unix(valid_rfc3339);
    assert!(parsed.is_some());
    assert_eq!(parsed.unwrap(), 1790087400);

    let empty = "";
    assert_eq!(parse_reset_time_to_unix(empty), None);

    let invalid = "not-a-date";
    assert_eq!(parse_reset_time_to_unix(invalid), None);
}

#[test]
pub(crate) fn test_evaluate_account_period_status_before_finish() {
    let now_sec = 1790080000;
    let future_time = "2026-09-22T14:30:00Z"; // 1790087400, +7400s
    let acc = make_test_account("acc-1", "test@domain.com", "gemini-pro", 8, future_time);

    let status = evaluate_account_period_status(&acc, "gemini-pro", 15.0, now_sec);
    assert!(status.is_some());
    let stat = status.unwrap();

    assert_eq!(stat.quota_percent, 8.0);
    assert!(stat.is_depleted_before_finish);
    let period_finished = stat.is_period_finished;
    assert!(!period_finished);
    assert_eq!(stat.seconds_until_reset, Some(7400));
}

#[test]
pub(crate) fn test_evaluate_account_period_status_after_finish() {
    let now_sec = 1790090000;
    let past_time = "2026-09-22T14:30:00Z"; // 1790087400, -2600s
    let acc = make_test_account("acc-2", "test2@domain.com", "gemini-pro", 5, past_time);

    let status = evaluate_account_period_status(&acc, "gemini-pro", 15.0, now_sec);
    assert!(status.is_some());
    let stat = status.unwrap();

    assert_eq!(stat.quota_percent, 5.0);
    assert!(stat.is_period_finished);
    let depleted_before_finish = stat.is_depleted_before_finish;
    assert!(!depleted_before_finish);
    assert_eq!(stat.seconds_until_reset, Some(-2600));
}

#[test]
pub(crate) fn test_score_candidate_account_reset_time_priority() {
    let now_sec = 1790090000;
    let past_time = "2026-09-22T14:30:00Z"; // Period finished (resets to 100%)
    let future_time = "2026-09-22T20:00:00Z"; // Un-refilled 10% (< 100% -> 0.0)

    let mut acc_a = make_test_account("acc-a", "a@domain.com", "gemini-pro", 10, past_time);
    acc_a.last_used = now_sec - 3600;

    let mut acc_b = make_test_account("acc-b", "b@domain.com", "gemini-pro", 10, future_time);
    acc_b.last_used = now_sec - 3600;

    let score_a = score_candidate_account(&acc_a, "gemini-pro", now_sec);
    let score_b = score_candidate_account(&acc_b, "gemini-pro", now_sec);

    // Account A period finished resets to 100% capacity:
    // base_score = (3.0 * 168.0 * 100.0) / 100.0 = 504.0
    // Account B has < 100% quota and period not finished -> strictly 0.0 in primary phase
    assert_eq!(score_a, 504.0);
    assert_eq!(score_b, 0.0);
    assert!(score_a > score_b);
}

#[test]
pub(crate) fn test_score_candidate_account_weekly_quota_groups_bottleneck() {
    let now_sec = 1790090000;
    let future_time = "2026-09-30T14:30:00Z";

    // Candidate A: Pro tier, 100% weekly Gemini, 100% weekly Claude
    let mut acc_a = make_test_account(
        "synth_acc_a",
        "synth_a@test.local",
        "gemini-pro",
        100,
        future_time,
    );
    if let Some(ref mut q) = acc_a.quota {
        q.quota_groups = Some(vec![
            crate::models::quota::QuotaGroup {
                display_name: "Gemini Models".to_string(),
                description: None,
                buckets: vec![crate::models::quota::QuotaBucket {
                    bucket_id: "gemini-weekly".to_string(),
                    window: "weekly".to_string(),
                    remaining_fraction: 1.0,
                    reset_time: future_time.to_string(),
                    observed_at: None,
                    cycle_tokens: None,
                    display_name: None,
                    description: None,
                }],
            },
            crate::models::quota::QuotaGroup {
                display_name: "Claude and GPT models".to_string(),
                description: None,
                buckets: vec![crate::models::quota::QuotaBucket {
                    bucket_id: "claude-weekly".to_string(),
                    window: "weekly".to_string(),
                    remaining_fraction: 1.0,
                    reset_time: future_time.to_string(),
                    observed_at: None,
                    cycle_tokens: None,
                    display_name: None,
                    description: None,
                }],
            },
        ]);
    }

    // Candidate B: Pro tier, 21% weekly Gemini, 100% weekly Claude
    let mut acc_b = make_test_account(
        "synth_acc_b",
        "synth_b@test.local",
        "gemini-pro",
        100,
        future_time,
    );
    if let Some(ref mut q) = acc_b.quota {
        q.quota_groups = Some(vec![
            crate::models::quota::QuotaGroup {
                display_name: "Gemini Models".to_string(),
                description: None,
                buckets: vec![crate::models::quota::QuotaBucket {
                    bucket_id: "gemini-weekly".to_string(),
                    window: "weekly".to_string(),
                    remaining_fraction: 0.21,
                    reset_time: future_time.to_string(),
                    observed_at: None,
                    cycle_tokens: None,
                    display_name: None,
                    description: None,
                }],
            },
            crate::models::quota::QuotaGroup {
                display_name: "Claude and GPT models".to_string(),
                description: None,
                buckets: vec![crate::models::quota::QuotaBucket {
                    bucket_id: "claude-weekly".to_string(),
                    window: "weekly".to_string(),
                    remaining_fraction: 1.0,
                    reset_time: future_time.to_string(),
                    observed_at: None,
                    cycle_tokens: None,
                    display_name: None,
                    description: None,
                }],
            },
        ]);
    }

    let score_a = score_candidate_account(&acc_a, "gemini-pro", now_sec);
    let score_b = score_candidate_account(&acc_b, "gemini-pro", now_sec);

    // Account A has 100% weekly quota while Account B has 21% -> score_a > score_b > 0
    assert!(score_a > 0.0);
    assert!(score_b > 0.0);
    assert!(score_a > score_b);
}

#[test]
pub(crate) fn test_score_candidate_account_fallback_partial_4h() {
    let now_sec = 1790090000;
    let past_time = "2026-09-22T14:30:00Z";
    let mut acc = make_test_account("acc-part", "part@domain.com", "gemini-pro", 50, past_time);
    acc.last_used = now_sec - 3600;

    // In primary scoring mode, an account with partial 4h quota (< 100%) and future reset scores 0.0
    let future_time = "2026-09-22T20:00:00Z";
    let acc_unrefilled = make_test_account(
        "acc-unrefilled",
        "unrefilled@domain.com",
        "gemini-pro",
        50,
        future_time,
    );
    assert_eq!(
        score_candidate_account(&acc_unrefilled, "gemini-pro", now_sec),
        0.0
    );

    // In fallback mode, partial 4h quota (50%) scales the base score:
    // For past_time (period finished): base_score = 504.0
    // Fallback score = (50 / 100) * 504.0 = 252.0
    let fallback_score = score_candidate_account_fallback(&acc, "gemini-pro", now_sec);
    assert_eq!(fallback_score, 252.0);
}
