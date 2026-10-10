#![cfg(test)]

use super::*;
use std::collections::HashSet;
use std::sync::Mutex as StdMutex;

use super::*;

#[test]
pub(crate) fn task_quota_refresh_keeps_unexpired_live_limit() {
    let _env_guard = TEST_DATA_DIR_MUTEX
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let _guard = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let dir = TestDataDir::new();
    let account_id = "live-limit-account";
    create_account_file(dir.path(), account_id, "live-limit@example.com");
    let previous = std::env::var_os("ABV_DATA_DIR");
    std::env::set_var("ABV_DATA_DIR", dir.path());

    let now = chrono::Utc::now().timestamp();
    let mut account = load_account(account_id).unwrap();
    account.live_limited_models.insert(
            "gemini-3-pro-image".to_string(),
            crate::models::account::LiveLimitStatus {
                model: "gemini-3-pro-image".to_string(),
                status: 429,
                reason: "QuotaExhausted".to_string(),
                until: now + 7200,
                detected_at: now,
                message: Some(
                    r#"{"error":{"details":[{"reason":"QUOTA_EXHAUSTED","metadata":{"quotaResetDelay":"2h"}}]}}"#
                        .to_string(),
                ),
            },
        );
    account.live_limited_models.insert(
        "gemini-3.1-flash-image".to_string(),
        crate::models::account::LiveLimitStatus {
            model: "gemini-3.1-flash-image".to_string(),
            status: 429,
            reason: "QuotaExhausted".to_string(),
            until: now + 7200,
            detected_at: now,
            message: Some("QUOTA_EXHAUSTED".to_string()),
        },
    );
    account.live_limited_models.insert(
        "gemini-2.5-pro".to_string(),
        crate::models::account::LiveLimitStatus {
            model: "gemini-2.5-pro".to_string(),
            status: 429,
            reason: "QuotaExhausted".to_string(),
            until: now + 7200,
            detected_at: now,
            message: Some("QUOTA_EXHAUSTED; reset after 2h".to_string()),
        },
    );
    save_account(&account).unwrap();

    let quota: QuotaData = serde_json::from_value(serde_json::json!({
        "models": [
            {"name": "gemini-3-pro-image", "percentage": 99, "reset_time": ""},
            {"name": "gemini-3.1-flash-image", "percentage": 99, "reset_time": ""},
            {"name": "gemini-2.5-pro", "percentage": 99, "reset_time": ""}
        ],
        "last_updated": now
    }))
    .unwrap();
    update_account_quota(account_id, quota).unwrap();

    let updated = load_account(account_id).unwrap();
    assert!(updated
        .live_limited_models
        .contains_key("gemini-3-pro-image"));
    assert!(!updated
        .live_limited_models
        .contains_key("gemini-3.1-flash-image"));
    assert!(!updated.live_limited_models.contains_key("gemini-2.5-pro"));
    if let Some(previous) = previous {
        std::env::set_var("ABV_DATA_DIR", previous);
    } else {
        std::env::remove_var("ABV_DATA_DIR");
    }
}

#[test]
pub(crate) fn test_is_tier_missing_or_unknown() {
    let mut acc = Account::new(
        "test_id".to_string(),
        "user@example.com".to_string(),
        TokenData::new(
            "access".to_string(),
            "refresh".to_string(),
            3600,
            Some("user@example.com".to_string()),
            None,
            None,
            true,
            None,
        ),
    );
    // Quota is None -> missing
    assert!(is_tier_missing_or_unknown(&acc));

    let mut q = QuotaData::new();
    acc.quota = Some(q.clone());
    // subscription_tier is None -> missing
    assert!(is_tier_missing_or_unknown(&acc));

    q.subscription_tier = Some("unknown".to_string());
    acc.quota = Some(q.clone());
    assert!(is_tier_missing_or_unknown(&acc));

    q.subscription_tier = Some("FREE".to_string());
    acc.quota = Some(q.clone());
    assert!(!is_tier_missing_or_unknown(&acc));

    q.subscription_tier = Some("Google AI Pro".to_string());
    acc.quota = Some(q.clone());
    assert!(!is_tier_missing_or_unknown(&acc));

    q.subscription_tier = Some("g1-ultra-tier".to_string());
    acc.quota = Some(q.clone());
    assert!(!is_tier_missing_or_unknown(&acc));
}
