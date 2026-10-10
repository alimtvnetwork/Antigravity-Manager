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
    // 同等级下，健康分高的优先
    let high_health = create_test_token("high@test.com", Some("PRO"), 1.0, None, Some(50));
    let low_health = create_test_token("low@test.com", Some("PRO"), 0.5, None, Some(50));

    assert_eq!(compare_tokens(&high_health, &low_health), Ordering::Less);
    assert_eq!(compare_tokens(&low_health, &high_health), Ordering::Greater);
}

#[test]
fn test_sorting_reset_time_priority() {
    let now = chrono::Utc::now().timestamp();

    // 刷新时间更近（30分钟后）的优先于更远（5小时后）的
    let soon_reset = create_test_token(
        "soon@test.com",
        Some("PRO"),
        1.0,
        Some(now + 1800),
        Some(50),
    ); // 30分钟后
    let late_reset = create_test_token(
        "late@test.com",
        Some("PRO"),
        1.0,
        Some(now + 18000),
        Some(50),
    ); // 5小时后

    assert_eq!(compare_tokens(&soon_reset, &late_reset), Ordering::Less);
    assert_eq!(compare_tokens(&late_reset, &soon_reset), Ordering::Greater);
}

#[test]
fn test_sorting_reset_time_threshold() {
    let now = chrono::Utc::now().timestamp();

    // 差异小于10分钟（600秒）视为相同优先级，此时按配额排序
    let reset_a = create_test_token("a@test.com", Some("PRO"), 1.0, Some(now + 1800), Some(80)); // 30分钟后, 80%配额
    let reset_b = create_test_token("b@test.com", Some("PRO"), 1.0, Some(now + 2100), Some(50)); // 35分钟后, 50%配额

    // 差5分钟 < 10分钟阈值，视为相同，按配额排序（80% > 50%）
    assert_eq!(compare_tokens(&reset_a, &reset_b), Ordering::Less);
}

#[test]
fn test_sorting_reset_time_beyond_threshold() {
    let now = chrono::Utc::now().timestamp();

    // 差异超过10分钟，按刷新时间排序（忽略配额）
    let soon_low_quota = create_test_token(
        "soon@test.com",
        Some("PRO"),
        1.0,
        Some(now + 1800),
        Some(20),
    ); // 30分钟后, 20%
    let late_high_quota = create_test_token(
        "late@test.com",
        Some("PRO"),
        1.0,
        Some(now + 18000),
        Some(90),
    ); // 5小时后, 90%

    // 差4.5小时 > 10分钟，刷新时间优先，30分钟 < 5小时
    assert_eq!(
        compare_tokens(&soon_low_quota, &late_high_quota),
        Ordering::Less
    );
}

#[test]
fn test_sorting_quota_fallback() {
    // 其他条件相同时，配额高的优先
    let high_quota = create_test_token("high@test.com", Some("PRO"), 1.0, None, Some(80));
    let low_quota = create_test_token("low@test.com", Some("PRO"), 1.0, None, Some(20));

    assert_eq!(compare_tokens(&high_quota, &low_quota), Ordering::Less);
    assert_eq!(compare_tokens(&low_quota, &high_quota), Ordering::Greater);
}

#[test]
fn test_sorting_missing_reset_time() {
    let now = chrono::Utc::now().timestamp();

    // 没有 reset_time 的账号应该排在有 reset_time 的后面
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
        ), // 30分钟后
        create_test_token(
            "pro_late@test.com",
            Some("PRO"),
            1.0,
            Some(now + 18000),
            Some(90),
        ), // 5小时后
        create_test_token(
            "ultra@test.com",
            Some("ULTRA"),
            1.0,
            Some(now + 36000),
            Some(10),
        ),
    ];

    tokens.sort_by(compare_tokens);

    // 预期顺序:
    // 1. ULTRA (最高等级，即使刷新时间最远)
    // 2. PRO + 高健康分 + 30分钟后刷新
    // 3. PRO + 高健康分 + 5小时后刷新
    // 4. PRO + 低健康分
    // 5. FREE (最低等级，即使配额最高)
    assert_eq!(tokens[0].email, "ultra@test.com");
    assert_eq!(tokens[1].email, "pro_soon@test.com");
    assert_eq!(tokens[2].email, "pro_late@test.com");
    assert_eq!(tokens[3].email, "pro_low_health@test.com");
    assert_eq!(tokens[4].email, "free_high@test.com");
}

#[test]
fn test_realistic_scenario() {
    // 模拟用户描述的场景:
    // a 账号 claude 4h55m 后刷新
    // b 账号 claude 31m 后刷新
    // 应该优先使用 b（31分钟后刷新）
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

    // b 应该排在 a 前面（刷新时间更近）
    assert_eq!(compare_tokens(&account_b, &account_a), Ordering::Less);

    let mut tokens = vec![account_a.clone(), account_b.clone()];
    tokens.sort_by(compare_tokens);

    assert_eq!(tokens[0].email, "b@test.com");
    assert_eq!(tokens[1].email, "a@test.com");
}

#[test]
fn test_extract_earliest_reset_time() {
    let manager = TokenManager::new(PathBuf::from("/tmp/test"));

    // 测试包含 claude 模型的 reset_time 提取
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
    // 应该返回 claude 的时间（08:00）而不是 gemini 的（10:00）
    let expected_ts = chrono::DateTime::parse_from_rfc3339("2025-01-31T08:00:00Z")
        .unwrap()
        .timestamp();
    assert_eq!(result.unwrap(), expected_ts);
}

#[test]
fn test_extract_reset_time_no_claude() {
    let manager = TokenManager::new(PathBuf::from("/tmp/test"));

    // 没有 claude 模型时，应该取任意模型的最近时间
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

    // 没有 quota 字段时应返回 None
    let account_no_quota = serde_json::json!({
        "email": "test@test.com"
    });

    assert!(manager
        .extract_earliest_reset_time(&account_no_quota)
        .is_none());
}

// ===== P2C 算法测试 =====
