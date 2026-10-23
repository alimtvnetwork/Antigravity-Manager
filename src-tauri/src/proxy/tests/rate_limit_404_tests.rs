//! Test RateLimitTracker's processing benchmark for 404 / 500 / 503 and 429:
//! - 404 (model does not exist) and 500/503 (server-side exception) will never be entered into the account cooling pool (return None)
//! - The real 429 (current limit) is accurately entered into the cooling pool, and model-level isolation is strictly carried out when providing models, and it is forbidden to use all accounts together.

use crate::proxy::rate_limit::RateLimitTracker;

#[test]
fn test_404_and_5xx_never_lock_account() {
    let tracker = RateLimitTracker::new();
    let backoff_steps = vec![60, 300, 1800, 7200];

    // 404 Model does not exist: It is absolutely forbidden to enter the cooling pool
    let info_404 = tracker.parse_from_error(
        "acc_test",
        404,
        None,
        "models/nonexistent is not found",
        Some("nonexistent".to_string()),
        &backoff_steps,
    );
    assert!(
        info_404.is_none(),
        "404 must return None (no account lockout)"
    );
    assert!(!tracker.is_rate_limited("acc_test", Some("nonexistent")));
    assert!(!tracker.is_rate_limited("acc_test", None));

    // 500 Server internal error: service failure, never used as account flow limit
    let info_500 = tracker.parse_from_error(
        "acc_test",
        500,
        None,
        "Internal server error",
        Some("gemini-2.5-pro".to_string()),
        &backoff_steps,
    );
    assert!(info_500.is_none(), "500 must return None");
    assert!(!tracker.is_rate_limited("acc_test", Some("gemini-2.5-pro")));

    // 503 Service Unavailable: Momentary failure, never used as account current limit
    let info_503 = tracker.parse_from_error(
        "acc_test",
        503,
        None,
        "Service Unavailable",
        Some("gemini-2.5-pro".to_string()),
        &backoff_steps,
    );
    assert!(info_503.is_none(), "503 must return None");
    assert!(!tracker.is_rate_limited("acc_test", Some("gemini-2.5-pro")));
}

#[test]
fn test_rate_limit_with_model_isolates_fine_grained() {
    let tracker = RateLimitTracker::new();
    let backoff_steps = vec![60, 300, 1800, 7200];

    // 429 rate limit trigger (with specific model)
    let info_429 = tracker.parse_from_error(
        "acc_isolated",
        429,
        None,
        r#"{"error":{"message":"Resource has been exhausted (e.g. check quota)."}}"#,
        Some("claude-test-model".to_string()),
        &backoff_steps,
    );
    assert!(info_429.is_some(), "429 should return Some");

    // This account is currently restricted for claude-test-model
    assert!(tracker.is_rate_limited("acc_isolated", Some("claude-test-model")));

    // Key verification: This account will definitely not be linked to other normal models (such as gemini-2.5-flash / claude-sonnet-4-6)!
    assert!(
        !tracker.is_rate_limited("acc_isolated", Some("gemini-2.5-flash")),
        "Other models must NOT be locked when rate limit specifies a model"
    );
    assert!(
        !tracker.is_rate_limited("acc_isolated", Some("claude-sonnet-4-6")),
        "Healthy models must remain available"
    );
}
