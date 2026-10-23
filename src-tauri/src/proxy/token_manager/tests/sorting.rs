//! token sorting (tier/health/reset-time/quota) tests.

use super::super::TokenManager;
use super::helpers::*;
use std::cmp::Ordering;

#[test]
fn test_sorting_tier_priority() {
    // ULTRA > PRO > FREE
    let ultra = create_test_token("ultra@test.com", Some("ULTRA"), 1.0, None, Some(50));
    let pro = create_test_token("pro@test.com", Some("PRO"), 1.0, None, Some(50));
    let premium = create_test_token(
        "premium@test.com",
        Some("Google One AI Premium"),
        1.0,
        None,
        Some(50),
    );
    let advanced = create_test_token(
        "advanced@test.com",
        Some("Gemini Advanced"),
        1.0,
        None,
        Some(50),
    );
    let free = create_test_token("free@test.com", Some("FREE"), 1.0, None, Some(50));

    assert_eq!(compare_tokens(&ultra, &pro), Ordering::Less);
    assert_eq!(compare_tokens(&ultra, &premium), Ordering::Less);
    assert_eq!(compare_tokens(&pro, &free), Ordering::Less);
    assert_eq!(compare_tokens(&premium, &free), Ordering::Less);
    assert_eq!(compare_tokens(&advanced, &free), Ordering::Less);
    assert_eq!(compare_tokens(&ultra, &free), Ordering::Less);
    assert_eq!(compare_tokens(&free, &ultra), Ordering::Greater);
}

#[test]
fn test_sorting_health_score_priority() {
    // At the same level, those with higher health scores will be given priority.
    let high_health = create_test_token("high@test.com", Some("PRO"), 1.0, None, Some(50));
    let low_health = create_test_token("low@test.com", Some("PRO"), 0.5, None, Some(50));

    assert_eq!(compare_tokens(&high_health, &low_health), Ordering::Less);
    assert_eq!(compare_tokens(&low_health, &high_health), Ordering::Greater);
}

#[test]
fn test_sorting_reset_time_priority() {
    let now = chrono::Utc::now().timestamp();

    // The refresh time is closer (30 minutes later) than the refresh time is farther (5 hours later)
    let soon_reset = create_test_token(
        "soon@test.com",
        Some("PRO"),
        1.0,
        Some(now + 1800),
        Some(50),
    ); // 30 minutes later
    let late_reset = create_test_token(
        "late@test.com",
        Some("PRO"),
        1.0,
        Some(now + 18000),
        Some(50),
    ); // 5 hours later

    assert_eq!(compare_tokens(&soon_reset, &late_reset), Ordering::Less);
    assert_eq!(compare_tokens(&late_reset, &soon_reset), Ordering::Greater);
}

#[test]
fn test_sorting_reset_time_threshold() {
    let now = chrono::Utc::now().timestamp();

    // If the difference is less than 10 minutes (600 seconds), it will be regarded as the same priority, and it will be sorted by quota.
    let reset_a = create_test_token("a@test.com", Some("PRO"), 1.0, Some(now + 1800), Some(80)); // After 30 minutes, 80% quota
    let reset_b = create_test_token("b@test.com", Some("PRO"), 1.0, Some(now + 2100), Some(50)); // After 35 minutes, 50% quota

    // If the difference is 5 minutes < 10 minutes threshold, it is considered to be the same and sorted by quota (80% > 50%)
    assert_eq!(compare_tokens(&reset_a, &reset_b), Ordering::Less);
}

#[test]
fn test_sorting_reset_time_beyond_threshold() {
    let now = chrono::Utc::now().timestamp();

    // Differences greater than 10 minutes, sorted by refresh time (ignoring quota)
    let soon_low_quota = create_test_token(
        "soon@test.com",
        Some("PRO"),
        1.0,
        Some(now + 1800),
        Some(20),
    ); // After 30 minutes, 20%
    let late_high_quota = create_test_token(
        "late@test.com",
        Some("PRO"),
        1.0,
        Some(now + 18000),
        Some(90),
    ); // After 5 hours, 90%

    // Difference 4.5 hours > 10 minutes, refresh time takes priority, 30 minutes < 5 hours
    assert_eq!(
        compare_tokens(&soon_low_quota, &late_high_quota),
        Ordering::Less
    );
}

#[test]
fn test_sorting_quota_fallback() {
    // When other conditions are equal, the one with the highest quota will be given priority.
    let high_quota = create_test_token("high@test.com", Some("PRO"), 1.0, None, Some(80));
    let low_quota = create_test_token("low@test.com", Some("PRO"), 1.0, None, Some(20));

    assert_eq!(compare_tokens(&high_quota, &low_quota), Ordering::Less);
    assert_eq!(compare_tokens(&low_quota, &high_quota), Ordering::Greater);
}

#[test]
fn test_sorting_missing_reset_time() {
    let now = chrono::Utc::now().timestamp();

    // Accounts without reset_time should be ranked behind those with reset_time
    let with_reset = create_test_token(
        "with@test.com",
        Some("PRO"),
        1.0,
        Some(now + 1800),
        Some(50),
    );
    let without_reset = create_test_token("without@test.com", Some("PRO"), 1.0, None, Some(50));

    assert_eq!(compare_tokens(&with_reset, &without_reset), Ordering::Less);
}

#[test]
fn test_full_sorting_integration() {
    let now = chrono::Utc::now().timestamp();

    let mut tokens = vec![
        create_test_token(
            "free_high@test.com",
            Some("FREE"),
            1.0,
            Some(now + 1800),
            Some(90),
        ),
        create_test_token(
            "pro_low_health@test.com",
            Some("PRO"),
            0.5,
            Some(now + 1800),
            Some(90),
        ),
        create_test_token(
            "pro_soon@test.com",
            Some("PRO"),
            1.0,
            Some(now + 1800),
            Some(50),
        ), // 30 minutes later
        create_test_token(
            "pro_late@test.com",
            Some("PRO"),
            1.0,
            Some(now + 18000),
            Some(90),
        ), // 5 hours later
        create_test_token(
            "ultra@test.com",
            Some("ULTRA"),
            1.0,
            Some(now + 36000),
            Some(10),
        ),
    ];

    tokens.sort_by(compare_tokens);

    // Expected order:
    // 1. ULTRA (the highest level, even if the refresh time is the furthest)
    // 2. PRO + high health score + refresh after 30 minutes
    // 3. PRO + high health score + refresh after 5 hours
    // 4. PRO + low health score
    // 5. FREE (lowest level, even with the highest quota)
    assert_eq!(tokens[0].email, "ultra@test.com");
    assert_eq!(tokens[1].email, "pro_soon@test.com");
    assert_eq!(tokens[2].email, "pro_late@test.com");
    assert_eq!(tokens[3].email, "pro_low_health@test.com");
    assert_eq!(tokens[4].email, "free_high@test.com");
}

#[test]
fn test_realistic_scenario() {
    // Simulate the scenario described by the user:
    // a account claude refreshed after 4h55m
    // b Account claude Refresh after 31m
    // b should be used first (refreshed after 31 minutes)
    let now = chrono::Utc::now().timestamp();

    let account_a = create_test_token(
        "a@test.com",
        Some("PRO"),
        1.0,
        Some(now + 295 * 60),
        Some(80),
    ); // 4h55m
    let account_b = create_test_token(
        "b@test.com",
        Some("PRO"),
        1.0,
        Some(now + 31 * 60),
        Some(30),
    ); // 31m

    // b should be ranked before a (refresh time is closer)
    assert_eq!(compare_tokens(&account_b, &account_a), Ordering::Less);

    let mut tokens = vec![account_a.clone(), account_b.clone()];
    tokens.sort_by(compare_tokens);

    assert_eq!(tokens[0].email, "b@test.com");
    assert_eq!(tokens[1].email, "a@test.com");
}

#[test]
fn test_extract_earliest_reset_time() {
    let manager = TokenManager::new(PathBuf::from("/tmp/test"));

    // Test contains reset_time extraction of claude model
    let account_with_claude = serde_json::json!({
        "quota": {
            "models": [
                {"name": "gemini-flash", "reset_time": "2025-01-31T10:00:00Z"},
                {"name": "claude-sonnet", "reset_time": "2025-01-31T08:00:00Z"},
                {"name": "claude-opus", "reset_time": "2025-01-31T08:00:00Z"}
            ]
        }
    });

    let result = manager.extract_earliest_reset_time(&account_with_claude);
    assert!(result.is_some());
    // should return claude's time (08:00) instead of gemini's (10:00)
    let expected_ts = chrono::DateTime::parse_from_rfc3339("2025-01-31T08:00:00Z")
        .unwrap()
        .timestamp();
    assert_eq!(result.unwrap(), expected_ts);
}

#[test]
fn test_extract_reset_time_no_claude() {
    let manager = TokenManager::new(PathBuf::from("/tmp/test"));

    // When there is no claude model, the latest time of any model should be taken
    let account_no_claude = serde_json::json!({
        "quota": {
            "models": [
                {"name": "gemini-flash", "reset_time": "2025-01-31T10:00:00Z"},
                {"name": "gemini-pro", "reset_time": "2025-01-31T08:00:00Z"}
            ]
        }
    });

    let result = manager.extract_earliest_reset_time(&account_no_claude);
    assert!(result.is_some());
    let expected_ts = chrono::DateTime::parse_from_rfc3339("2025-01-31T08:00:00Z")
        .unwrap()
        .timestamp();
    assert_eq!(result.unwrap(), expected_ts);
}

#[test]
fn test_extract_reset_time_missing_quota() {
    let manager = TokenManager::new(PathBuf::from("/tmp/test"));

    // Should return None when there is no quota field
    let account_no_quota = serde_json::json!({
        "email": "test@test.com"
    });

    assert!(manager
        .extract_earliest_reset_time(&account_no_quota)
        .is_none());
}

// ===== P2C Algorithm Test =====
