//! Local-only End-to-End Integration Tests for Auto-Switcher
//! Run manually with: cargo test --test auto_switcher_e2e_test -- --ignored

use antigravity_tools_lib::models::account::Account;
use antigravity_tools_lib::models::config::AutoProfileSwitcherConfig;
use antigravity_tools_lib::models::quota::{ModelQuota, QuotaData};
use antigravity_tools_lib::models::token::TokenData;
use antigravity_tools_lib::modules::auto_switcher::{
    calculate_4h_window_quota, calculate_account_quota, calculate_next_interval_seconds,
    evaluate_account_period_status, get_daemon_status,
};
use antigravity_tools_lib::modules::instance::{
    get_instance_workspace_paths, restore_and_inject_prompts_for_instance,
};

fn create_sample_account(id: &str, email: &str, model_name: &str, pct: i32) -> Account {
    let token = TokenData::new(
        "mock_access".to_string(),
        "mock_refresh".to_string(),
        3600,
        Some(email.to_string()),
        None,
        None,
        false,
        None,
    );
    let mut acc = Account::new(id.to_string(), email.to_string(), token);
    acc.quota = Some(QuotaData {
        models: vec![ModelQuota {
            name: model_name.to_string(),
            percentage: pct,
            reset_time: "2026-10-04T18:00:00Z".to_string(),
            display_name: None,
            supports_images: None,
            supports_thinking: None,
            thinking_budget: None,
            recommended: None,
            max_tokens: None,
            max_output_tokens: None,
            supported_mime_types: None,
        }],
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

#[tokio::test]
#[ignore = "local-only e2e test, run manually via cargo test --test auto_switcher_e2e_test -- --ignored"]
async fn test_e2e_low_quota_detection_triggers_failover() {
    let now_sec = chrono::Utc::now().timestamp();
    let mut acc = create_sample_account("acc-active", "active@user.local", "gemini-3.8-flash", 100);

    // Actively consumed Gemini 3.1 Pro down to 6% (below 15% threshold)
    if let Some(ref mut q) = acc.quota {
        q.models.push(ModelQuota {
            name: "Gemini 3.1 Pro".to_string(),
            percentage: 6,
            reset_time: "2026-10-04T22:00:00Z".to_string(),
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
    assert!(status.is_some(), "Expected quota period status");
    let stat = status.unwrap();

    assert_eq!(
        stat.quota_percent, 6.0,
        "Should detect 6% lowest consumed model"
    );
    assert!(stat.is_low_quota, "is_low_quota must be true at 6%");
    assert!(
        stat.below_threshold,
        "below_threshold must be true to trigger rotation"
    );

    // Also check direct calculation functions reflect 6%
    assert_eq!(calculate_account_quota(&acc, "gemini-3.8-flash"), Some(6.0));
    assert_eq!(
        calculate_4h_window_quota(&acc, "gemini-3.8-flash"),
        Some(6.0)
    );
}

#[tokio::test]
#[ignore = "local-only e2e test, run manually via cargo test --test auto_switcher_e2e_test -- --ignored"]
async fn test_e2e_candidate_quota_not_disqualified_by_other_depleted_models() {
    let mut candidate_acc =
        create_sample_account("acc-cand", "cand@user.local", "gemini-3.8-flash", 100);
    // Add another model that was exhausted to 0%
    if let Some(ref mut q) = candidate_acc.quota {
        q.models.push(ModelQuota {
            name: "Claude 3.7 Sonnet".to_string(),
            percentage: 0,
            reset_time: "2026-10-04T22:00:00Z".to_string(),
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

    // Candidate must evaluate to 100% for target model gemini-3.8-flash, not 0%
    let candidate_quota = antigravity_tools_lib::modules::auto_switcher::calculate_candidate_quota(
        &candidate_acc,
        "gemini-3.8-flash",
    );
    assert_eq!(
        candidate_quota,
        Some(100.0),
        "Candidate should have 100% quota on target model"
    );
}

#[tokio::test]
#[ignore = "local-only e2e test, run manually via cargo test --test auto_switcher_e2e_test -- --ignored"]
async fn test_e2e_prompt_backup_snapshotting() {
    let inst_id = "test-e2e-instance";
    let backup_repo_res = antigravity_tools_lib::modules::repo_db::backup_running_prompts(inst_id);
    assert!(backup_repo_res.is_ok(), "repo_db backup should succeed");

    let backup_active_res =
        antigravity_tools_lib::modules::backup_prompts_db::backup_active_running_prompts(
            Some(inst_id),
            None,
        );
    assert!(
        backup_active_res.is_ok(),
        "backup_prompts_db should succeed"
    );
}

#[tokio::test]
#[ignore = "local-only e2e test, run manually via cargo test --test auto_switcher_e2e_test -- --ignored"]
async fn test_e2e_async_5s_prompt_restoration_timing() {
    let start = std::time::Instant::now();
    let delay_dur = std::time::Duration::from_millis(100);

    // Simulate the async delay pattern
    tokio::time::sleep(delay_dur).await;
    let elapsed = start.elapsed();
    assert!(elapsed >= delay_dur, "Timer should elapse correctly");

    let workspace_roots = get_instance_workspace_paths("default");
    let restore_res = restore_and_inject_prompts_for_instance("default", &workspace_roots);
    assert!(
        restore_res.is_ok(),
        "Prompt restoration should complete without error"
    );
}

#[tokio::test]
#[ignore = "local-only e2e test, run manually via cargo test --test auto_switcher_e2e_test -- --ignored"]
async fn test_e2e_daemon_status_and_intervals() {
    let cfg = AutoProfileSwitcherConfig::default();
    assert_eq!(calculate_next_interval_seconds(Some(5.0), &cfg), 40);
    assert_eq!(calculate_next_interval_seconds(Some(16.0), &cfg), 60);
    assert_eq!(calculate_next_interval_seconds(Some(90.0), &cfg), 120);

    let daemon_status = get_daemon_status();
    assert!(daemon_status.check_interval_seconds >= 10);
    assert!(!daemon_status.current_stage.is_empty());
}

#[tokio::test]
#[ignore = "local-only e2e test, run manually via cargo test --test auto_switcher_e2e_test -- --ignored"]
async fn test_e2e_period_finished_does_not_suppress_low_quota_rotation() {
    let now_sec = chrono::Utc::now().timestamp();
    let mut acc = create_sample_account("acc-active", "active@user.local", "gemini-3.8-flash", 6);
    // Reset time is in the past (period finished)
    if let Some(ref mut q) = acc.quota {
        if let Some(m) = q.models.first_mut() {
            m.reset_time = "2026-10-04T00:00:00Z".to_string();
        }
    }

    let status = evaluate_account_period_status(&acc, "gemini-3.8-flash", 15.0, now_sec);
    assert!(status.is_some());
    let stat = status.unwrap();
    assert!(stat.is_period_finished, "Period must be finished");
    assert!(stat.is_low_quota, "Must still be detected as low quota");
    assert!(stat.below_threshold, "below_threshold must still be true");
}
