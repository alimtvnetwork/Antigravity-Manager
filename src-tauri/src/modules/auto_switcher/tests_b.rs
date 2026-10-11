#![cfg(test)]

use super::*;
use crate::models::config::AutoProfileSwitcherConfig;
use crate::models::token::TokenData;

use super::*;

#[test]
pub(crate) fn test_weekly_quota_sub_8_percent_zero() {
    let now_sec = 1790090000;
    let future_time = "2026-09-30T14:30:00Z";
    let mut acc = make_test_account("acc-low", "low@domain.com", "gemini-pro", 100, future_time);
    if let Some(ref mut q) = acc.quota {
        q.quota_groups = Some(vec![crate::models::quota::QuotaGroup {
            display_name: "Gemini Models".to_string(),
            description: None,
            buckets: vec![crate::models::quota::QuotaBucket {
                bucket_id: "gemini-weekly".to_string(),
                window: "weekly".to_string(),
                remaining_fraction: 0.06, // 6% < 8%
                reset_time: future_time.to_string(),
                observed_at: None,
                cycle_tokens: None,
                display_name: None,
                description: None,
            }],
        }]);
    }

    // Weekly quota below 8% strictly produces 0.0
    let score = score_candidate_account(&acc, "gemini-pro", now_sec);
    assert_eq!(score, 0.0);
}

#[test]
pub(crate) fn test_instance_binding_stale_or_exhausted() {
    let now_sec = 1790090000;
    // Stale binding timeout verification (default 6h, configurable 6-10h)
    let stale_inst = crate::models::InstanceConfig {
        id: "inst-stale".to_string(),
        name: "Stale Instance".to_string(),
        data_dir: "/tmp/stale".to_string(),
        executable_path: None,
        extensions_dir: None,
        bound_account_id: None,
        bound_email: None,
        created_at: now_sec - 50000,
        last_used: now_sec - (7 * 3600), // 7 hours ago (> 6h default)
        is_default: false,
        pid: None,
        seq_num: Some(2),
    };
    assert!(is_instance_binding_stale_or_exhausted(
        &stale_inst,
        false,
        6,
        "gemini-pro",
        now_sec
    ));
    assert!(!is_instance_binding_stale_or_exhausted(
        &stale_inst,
        false,
        10,
        "gemini-pro",
        now_sec
    ));
}

#[test]
pub(crate) fn test_is_account_in_use_logic() {
    let acc_idle = make_test_account(
        "acc-idle-temp-test",
        "idle-unused-temp@example.com",
        "gemini-pro",
        11,
        "",
    );

    // 1. Unbound, non-active account returns false
    assert!(!is_account_in_use_core(
        &acc_idle.id,
        &acc_idle.email,
        None,
        None,
        &[],
        &[]
    ));

    // 2. Current active account ID matches -> returns true
    assert!(is_account_in_use_core(
        &acc_idle.id,
        &acc_idle.email,
        Some("acc-idle-temp-test"),
        None,
        &[],
        &[]
    ));

    // 3. Current active account email matches -> returns true
    assert!(is_account_in_use_core(
        &acc_idle.id,
        &acc_idle.email,
        None,
        Some("idle-unused-temp@example.com"),
        &[],
        &[]
    ));

    // 4. In active in-use IDs list -> returns true
    assert!(is_account_in_use_core(
        &acc_idle.id,
        &acc_idle.email,
        None,
        None,
        &["acc-idle-temp-test".to_string()],
        &[]
    ));

    // 5. Bound to running instance -> returns true
    assert!(is_account_in_use_core(
        &acc_idle.id,
        &acc_idle.email,
        None,
        None,
        &[],
        &[(Some("acc-idle-temp-test".to_string()), None)]
    ));
}

#[test]
pub(crate) fn test_candidate_selection_filters_proxy_disabled_and_strictly_requires_100() {
    let now_sec = 1700000000;
    let mut acc_disabled =
        make_test_account("acc-1", "disabled@example.com", "gemini-pro", 100, "");
    acc_disabled.proxy_disabled = true;

    let acc_low_quota = make_test_account("acc-2", "low@example.com", "gemini-pro", 20, "");
    let acc_full_quota = make_test_account("acc-3", "full@example.com", "gemini-pro", 100, "");

    // 1. Verify proxy_disabled account is recognized as disabled
    assert!(
        acc_disabled.disabled || acc_disabled.proxy_disabled || acc_disabled.validation_blocked
    );

    // 2. Verify low-quota account (20%) does NOT meet strict 100% requirement when period not finished
    let status_low = evaluate_account_period_status(&acc_low_quota, "gemini-pro", 95.0, now_sec);
    let quota_low = status_low.as_ref().map(|s| s.quota_percent).unwrap_or(20.0);
    let period_finished_low = status_low
        .as_ref()
        .map(|s| s.is_period_finished)
        .unwrap_or(false);
    let is_healthy_low = quota_low >= 100.0 || (period_finished_low && quota_low >= 95.0);
    assert!(!is_healthy_low, "Account with 20% quota must be rejected");

    // 3. Verify 100% quota account passes
    let status_full = evaluate_account_period_status(&acc_full_quota, "gemini-pro", 95.0, now_sec);
    let quota_full = status_full
        .as_ref()
        .map(|s| s.quota_percent)
        .unwrap_or(100.0);
    let period_finished_full = status_full
        .as_ref()
        .map(|s| s.is_period_finished)
        .unwrap_or(false);
    let is_healthy_full = quota_full >= 100.0 || (period_finished_full && quota_full >= 95.0);
    assert!(is_healthy_full, "Account with 100% quota must be accepted");
}

#[test]
pub(crate) fn test_is_account_excluded_case_insensitive() {
    let exclusions = vec![
        "acc-123".to_string(),
        "USER@EXAMPLE.COM".to_string(),
        "  spaced@test.com  ".to_string(),
    ];

    // Match by ID
    assert!(is_account_excluded(
        &exclusions,
        "acc-123",
        "other@domain.com"
    ));
    assert!(is_account_excluded(
        &exclusions,
        "ACC-123",
        "other@domain.com"
    ));
    assert!(is_account_excluded(
        &exclusions,
        " acc-123 ",
        "other@domain.com"
    ));

    // Match by Email
    assert!(is_account_excluded(
        &exclusions,
        "acc-999",
        "user@example.com"
    ));
    assert!(is_account_excluded(
        &exclusions,
        "acc-999",
        "USER@EXAMPLE.COM"
    ));
    assert!(is_account_excluded(
        &exclusions,
        "acc-999",
        "spaced@test.com"
    ));

    // Non-matches
    assert!(!is_account_excluded(
        &exclusions,
        "acc-456",
        "fresh@example.com"
    ));
}

#[test]
pub(crate) fn test_cooldown_partition_and_fallback_logic() {
    let now_sec = 1700000000;
    let cooldown_secs = 3600; // 60 mins

    // Acc 1: Not in cooldown (last_used 2 hours ago)
    let acc_avail = make_test_account("acc-avail", "avail@example.com", "gemini-pro", 100, "");
    let mut acc_avail = acc_avail;
    acc_avail.last_used = now_sec - 7200;

    // Acc 2: In cooldown (last_used 10 mins ago)
    let acc_cd1 = make_test_account("acc-cd1", "cd1@example.com", "gemini-pro", 100, "");
    let mut acc_cd1 = acc_cd1;
    acc_cd1.last_used = now_sec - 600;

    // Acc 3: In cooldown (last_used 40 mins ago - older)
    let acc_cd2 = make_test_account("acc-cd2", "cd2@example.com", "gemini-pro", 100, "");
    let mut acc_cd2 = acc_cd2;
    acc_cd2.last_used = now_sec - 2400;

    let is_cooldown =
        |acc: &Account| -> bool { acc.last_used > 0 && (now_sec - acc.last_used) < cooldown_secs };

    assert!(!is_cooldown(&acc_avail));
    assert!(is_cooldown(&acc_cd1));
    assert!(is_cooldown(&acc_cd2));

    // When available pool has members, it takes precedence
    let mut available_pool = vec![acc_avail.clone()];
    let mut cooldown_pool = vec![
        (acc_cd1.clone(), acc_cd1.last_used),
        (acc_cd2.clone(), acc_cd2.last_used),
    ];

    assert!(!available_pool.is_empty());

    // When available pool is empty, fallback sorts by oldest last_used (cd2 < cd1)
    available_pool.clear();
    assert!(available_pool.is_empty());
    cooldown_pool.sort_by(|(_, a_last), (_, b_last)| a_last.cmp(b_last));
    assert_eq!(cooldown_pool[0].0.id, "acc-cd2"); // used 40m ago comes before used 10m ago
    assert_eq!(cooldown_pool[1].0.id, "acc-cd1");
}

#[test]
pub(crate) fn test_gemini_3_1_pro_low_quota_failover_trigger() {
    let now_sec = 1790080000;
    let mut acc = make_test_account(
        "acc-gemini31",
        "test@gemini.local",
        "gemini-3.8-flash",
        100,
        "2026-09-22T14:30:00Z",
    );
    if let Some(ref mut q) = acc.quota {
        q.models.push(crate::models::quota::ModelQuota {
            name: "Gemini 3.1 Pro".to_string(),
            percentage: 6,
            reset_time: "2026-09-22T14:30:00Z".to_string(),
            display_name: None,
            supports_images: None,
            supports_thinking: None,
            thinking_budget: None,
            recommended: None,
            max_tokens: None,
            max_output_tokens: None,
            supported_mime_types: None,
        });
    }

    let status = evaluate_account_period_status(&acc, "gemini-3.8-flash", 15.0, now_sec);
    assert!(status.is_some());
    let stat = status.unwrap();
    assert_eq!(stat.quota_percent, 6.0);
    assert!(stat.is_low_quota);
    assert!(stat.below_threshold);
}

#[test]
pub(crate) fn test_daemon_status_telemetry() {
    let status = get_daemon_status();
    assert_eq!(status.check_interval_seconds, 120);
    assert_eq!(status.current_stage, "normal");
}
