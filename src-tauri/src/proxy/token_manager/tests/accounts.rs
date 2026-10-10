//! account load/reload, fixed-account mode, sticky session tests.

use super::super::TokenManager;
use super::helpers::*;

#[tokio::test]
async fn task_reload_account_preserves_live_limit_and_syncs_disabled_state() {
    let tmp_root = std::env::temp_dir().join(format!(
        "antigravity-token-manager-test-{}",
        uuid::Uuid::new_v4()
    ));
    let accounts_dir = tmp_root.join("accounts");
    std::fs::create_dir_all(&accounts_dir).unwrap();

    let account_id = "acc1";
    let model = "gemini-3-pro-image";
    let email = "a@test.com";
    let now = chrono::Utc::now().timestamp();
    let account_path = accounts_dir.join(format!("{}.json", account_id));

    let account_json = serde_json::json!({
        "id": account_id,
        "email": email,
        "token": {
            "access_token": "atk",
            "refresh_token": "rtk",
            "expires_in": 3600,
            "expiry_timestamp": now + 3600
        },
        "disabled": false,
        "proxy_disabled": false,
        "created_at": now,
        "last_used": now,
        "live_limited_models": {
            model: {
                "model": model,
                "status": 429,
                "reason": "QuotaExhausted",
                "until": now + 7200,
                "detected_at": now,
                "message": "{\"error\":{\"details\":[{\"reason\":\"QUOTA_EXHAUSTED\",\"metadata\":{\"quotaResetDelay\":\"2h\"}}]}}"
            },
            "gemini-3.1-flash-image": {
                "model": "gemini-3.1-flash-image",
                "status": 429,
                "reason": "QuotaExhausted",
                "until": now + 7200,
                "detected_at": now,
                "message": "QUOTA_EXHAUSTED"
            },
            "gemini-2.5-pro": {
                "model": "gemini-2.5-pro",
                "status": 429,
                "reason": "QuotaExhausted",
                "until": now + 7200,
                "detected_at": now,
                "message": "QUOTA_EXHAUSTED; reset after 2h"
            }
        }
    });
    std::fs::write(
        &account_path,
        serde_json::to_string_pretty(&account_json).unwrap(),
    )
    .unwrap();

    let manager = TokenManager::new(tmp_root.clone());
    manager.load_accounts().await.unwrap();
    assert!(manager.tokens.get(account_id).is_some());
    assert!(manager
        .rate_limit_tracker
        .is_rate_limited(account_id, Some(model)));
    assert!(!manager
        .rate_limit_tracker
        .is_rate_limited(account_id, Some("gemini-3.1-flash-image")));
    assert!(!manager
        .rate_limit_tracker
        .is_rate_limited(account_id, Some("gemini-2.5-pro")));

    let temporary_model = "gemini-3.1-flash-image";
    manager.rate_limit_tracker.parse_from_error(
        account_id,
        429,
        Some("60"),
        "temporary rate limit",
        Some(temporary_model.to_string()),
        &[60, 300],
    );
    assert!(manager
        .rate_limit_tracker
        .is_rate_limited(account_id, Some(temporary_model)));
    assert!(manager.clear_rate_limit_memory(account_id));
    assert!(!manager
        .rate_limit_tracker
        .is_rate_limited(account_id, Some(model)));

    manager.reload_account(account_id).await.unwrap();
    assert!(manager
        .rate_limit_tracker
        .is_rate_limited(account_id, Some(model)));
    assert!(!manager
        .rate_limit_tracker
        .is_rate_limited(account_id, Some(temporary_model)));

    // Prime extra caches to ensure remove_account() is really called.
    manager
        .session_accounts
        .insert("sid1".to_string(), account_id.to_string());
    {
        let mut preferred = manager.preferred_account_id.write().await;
        *preferred = Some(account_id.to_string());
    }

    // Mark account as proxy-disabled on disk (manual disable).
    let mut disabled_json = account_json.clone();
    disabled_json["proxy_disabled"] = serde_json::Value::Bool(true);
    disabled_json["proxy_disabled_reason"] = serde_json::Value::String("manual".to_string());
    disabled_json["proxy_disabled_at"] = serde_json::Value::Number(now.into());
    std::fs::write(
        &account_path,
        serde_json::to_string_pretty(&disabled_json).unwrap(),
    )
    .unwrap();

    manager.reload_account(account_id).await.unwrap();

    assert!(manager.tokens.get(account_id).is_none());
    assert!(manager.session_accounts.get("sid1").is_none());
    assert!(manager.preferred_account_id.read().await.is_none());

    disabled_json["proxy_disabled"] = serde_json::Value::Bool(false);
    std::fs::write(
        &account_path,
        serde_json::to_string_pretty(&disabled_json).unwrap(),
    )
    .unwrap();
    manager.reload_account(account_id).await.unwrap();
    assert!(manager
        .rate_limit_tracker
        .is_rate_limited(account_id, Some(model)));

    let restarted = TokenManager::new(tmp_root.clone());
    restarted.load_accounts().await.unwrap();
    assert!(restarted
        .rate_limit_tracker
        .is_rate_limited(account_id, Some(model)));

    // Justification: best-effort cleanup; a leftover directory is harmless
    crate::error::record_ignored(std::fs::remove_dir_all(&tmp_root), "remove_dir_all");
}

#[tokio::test]
async fn test_fixed_account_mode_skips_preferred_when_disabled_on_disk_without_reload() {
    let tmp_root = std::env::temp_dir().join(format!(
        "antigravity-token-manager-test-fixed-mode-{}",
        uuid::Uuid::new_v4()
    ));
    let accounts_dir = tmp_root.join("accounts");
    std::fs::create_dir_all(&accounts_dir).unwrap();

    let now = chrono::Utc::now().timestamp();

    let write_account = |id: &str, email: &str, proxy_disabled: bool| {
        let account_path = accounts_dir.join(format!("{}.json", id));
        let json = serde_json::json!({
            "id": id,
            "email": email,
            "token": {
                "access_token": format!("atk-{}", id),
                "refresh_token": format!("rtk-{}", id),
                "expires_in": 3600,
                "expiry_timestamp": now + 3600,
                "project_id": format!("pid-{}", id)
            },
            "quota": {
                "models": [
                    { "name": "gemini-1.5-flash", "percentage": 100 }
                ]
            },
            "disabled": false,
            "proxy_disabled": proxy_disabled,
            "proxy_disabled_reason": if proxy_disabled { "manual" } else { "" },
            "quota": {
                "models": [
                    { "name": "gemini-3-flash", "percentage": 100 }
                ]
            },
            "created_at": now,
            "last_used": now
        });
        std::fs::write(&account_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();
    };

    // Two accounts in pool.
    write_account("acc1", "a@test.com", false);
    write_account("acc2", "b@test.com", false);

    let manager = TokenManager::new(tmp_root.clone());
    manager.load_accounts().await.unwrap();

    // Enable fixed account mode for acc1.
    manager
        .set_preferred_account(Some("acc1".to_string()))
        .await;

    // Disable acc1 on disk WITHOUT reloading the in-memory pool (simulates stale cache).
    write_account("acc1", "a@test.com", true);

    let (_token, _project_id, email, account_id, _wait_ms) = manager
        .get_token("gemini", false, Some("sid1"), "gemini-1.5-flash")
        .await
        .unwrap();

    // Should fall back to another account instead of using the disabled preferred one.
    assert_eq!(account_id, "acc2");
    assert_eq!(email, "b@test.com");
    assert!(manager.tokens.get("acc1").is_none());
    assert!(manager.get_preferred_account().await.is_none());

    // Justification: best-effort cleanup; a leftover directory is harmless
    crate::error::record_ignored(std::fs::remove_dir_all(&tmp_root), "remove_dir_all");
}

#[tokio::test]
async fn test_collected_models_preserve_raw_quota_model_names_for_model_listing() {
    let tmp_root = std::env::temp_dir().join(format!(
        "antigravity-token-manager-test-raw-models-{}",
        uuid::Uuid::new_v4()
    ));
    let accounts_dir = tmp_root.join("accounts");
    std::fs::create_dir_all(&accounts_dir).unwrap();

    let now = chrono::Utc::now().timestamp();
    let account_path = accounts_dir.join("acc1.json");
    let account_json = serde_json::json!({
        "id": "acc1",
        "email": "a@test.com",
        "token": {
            "access_token": "atk",
            "refresh_token": "rtk",
            "expires_in": 3600,
            "expiry_timestamp": now + 3600
        },
        "quota": {
            "models": [
                { "name": "gemini-3-flash-agent", "percentage": 88 }
            ]
        },
        "disabled": false,
        "proxy_disabled": false,
        "created_at": now,
        "last_used": now
    });
    std::fs::write(
        &account_path,
        serde_json::to_string_pretty(&account_json).unwrap(),
    )
    .unwrap();

    let manager = TokenManager::new(tmp_root.clone());
    manager.load_accounts().await.unwrap();

    let collected_models = manager.get_all_collected_models();
    assert!(collected_models.contains("gemini-3-flash-agent"));
    // Keep the normalized quota bucket too; it is used for quota/protection checks.
    assert!(collected_models.contains("gemini-3-flash"));

    // Justification: best-effort cleanup; a leftover directory is harmless
    crate::error::record_ignored(std::fs::remove_dir_all(&tmp_root), "remove_dir_all");
}

#[tokio::test]
async fn test_sticky_session_skips_bound_account_when_disabled_on_disk_without_reload() {
    let tmp_root = std::env::temp_dir().join(format!(
        "antigravity-token-manager-test-sticky-disabled-{}",
        uuid::Uuid::new_v4()
    ));
    let accounts_dir = tmp_root.join("accounts");
    std::fs::create_dir_all(&accounts_dir).unwrap();

    let now = chrono::Utc::now().timestamp();

    let write_account = |id: &str, email: &str, percentage: i64, proxy_disabled: bool| {
        let account_path = accounts_dir.join(format!("{}.json", id));
        let json = serde_json::json!({
            "id": id,
            "email": email,
            "token": {
                "access_token": format!("atk-{}", id),
                "refresh_token": format!("rtk-{}", id),
                "expires_in": 3600,
                "expiry_timestamp": now + 3600,
                "project_id": format!("pid-{}", id)
            },
            "quota": {
                "models": [
                    { "name": "gemini-1.5-flash", "percentage": percentage }
                ]
            },
            "disabled": false,
            "proxy_disabled": proxy_disabled,
            "proxy_disabled_reason": if proxy_disabled { "manual" } else { "" },
            "created_at": now,
            "last_used": now
        });
        std::fs::write(&account_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();
    };

    // Two accounts in pool. acc1 has higher quota -> should be selected and bound first.
    write_account("acc1", "a@test.com", 90, false);
    write_account("acc2", "b@test.com", 10, false);

    let manager = TokenManager::new(tmp_root.clone());
    manager.load_accounts().await.unwrap();

    // Prime: first request should bind the session to acc1.
    let (_token, _project_id, _email, account_id, _wait_ms) = manager
        .get_token("gemini", false, Some("sid1"), "gemini-1.5-flash")
        .await
        .unwrap();
    assert_eq!(account_id, "acc1");
    assert_eq!(
        manager.session_accounts.get("sid1").map(|v| v.clone()),
        Some("acc1".to_string())
    );

    // Disable acc1 on disk WITHOUT reloading the in-memory pool (simulates stale cache).
    write_account("acc1", "a@test.com", 90, true);

    let (_token, _project_id, email, account_id, _wait_ms) = manager
        .get_token("gemini", false, Some("sid1"), "gemini-1.5-flash")
        .await
        .unwrap();

    // Should fall back to another account instead of reusing the disabled bound one.
    assert_eq!(account_id, "acc2");
    assert_eq!(email, "b@test.com");
    assert!(manager.tokens.get("acc1").is_none());
    assert_ne!(
        manager.session_accounts.get("sid1").map(|v| v.clone()),
        Some("acc1".to_string())
    );

    // Justification: best-effort cleanup; a leftover directory is harmless
    crate::error::record_ignored(std::fs::remove_dir_all(&tmp_root), "remove_dir_all");
}
