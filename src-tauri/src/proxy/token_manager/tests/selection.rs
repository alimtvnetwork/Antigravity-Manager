//! P2C selection algorithm and account priority tests.

use super::super::ProxyToken;
use super::super::TokenManager;
use super::helpers::*;
use std::time::Duration;

#[test]
fn test_p2c_selects_higher_quota() {
    // P2C 应选择配额更高的账号
    let manager = TokenManager::new(PathBuf::from("/tmp/test"));

    let low_quota = create_test_token("low@test.com", Some("PRO"), 1.0, None, Some(20));
    let high_quota = create_test_token("high@test.com", Some("PRO"), 1.0, None, Some(80));

    let candidates = vec![low_quota, high_quota];
    let attempted: HashSet<String> = HashSet::new();

    // 运行多次确保选择高配额账号
    for _ in 0..10 {
        let result = manager.select_with_p2c(&candidates, &attempted, "claude-sonnet", false);
        assert!(result.is_some());
        // P2C 从两个候选中选择配额更高的
        // 由于只有两个候选，应该总是选择 high_quota
        assert_eq!(result.unwrap().email, "high@test.com");
    }
}

#[test]
fn account_priority_p2c_stays_in_highest_available_group() {
    let manager = TokenManager::new(PathBuf::new());
    let mut high = create_test_token("high", Some("FREE"), 0.5, None, Some(1));
    high.priority = 1;
    let low = create_test_token("low", Some("ULTRA"), 1.0, None, Some(100));
    let mut candidates = vec![low, high]; // Deliberately unsorted.
    let mut attempted = HashSet::new();
    for _ in 0..20 {
        let selected = manager
            .select_with_p2c(&candidates, &attempted, "claude", true)
            .unwrap();
        assert_eq!(selected.account_id, "high");
    }
    attempted.insert("high".to_string());
    let selected = manager
        .select_with_p2c(&candidates, &attempted, "claude", true)
        .unwrap();
    assert_eq!(selected.account_id, "low");
    attempted.clear();
    candidates[1].protected_models.insert("claude".to_string());
    let selected = manager
        .select_with_p2c(&candidates, &attempted, "claude", true)
        .unwrap();
    assert_eq!(selected.account_id, "low");
}

#[tokio::test]
async fn account_priority_save_reselects_preserving_sessions_and_limits() {
    async fn select(manager: &TokenManager, group: &str, session: Option<&str>) -> String {
        manager
            .get_token(group, false, session, "claude-sonnet-4-6")
            .await
            .unwrap()
            .3
    }
    let _dir = crate::proxy::monitor::prompt_log_tests::TestDataDir::new();
    let data_dir = crate::modules::account::get_data_dir().unwrap();
    let manager = TokenManager::new(data_dir.clone());
    for (id, priority) in [("high", 1), ("low", 100)] {
        let mut account = weekly_quota_account(chrono::Utc::now().timestamp());
        account["id"] = serde_json::json!(id);
        account["email"] = serde_json::json!(format!("{id}@test.invalid"));
        account["priority"] = serde_json::json!(priority);
        let account: crate::models::Account = serde_json::from_value(account).unwrap();
        crate::modules::account::save_account(&account).unwrap();
        manager.reload_account(id).await.unwrap();
    }
    assert_eq!(select(&manager, "claude", Some("existing")).await, "high");
    crate::modules::account::update_account_priority("high", 100).unwrap();
    manager.update_account_priority("high", 100);
    crate::modules::account::update_account_priority("low", 1).unwrap();
    manager.update_account_priority("low", 1);
    assert_eq!(
        crate::modules::account::load_account("low")
            .unwrap()
            .priority,
        1
    );
    assert_eq!(select(&manager, "claude", Some("existing")).await, "high");
    assert_eq!(select(&manager, "claude", Some("new")).await, "low");
    manager
        .set_preferred_account(Some("high".to_string()))
        .await;
    assert_eq!(select(&manager, "image_gen", None).await, "high");
    manager.set_preferred_account(None).await;
    manager.rate_limit_tracker.set_lockout_until(
        "low",
        std::time::SystemTime::now() + Duration::from_secs(60),
        crate::proxy::rate_limit::RateLimitReason::QuotaExhausted,
        Some("claude".to_string()),
    );
    manager.update_account_priority("low", 2);
    assert!(manager
        .rate_limit_tracker
        .is_rate_limited("low", Some("claude")));
    assert_eq!(select(&manager, "image_gen", None).await, "high");
    manager.rate_limit_tracker.clear("low");
    // A stale/failed disk candidate must not remove the lower-priority fallback.
    std::fs::write(data_dir.join("accounts/low.json"), "invalid JSON").unwrap();
    assert_eq!(select(&manager, "image_gen", None).await, "high");
}

#[test]
fn test_p2c_skips_attempted() {
    // P2C 应跳过已尝试的账号
    let manager = TokenManager::new(PathBuf::from("/tmp/test"));

    let token_a = create_test_token("a@test.com", Some("PRO"), 1.0, None, Some(80));
    let token_b = create_test_token("b@test.com", Some("PRO"), 1.0, None, Some(50));

    let candidates = vec![token_a, token_b];
    let mut attempted: HashSet<String> = HashSet::new();
    attempted.insert("a@test.com".to_string());

    let result = manager.select_with_p2c(&candidates, &attempted, "claude-sonnet", false);
    assert!(result.is_some());
    assert_eq!(result.unwrap().email, "b@test.com");
}

#[test]
fn test_p2c_skips_protected_models() {
    // P2C 应跳过对目标模型有保护的账号 (quota_protection_enabled = true)
    let manager = TokenManager::new(PathBuf::from("/tmp/test"));

    let mut protected = HashSet::new();
    protected.insert("claude-sonnet".to_string());

    let protected_account =
        create_test_token_with_protected("protected@test.com", Some(90), protected);
    let normal_account =
        create_test_token_with_protected("normal@test.com", Some(50), HashSet::new());

    let candidates = vec![protected_account, normal_account];
    let attempted: HashSet<String> = HashSet::new();

    let result = manager.select_with_p2c(&candidates, &attempted, "claude-sonnet", true);
    assert!(result.is_some());
    assert_eq!(result.unwrap().email, "normal@test.com");
}

#[test]
fn test_p2c_single_candidate() {
    // 单候选时直接返回
    let manager = TokenManager::new(PathBuf::from("/tmp/test"));

    let token = create_test_token("single@test.com", Some("PRO"), 1.0, None, Some(50));
    let candidates = vec![token];
    let attempted: HashSet<String> = HashSet::new();

    let result = manager.select_with_p2c(&candidates, &attempted, "claude-sonnet", false);
    assert!(result.is_some());
    assert_eq!(result.unwrap().email, "single@test.com");
}

#[test]
fn test_p2c_empty_candidates() {
    // 空候选返回 None
    let manager = TokenManager::new(PathBuf::from("/tmp/test"));

    let candidates: Vec<ProxyToken> = vec![];
    let attempted: HashSet<String> = HashSet::new();

    let result = manager.select_with_p2c(&candidates, &attempted, "claude-sonnet", false);
    assert!(result.is_none());
}

#[test]
fn test_p2c_all_attempted() {
    // 所有账号都已尝试时返回 None
    let manager = TokenManager::new(PathBuf::from("/tmp/test"));

    let token_a = create_test_token("a@test.com", Some("PRO"), 1.0, None, Some(80));
    let token_b = create_test_token("b@test.com", Some("PRO"), 1.0, None, Some(50));

    let candidates = vec![token_a, token_b];
    let mut attempted: HashSet<String> = HashSet::new();
    attempted.insert("a@test.com".to_string());
    attempted.insert("b@test.com".to_string());

    let result = manager.select_with_p2c(&candidates, &attempted, "claude-sonnet", false);
    assert!(result.is_none());
}

// ===== Ultra 优先逻辑测试 =====
