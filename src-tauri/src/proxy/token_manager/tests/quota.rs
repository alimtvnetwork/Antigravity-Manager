//! weekly quota windows and quota protection tests.

use super::super::helpers::update_account_json;
use super::super::TokenManager;
use super::helpers::*;
use std::time::Duration;

#[tokio::test]
async fn weekly_quota_blocks_by_default_and_survives_reload_and_resets() {
    let _data_dir = crate::proxy::monitor::prompt_log_tests::TestDataDir::new();
    let data_dir = crate::modules::account::get_data_dir().unwrap();
    let accounts = data_dir.join("accounts");
    std::fs::create_dir(&accounts).unwrap();
    let mut snapshot = weekly_quota_account(chrono::Utc::now().timestamp());
    snapshot["quota"]["quota_groups"][0]["buckets"][0]["remaining_fraction"] =
        serde_json::json!(0.0005);
    std::fs::write(accounts.join("weekly-test.json"), snapshot.to_string()).unwrap();
    let manager = TokenManager::new(data_dir);
    manager.load_accounts().await.unwrap();
    assert!(
        manager
            .is_rate_limited("weekly-test", Some("gemini-3-pro-high"))
            .await
    );
    assert!(!manager.is_rate_limited("weekly-test", Some("claude")).await);
    manager.circuit_breaker_config.write().await.enabled = false;
    manager.reload_account("weekly-test").await.unwrap();
    manager.load_accounts().await.unwrap();
    manager.rate_limit_tracker.clear_for_optimistic_reset();
    manager.clear_all_rate_limits();
    assert!(
        manager
            .is_rate_limited("weekly-test", Some("gemini-3-pro-high"))
            .await
    );
    assert!(manager
        .get_token("gemini", false, None, "gemini-3.1-pro-high")
        .await
        .is_err());
    assert!(manager
        .get_token("claude", false, None, "claude-sonnet-4-6")
        .await
        .is_ok());
}

#[tokio::test]
async fn weekly_quota_recovery_requires_new_same_bucket_and_preserves_other_limits() {
    let manager = TokenManager::new(PathBuf::new());
    let now = chrono::Utc::now().timestamp();
    let mut snapshot = weekly_quota_account(now);
    snapshot["quota"]["quota_groups"][0]["buckets"][0]["remaining_fraction"] =
        serde_json::json!(0.001);
    let tracker = &manager.rate_limit_tracker;
    manager
        .circuit_breaker_config
        .write()
        .await
        .lock_on_zero_quota = true;
    snapshot["quota"]["quota_groups"][0]["buckets"][1]["remaining_fraction"] = serde_json::json!(0);
    manager.sync_zero_quota_circuit_breaker("a", &snapshot);
    tracker.set_lockout_until_with_cap(
        "a",
        std::time::SystemTime::now() + Duration::from_secs(10800),
        crate::proxy::rate_limit::RateLimitReason::QuotaExhausted,
        Some("gemini-3-pro-image".into()),
        false,
    );
    tracker.set_lockout_until(
        "a",
        std::time::SystemTime::now() + Duration::from_secs(120),
        crate::proxy::rate_limit::RateLimitReason::RateLimitExceeded,
        None,
    );
    let mut positive = snapshot.clone();
    positive["quota"]["quota_groups"][0]["buckets"][0]["remaining_fraction"] =
        serde_json::json!(0.0011);
    manager.sync_zero_quota_circuit_breaker("a", &positive); // Same old snapshot cannot unlock.
    manager.sync_zero_quota_circuit_breaker("a", &serde_json::json!({"quota": {"models": []}}));
    assert!(tracker.get_quota_wait("a", Some("gemini-3-pro-high"), true) > 7000);
    for (offset, fraction) in [(1, 0.0005), (2, 0.001)] {
        positive["quota"]["last_updated"] = serde_json::json!(now + offset);
        positive["quota"]["quota_groups"][0]["buckets"][0]["remaining_fraction"] =
            serde_json::json!(fraction);
        manager.sync_zero_quota_circuit_breaker("a", &positive);
        assert!(tracker.get_quota_wait("a", Some("gemini-3-pro-high"), true) > 7000);
    }
    positive["quota"]["last_updated"] = serde_json::json!(now + 3);
    positive["quota"]["quota_groups"][0]["buckets"][0]["remaining_fraction"] =
        serde_json::json!(0.0011);
    manager.sync_zero_quota_circuit_breaker("a", &positive);
    assert_eq!(
        tracker.get_quota_wait("a", Some("gemini-3-pro-high"), true),
        0
    );
    assert!(tracker.get_remaining_wait("a", Some("gemini-3-pro-high")) > 1700); // 5h still exhausted.
    assert!(tracker.get_remaining_wait("a", Some("gemini-3-pro-image")) > 10000);
    assert!(tracker.is_rate_limited("a", None)); // Independent account-level upstream limit.
    manager.sync_zero_quota_circuit_breaker("a", &snapshot); // Older zero must not relock.
    assert_eq!(
        tracker.get_quota_wait("a", Some("gemini-3-pro-high"), true),
        0
    );
}

#[test]
fn weekly_quota_missing_data_persists_for_restart_without_renewing_observation() {
    let now = chrono::Utc::now().timestamp();
    let mut account: crate::models::Account =
        serde_json::from_value(weekly_quota_account(now)).unwrap();
    let mut refresh = account.quota.clone().unwrap();
    refresh.last_updated += 1;
    refresh.quota_groups.as_mut().unwrap()[0].buckets.remove(0); // Partial summary / 5h recovery.
    account.update_quota(refresh);
    let mut failed = account.quota.clone().unwrap();
    failed.last_updated += 1;
    failed.quota_groups = None;
    account.update_quota(failed);
    let snapshot = serde_json::to_value(&account).unwrap();
    let bucket = &snapshot["quota"]["quota_groups"][0]["buckets"][1];
    assert_eq!(bucket["bucket_id"], "gemini-weekly");
    assert_eq!(bucket["observed_at"], now * 1000);
    let restarted = TokenManager::new(PathBuf::new());
    restarted.sync_zero_quota_circuit_breaker("a", &snapshot);
    assert!(restarted
        .rate_limit_tracker
        .is_rate_limited("a", Some("gemini-3-pro-high")));
}

#[tokio::test]
async fn weekly_quota_windows_are_order_independent_and_expired_snapshots_stay_expired() {
    let now = chrono::Utc::now().timestamp();
    for reverse in [false, true] {
        let manager = TokenManager::new(PathBuf::new());
        manager
            .circuit_breaker_config
            .write()
            .await
            .lock_on_zero_quota = true;
        let mut snapshot = weekly_quota_account(now);
        let buckets = snapshot["quota"]["quota_groups"][0]["buckets"]
            .as_array_mut()
            .unwrap();
        buckets[1]["remaining_fraction"] = serde_json::json!(0);
        if reverse {
            buckets.reverse();
        }
        manager.sync_zero_quota_circuit_breaker("a", &snapshot);
        for model in [
            "gemini-3-pro-high",
            "gemini-3-flash",
            "gemini-3.1-flash-image",
            "gemini-3-pro-image",
        ] {
            assert!(
                manager
                    .rate_limit_tracker
                    .get_remaining_wait("a", Some(model))
                    > 7000
            );
        }
        let expired = weekly_quota_account(now - 8000);
        manager.sync_zero_quota_circuit_breaker("expired", &expired);
        manager.sync_zero_quota_circuit_breaker("expired", &expired);
        assert!(!manager
            .rate_limit_tracker
            .is_rate_limited("expired", Some("gemini-3-pro-high")));
    }
}

#[test]
fn test_build_dynamic_model_candidates_agent() {
    let candidates = TokenManager::build_dynamic_model_candidates("gemini-pro-agent").unwrap();
    assert_eq!(candidates[0], "gemini-pro-agent");
    assert!(candidates.contains(&"gemini-3.1-pro-low".to_string()));

    let candidates_high =
        TokenManager::build_dynamic_model_candidates("gemini-3.1-pro-high").unwrap();
    assert_eq!(candidates_high[0], "gemini-3.1-pro-high");
    assert!(candidates_high.contains(&"gemini-pro-agent".to_string()));
}

#[tokio::test]
async fn task_account_json_update_preserves_live_limits() {
    let tmp_root = std::env::temp_dir().join(format!(
        "antigravity-account-update-test-{}",
        uuid::Uuid::new_v4()
    ));
    let accounts_dir = tmp_root.join("accounts");
    std::fs::create_dir_all(&accounts_dir).unwrap();
    let account_id = "acc-update";
    let account_path = accounts_dir.join(format!("{}.json", account_id));
    let live_limit = serde_json::json!({
        "model": "gemini-3-pro-image",
        "status": 429,
        "reason": "QuotaExhausted",
        "until": chrono::Utc::now().timestamp() + 7200,
        "detected_at": chrono::Utc::now().timestamp(),
        "message": "QUOTA_EXHAUSTED; reset after 2h"
    });
    std::fs::write(
        &account_path,
        serde_json::to_string_pretty(&serde_json::json!({
            "id": account_id,
            "live_limited_models": {
                "gemini-3-pro-image": live_limit
            }
        }))
        .unwrap(),
    )
    .unwrap();

    let manager = TokenManager::new(tmp_root.clone());
    manager.disable_account(account_id, "test").await.unwrap();

    let updated: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&account_path).unwrap()).unwrap();
    assert_eq!(
        updated["live_limited_models"]["gemini-3-pro-image"],
        live_limit
    );
    assert_eq!(updated["disabled"], true);

    let mut account_snapshot = updated;
    assert!(manager
        .trigger_quota_protection(
            &mut account_snapshot,
            account_id,
            &account_path,
            0,
            10,
            "gemini-3-flash",
        )
        .await
        .unwrap());
    assert!(manager
        .restore_quota_protection(
            &mut account_snapshot,
            account_id,
            &account_path,
            "gemini-3-flash",
        )
        .await
        .unwrap());

    account_snapshot["proxy_disabled"] = serde_json::Value::Bool(true);
    account_snapshot["proxy_disabled_reason"] =
        serde_json::Value::String("quota_protection".to_string());
    let quota = serde_json::json!({
        "models": [{ "name": "gemini-3-flash", "percentage": 0 }]
    });
    let config = crate::models::QuotaProtectionConfig {
        enabled: true,
        threshold_percentage: 10,
        monitored_models: vec!["gemini-3-flash".to_string()],
    };
    manager
        .check_and_restore_quota(&mut account_snapshot, &account_path, &quota, &config)
        .await;

    update_account_json(&account_path, |latest| {
        latest["validation_blocked"] = serde_json::Value::Bool(true);
        latest["validation_blocked_until"] =
            serde_json::Value::Number((chrono::Utc::now().timestamp() - 1).into());
        latest["validation_blocked_reason"] = serde_json::Value::String("expired".to_string());
    })
    .await
    .unwrap();
    manager.load_single_account(&account_path).await.unwrap();

    let updated: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&account_path).unwrap()).unwrap();
    assert_eq!(
        updated["live_limited_models"]["gemini-3-pro-image"],
        live_limit
    );
    assert_eq!(updated["validation_blocked"], false);
    assert_eq!(
        updated["protected_models"],
        serde_json::json!(["gemini-3-flash"])
    );

    // Justification: best-effort cleanup; a leftover directory is harmless
    crate::error::record_ignored(std::fs::remove_dir_all(&tmp_root), "remove_dir_all");
}
