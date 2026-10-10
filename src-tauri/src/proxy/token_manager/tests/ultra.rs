//! ULTRA model priority and circuit breaker recovery tests.

use super::super::ProxyToken;
use super::super::TokenManager;
use super::helpers::*;
use std::cmp::Ordering;
use std::time::Duration;

/// 测试 is_ultra_required_model 辅助函数
#[test]
fn test_is_ultra_required_model() {
    // 需要 Ultra 账号的高端模型
    const ULTRA_REQUIRED_MODELS: &[&str] = &["claude-opus-4-6", "claude-opus-4-5", "opus"];

    fn is_ultra_required_model(model: &str) -> bool {
        let lower = model.to_lowercase();
        ULTRA_REQUIRED_MODELS.iter().any(|m| lower.contains(m))
    }

    // 应该识别为高端模型
    assert!(is_ultra_required_model("claude-opus-4-6"));
    assert!(is_ultra_required_model("claude-opus-4-5"));
    assert!(is_ultra_required_model("Claude-Opus-4-6")); // 大小写不敏感
    assert!(is_ultra_required_model("CLAUDE-OPUS-4-5")); // 大小写不敏感
    assert!(is_ultra_required_model("opus")); // 通配匹配
    assert!(is_ultra_required_model("opus-4-6-latest"));
    assert!(is_ultra_required_model("models/claude-opus-4-6"));

    // 应该识别为普通模型
    assert!(!is_ultra_required_model("claude-sonnet-4-5"));
    assert!(!is_ultra_required_model("claude-sonnet"));
    assert!(!is_ultra_required_model("gemini-1.5-flash"));
    assert!(!is_ultra_required_model("gemini-2.0-pro"));
    assert!(!is_ultra_required_model("claude-haiku"));
}

/// 测试高端模型排序：Ultra 账号优先于 Pro 账号（即使 Pro 配额更高）
#[test]
fn test_ultra_priority_for_high_end_models() {
    const RESET_TIME_THRESHOLD_SECS: i64 = 600;

    // 模拟高端模型排序逻辑
    fn compare_tokens_for_model(a: &ProxyToken, b: &ProxyToken, target_model: &str) -> Ordering {
        const ULTRA_REQUIRED_MODELS: &[&str] = &["claude-opus-4-6", "claude-opus-4-5", "opus"];
        let requires_ultra = {
            let lower = target_model.to_lowercase();
            ULTRA_REQUIRED_MODELS.iter().any(|m| lower.contains(m))
        };

        // 直接复用生产实现，避免测试里另写一份「简化版关键词表」
        // （旧版只匹配 "pro"，漏掉 premium/advanced，导致测试通过但生产行为未经验证）
        let tier_priority =
            |tier: &Option<String>| crate::models::quota::tier_priority(tier.as_deref());

        // Priority 0: 高端模型时，订阅等级优先
        if requires_ultra {
            let tier_cmp =
                tier_priority(&a.subscription_tier).cmp(&tier_priority(&b.subscription_tier));
            if tier_cmp != Ordering::Equal {
                return tier_cmp;
            }
        }

        // Priority 1: Quota (higher is better)
        let quota_a = a.remaining_quota.unwrap_or(0);
        let quota_b = b.remaining_quota.unwrap_or(0);
        let quota_cmp = quota_b.cmp(&quota_a);
        if quota_cmp != Ordering::Equal {
            return quota_cmp;
        }

        // Priority 2: Health score
        let health_cmp = b
            .health_score
            .partial_cmp(&a.health_score)
            .unwrap_or(Ordering::Equal);
        if health_cmp != Ordering::Equal {
            return health_cmp;
        }

        // Priority 3: Tier (for non-high-end models)
        if !requires_ultra {
            let tier_cmp =
                tier_priority(&a.subscription_tier).cmp(&tier_priority(&b.subscription_tier));
            if tier_cmp != Ordering::Equal {
                return tier_cmp;
            }
        }

        Ordering::Equal
    }

    // 创建测试账号：Ultra 低配额 vs Pro 高配额
    let ultra_low_quota = create_test_token("ultra@test.com", Some("ULTRA"), 1.0, None, Some(20));
    let pro_high_quota = create_test_token("pro@test.com", Some("PRO"), 1.0, None, Some(80));

    // 高端模型 (Opus 4.6): Ultra 应该优先，即使配额低
    assert_eq!(
        compare_tokens_for_model(&ultra_low_quota, &pro_high_quota, "claude-opus-4-6"),
        Ordering::Less, // Ultra 排在前面
        "Opus 4.6 should prefer Ultra account over Pro even with lower quota"
    );

    // 高端模型 (Opus 4.5): Ultra 应该优先
    assert_eq!(
        compare_tokens_for_model(&ultra_low_quota, &pro_high_quota, "claude-opus-4-5"),
        Ordering::Less,
        "Opus 4.5 should prefer Ultra account over Pro"
    );

    // 普通模型 (Sonnet): 高配额 Pro 应该优先
    assert_eq!(
        compare_tokens_for_model(&ultra_low_quota, &pro_high_quota, "claude-sonnet-4-5"),
        Ordering::Greater, // Pro (高配额) 排在前面
        "Sonnet should prefer high-quota Pro over low-quota Ultra"
    );

    // 普通模型 (Flash): 高配额 Pro 应该优先
    assert_eq!(
        compare_tokens_for_model(&ultra_low_quota, &pro_high_quota, "gemini-1.5-flash"),
        Ordering::Greater,
        "Flash should prefer high-quota Pro over low-quota Ultra"
    );
}

/// 测试排序：同为 Ultra 时按配额排序
#[test]
fn test_ultra_accounts_sorted_by_quota() {
    fn compare_tokens_for_model(a: &ProxyToken, b: &ProxyToken, target_model: &str) -> Ordering {
        const ULTRA_REQUIRED_MODELS: &[&str] = &["claude-opus-4-6", "claude-opus-4-5", "opus"];
        let requires_ultra = {
            let lower = target_model.to_lowercase();
            ULTRA_REQUIRED_MODELS.iter().any(|m| lower.contains(m))
        };

        // 直接复用生产实现，避免测试里另写一份「简化版关键词表」
        // （旧版只匹配 "pro"，漏掉 premium/advanced，导致测试通过但生产行为未经验证）
        let tier_priority =
            |tier: &Option<String>| crate::models::quota::tier_priority(tier.as_deref());

        if requires_ultra {
            let tier_cmp =
                tier_priority(&a.subscription_tier).cmp(&tier_priority(&b.subscription_tier));
            if tier_cmp != Ordering::Equal {
                return tier_cmp;
            }
        }

        let quota_a = a.remaining_quota.unwrap_or(0);
        let quota_b = b.remaining_quota.unwrap_or(0);
        quota_b.cmp(&quota_a)
    }

    let ultra_high = create_test_token("ultra_high@test.com", Some("ULTRA"), 1.0, None, Some(80));
    let ultra_low = create_test_token("ultra_low@test.com", Some("ULTRA"), 1.0, None, Some(20));

    // Opus 4.6: 同为 Ultra，高配额优先
    assert_eq!(
        compare_tokens_for_model(&ultra_high, &ultra_low, "claude-opus-4-6"),
        Ordering::Less, // ultra_high 排在前面
        "Among Ultra accounts, higher quota should come first"
    );
}

/// 测试完整排序场景：混合账号池
#[test]
fn test_full_sorting_mixed_accounts() {
    fn sort_tokens_for_model(tokens: &mut Vec<ProxyToken>, target_model: &str) {
        const ULTRA_REQUIRED_MODELS: &[&str] = &["claude-opus-4-6", "claude-opus-4-5", "opus"];
        let requires_ultra = {
            let lower = target_model.to_lowercase();
            ULTRA_REQUIRED_MODELS.iter().any(|m| lower.contains(m))
        };

        tokens.sort_by(|a, b| {
            // 直接复用生产实现
            let tier_priority =
                |tier: &Option<String>| crate::models::quota::tier_priority(tier.as_deref());

            if requires_ultra {
                let tier_cmp =
                    tier_priority(&a.subscription_tier).cmp(&tier_priority(&b.subscription_tier));
                if tier_cmp != Ordering::Equal {
                    return tier_cmp;
                }
            }

            let quota_a = a.remaining_quota.unwrap_or(0);
            let quota_b = b.remaining_quota.unwrap_or(0);
            let quota_cmp = quota_b.cmp(&quota_a);
            if quota_cmp != Ordering::Equal {
                return quota_cmp;
            }

            if !requires_ultra {
                let tier_cmp =
                    tier_priority(&a.subscription_tier).cmp(&tier_priority(&b.subscription_tier));
                if tier_cmp != Ordering::Equal {
                    return tier_cmp;
                }
            }

            Ordering::Equal
        });
    }

    // 创建混合账号池
    let ultra_high = create_test_token("ultra_high@test.com", Some("ULTRA"), 1.0, None, Some(80));
    let ultra_low = create_test_token("ultra_low@test.com", Some("ULTRA"), 1.0, None, Some(20));
    let pro_high = create_test_token("pro_high@test.com", Some("PRO"), 1.0, None, Some(90));
    let pro_low = create_test_token("pro_low@test.com", Some("PRO"), 1.0, None, Some(30));
    let free = create_test_token("free@test.com", Some("FREE"), 1.0, None, Some(100));

    // 高端模型 (Opus 4.6) 排序
    let mut tokens_opus = vec![
        pro_high.clone(),
        free.clone(),
        ultra_low.clone(),
        pro_low.clone(),
        ultra_high.clone(),
    ];
    sort_tokens_for_model(&mut tokens_opus, "claude-opus-4-6");

    let emails_opus: Vec<&str> = tokens_opus.iter().map(|t| t.email.as_str()).collect();
    // 期望顺序: Ultra(高配额) > Ultra(低配额) > Pro(高配额) > Pro(低配额) > Free
    assert_eq!(
        emails_opus,
        vec![
            "ultra_high@test.com",
            "ultra_low@test.com",
            "pro_high@test.com",
            "pro_low@test.com",
            "free@test.com"
        ],
        "Opus 4.6 should sort Ultra first, then by quota within each tier"
    );

    // 普通模型 (Sonnet) 排序
    let mut tokens_sonnet = vec![
        pro_high.clone(),
        free.clone(),
        ultra_low.clone(),
        pro_low.clone(),
        ultra_high.clone(),
    ];
    sort_tokens_for_model(&mut tokens_sonnet, "claude-sonnet-4-5");

    let emails_sonnet: Vec<&str> = tokens_sonnet.iter().map(|t| t.email.as_str()).collect();
    // 期望顺序: Free(100%) > Pro(90%) > Ultra(80%) > Pro(30%) > Ultra(20%) - 按配额优先
    assert_eq!(
        emails_sonnet,
        vec![
            "free@test.com",
            "pro_high@test.com",
            "ultra_high@test.com",
            "pro_low@test.com",
            "ultra_low@test.com"
        ],
        "Sonnet should sort by quota first, then by tier as tiebreaker"
    );
}

#[test]
fn test_sync_zero_quota_circuit_breaker_later_deadline_and_recovery() {
    let manager = TokenManager::new(PathBuf::from("/tmp/test"));

    // 1. 周配额为 0，5H 配额为 0，两者均耗尽
    // 周配额 reset_time 为 5天后，5H reset_time 为 2小时后
    let now = chrono::Utc::now();
    let base_timestamp = now.timestamp();
    let reset_5h = (now + chrono::Duration::hours(2)).to_rfc3339();
    let reset_weekly = (now + chrono::Duration::days(5)).to_rfc3339();

    let account = serde_json::json!({
        "quota": {
            "last_updated": base_timestamp,
            "quota_groups": [
                {
                    "display_name": "Claude & 3P Models",
                    "buckets": [
                        {
                            "bucket_id": "3p-5h",
                            "window": "5h",
                            "remaining_fraction": 0.0,
                            "reset_time": reset_5h
                        },
                        {
                            "bucket_id": "3p-weekly",
                            "window": "7d",
                            "remaining_fraction": 0.0,
                            "reset_time": reset_weekly
                        }
                    ]
                }
            ]
        }
    });

    // 即使 lock_on_zero 为 false，周配额耗尽依然无条件锁定至周截止时间
    manager.sync_zero_quota_circuit_breaker("acc1", &account);
    assert!(manager
        .rate_limit_tracker
        .is_rate_limited("acc1", Some("claude-sonnet-4-6")));
    let wait = manager
        .rate_limit_tracker
        .get_remaining_wait("acc1", Some("claude-sonnet-4-6"));
    assert!(
        wait > 4 * 86400,
        "Should be locked for > 4 days due to weekly constraint"
    );

    // 2. 模拟 provider 提前重置：周配额恢复为 100%，5H 配额仍为 0
    // 若开启 lock_on_zero，应自动对齐到较短的 5H 截止时间 (2小时)
    {
        let mut cfg = manager.circuit_breaker_config.blocking_write();
        cfg.enabled = true;
        cfg.lock_on_zero_quota = true;
    }

    let account_recovered_weekly = serde_json::json!({
        "quota": {
            "last_updated": base_timestamp + 1,
            "quota_groups": [
                {
                    "display_name": "Claude & 3P Models",
                    "buckets": [
                        {
                            "bucket_id": "3p-5h",
                            "window": "5h",
                            "remaining_fraction": 0.0,
                            "reset_time": reset_5h
                        },
                        {
                            "bucket_id": "3p-weekly",
                            "window": "7d",
                            "remaining_fraction": 1.0,
                            "reset_time": reset_weekly
                        }
                    ]
                }
            ]
        }
    });

    manager.sync_zero_quota_circuit_breaker("acc1", &account_recovered_weekly);
    let wait_5h = manager
        .rate_limit_tracker
        .get_remaining_wait("acc1", Some("claude-sonnet-4-6"));
    assert!(
        wait_5h <= 2 * 3600 && wait_5h > 0,
        "Should reconcile to 5h deadline (<= 2h)"
    );

    // 3. 模拟 5H 也完全恢复 (全部配额为正)
    let account_fully_recovered = serde_json::json!({
        "quota": {
            "last_updated": base_timestamp + 2,
            "quota_groups": [
                {
                    "display_name": "Claude & 3P Models",
                    "buckets": [
                        {
                            "bucket_id": "3p-5h",
                            "window": "5h",
                            "remaining_fraction": 1.0,
                            "reset_time": reset_5h
                        },
                        {
                            "bucket_id": "3p-weekly",
                            "window": "7d",
                            "remaining_fraction": 1.0,
                            "reset_time": reset_weekly
                        }
                    ]
                }
            ]
        }
    });

    manager.sync_zero_quota_circuit_breaker("acc1", &account_fully_recovered);
    assert!(!manager
        .rate_limit_tracker
        .is_rate_limited("acc1", Some("claude-sonnet-4-6")));
}
