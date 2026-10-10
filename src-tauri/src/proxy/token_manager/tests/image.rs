//! image token selection and queue scheduling tests.

use super::super::helpers::wait_for_image_account_change;
use super::super::helpers::wait_for_image_token_selection;
use super::super::TokenManager;
use super::helpers::*;
use std::time::Duration;

#[tokio::test]
async fn task_image_selection_respects_queue_deadline() {
    let result = wait_for_image_token_selection(
        tokio::time::Instant::now() + std::time::Duration::from_millis(20),
        std::future::pending::<()>(),
    )
    .await;
    assert!(result.is_none());
}

#[tokio::test]
async fn task_image_queue_reselects_without_scheduler_notification() {
    let (_sender, mut changes) = tokio::sync::watch::channel(0);
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(1),
        wait_for_image_account_change(&mut changes, std::time::Duration::from_secs(1)),
    )
    .await;
    assert_eq!(result, Ok(true));
}

#[tokio::test]
async fn task_short_limit_buffer_reselects_without_blocking_runtime() {
    let tmp_root = std::env::temp_dir().join(format!(
        "antigravity-short-limit-test-{}",
        uuid::Uuid::new_v4()
    ));
    let accounts_dir = tmp_root.join("accounts");
    std::fs::create_dir_all(&accounts_dir).unwrap();
    let account_id = "acc-short-limit";
    let model = "gemini-3-flash";
    let now = chrono::Utc::now().timestamp();
    std::fs::write(
        accounts_dir.join(format!("{}.json", account_id)),
        serde_json::to_string_pretty(&serde_json::json!({
            "id": account_id,
            "email": "short-limit@test.com",
            "token": {
                "access_token": "atk",
                "refresh_token": "rtk",
                "expires_in": 3600,
                "expiry_timestamp": now + 3600,
                "project_id": "pid"
            },
            "quota": {
                "models": [{ "name": model, "percentage": 100 }]
            },
            "disabled": false,
            "proxy_disabled": false,
            "created_at": now,
            "last_used": now
        }))
        .unwrap(),
    )
    .unwrap();

    let manager = TokenManager::new(tmp_root.clone());
    manager.load_accounts().await.unwrap();
    manager.rate_limit_tracker.parse_from_error(
        account_id,
        429,
        Some("1"),
        r#"{"error":{"details":[{"reason":"QUOTA_EXHAUSTED"}]}}"#,
        Some(model.to_string()),
        &[60, 300],
    );

    let selected = tokio::time::timeout(
        std::time::Duration::from_secs(4),
        manager.get_token("gemini", false, None, model),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(selected.3, account_id);
    assert!(!manager.is_rate_limited(account_id, Some(model)).await);

    // Justification: best-effort cleanup; a leftover directory is harmless
    crate::error::record_ignored(std::fs::remove_dir_all(&tmp_root), "remove_dir_all");
}

#[tokio::test]
async fn task_concurrent_image_limits_persist_and_clear_exact_bucket() {
    let tmp_root = std::env::temp_dir().join(format!(
        "antigravity-live-limit-test-{}",
        uuid::Uuid::new_v4()
    ));
    let accounts_dir = tmp_root.join("accounts");
    std::fs::create_dir_all(&accounts_dir).unwrap();
    let account_id = "acc-concurrent";
    let account_path = accounts_dir.join(format!("{}.json", account_id));
    std::fs::write(
        &account_path,
        serde_json::to_string_pretty(&serde_json::json!({
            "id": account_id,
            "email": "concurrent@test.com",
            "live_limited_models": {}
        }))
        .unwrap(),
    )
    .unwrap();

    let manager = TokenManager::new(tmp_root.clone());
    let body = r#"{"error":{"details":[{"reason":"QUOTA_EXHAUSTED","metadata":{"quotaResetDelay":"72h"}}]}}"#;
    assert!(
        !manager
            .mark_rate_limited_fast(account_id, 429, None, body, Some("gemini-3.1-flash-image"),)
            .await
    );
    assert!(manager
        .rate_limit_tracker
        .is_rate_limited(account_id, Some("gemini-3.1-flash-image")));
    let persisted: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&account_path).unwrap()).unwrap();
    let flash_limit = &persisted["live_limited_models"]["gemini-3.1-flash-image"];
    assert_eq!(flash_limit["status"], 429);
    assert!(flash_limit["until"].as_i64().unwrap() - chrono::Utc::now().timestamp() > 71 * 3600);

    manager.clear_persisted_live_limit(account_id, Some("gemini-3.1-flash-image-4k"));
    std::thread::scope(|scope| {
        for model in ["gemini-3.1-flash-image", "gemini-3-pro-image"] {
            let manager = &manager;
            scope.spawn(move || {
                manager.record_rate_limit_atomic(
                    account_id,
                    429,
                    None,
                    body,
                    Some(model),
                    &[60, 300],
                    TrackerParserMode::Current,
                );
            });
        }
    });

    let persisted: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&account_path).unwrap()).unwrap();
    let limits = persisted["live_limited_models"].as_object().unwrap();
    assert!(limits.contains_key("gemini-3.1-flash-image"));
    assert!(limits.contains_key("gemini-3-pro-image"));

    manager.clear_persisted_live_limit(account_id, Some("gemini-3.1-flash-image-4k"));
    let persisted: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&account_path).unwrap()).unwrap();
    let limits = persisted["live_limited_models"].as_object().unwrap();
    assert!(!limits.contains_key("gemini-3.1-flash-image"));
    assert!(limits.contains_key("gemini-3-pro-image"));

    std::thread::scope(|scope| {
        scope.spawn(|| {
            manager.record_rate_limit_atomic(
                account_id,
                429,
                None,
                body,
                Some("gemini-3-pro-image"),
                &[60, 300],
                TrackerParserMode::Current,
            );
        });
        scope.spawn(|| {
            manager.clear_persisted_live_limit(account_id, Some("gemini-3-pro-image"));
        });
    });
    let persisted: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&account_path).unwrap()).unwrap();
    let disk_has_limit = persisted["live_limited_models"]
        .as_object()
        .unwrap()
        .contains_key("gemini-3-pro-image");
    assert_eq!(
        disk_has_limit,
        manager
            .rate_limit_tracker
            .is_rate_limited(account_id, Some("gemini-3-pro-image"))
    );

    // Justification: best-effort cleanup; a leftover directory is harmless
    crate::error::record_ignored(std::fs::remove_dir_all(&tmp_root), "remove_dir_all");
}
