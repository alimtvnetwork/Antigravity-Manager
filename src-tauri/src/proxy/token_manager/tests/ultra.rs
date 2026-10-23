//! ULTRA model priority and circuit breaker recovery tests.

use super::super::ProxyToken;
use super::super::TokenManager;
use super::helpers::*;
use std::cmp::Ordering;
use std::time::Duration;

/// Test the is_ultra_required_model helper function
#[test]
fn test_is_ultra_required_model() {
    // High-end models requiring Ultra account
    const ULTRA_REQUIRED_MODELS: &[&str] = &["claude-opus-4-6", "claude-opus-4-5", "opus"];

    fn is_ultra_required_model(model: &str) -> bool {
        let lower = model.to_lowercase();
        ULTRA_REQUIRED_MODELS.iter().any(|m| lower.contains(m))
    }

    // Should be identified as a high-end model
    assert!(is_ultra_required_model("claude-opus-4-6"));
    assert!(is_ultra_required_model("claude-opus-4-5"));
    assert!(is_ultra_required_model("Claude-Opus-4-6")); // Case insensitive
    assert!(is_ultra_required_model("CLAUDE-OPUS-4-5")); // Case insensitive
    assert!(is_ultra_required_model("opus")); // Wildcard match
    assert!(is_ultra_required_model("opus-4-6-latest"));
    assert!(is_ultra_required_model("models/claude-opus-4-6"));

    // Should be recognized as a normal model
    assert!(!is_ultra_required_model("claude-sonnet-4-5"));
    assert!(!is_ultra_required_model("claude-sonnet"));
    assert!(!is_ultra_required_model("gemini-1.5-flash"));
    assert!(!is_ultra_required_model("gemini-2.0-pro"));
    assert!(!is_ultra_required_model("claude-haiku"));
}

/// Test high-end model sorting: Ultra accounts take precedence over Pro accounts (even if Pro has a higher quota)
#[test]
fn test_ultra_priority_for_high_end_models() {
    const RESET_TIME_THRESHOLD_SECS: i64 = 600;

    // Simulate high-end model sorting logic
    fn compare_tokens_for_model(a: &ProxyToken, b: &ProxyToken, target_model: &str) -> Ordering {
        const ULTRA_REQUIRED_MODELS: &[&str] = &["claude-opus-4-6", "claude-opus-4-5", "opus"];
        let requires_ultra = {
            let lower = target_model.to_lowercase();
            ULTRA_REQUIRED_MODELS.iter().any(|m| lower.contains(m))
        };

        // Directly reuse the production implementation to avoid writing another "simplified version of the keyword list" in the test
        // (The old version only matches "pro" and misses premium/advanced, causing the test to pass but the production behavior is unverified)
        let tier_priority =
            |tier: &Option<String>| crate::models::quota::tier_priority(tier.as_deref());

        // Priority 0: For high-end models, subscription level takes priority
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

    // Create a test account: Ultra low quota vs Pro high quota
    let ultra_low_quota = create_test_token("ultra@test.com", Some("ULTRA"), 1.0, None, Some(20));
    let pro_high_quota = create_test_token("pro@test.com", Some("PRO"), 1.0, None, Some(80));

    // High-end models (Opus 4.6): Ultra should take precedence even if quota is low
    assert_eq!(
        compare_tokens_for_model(&ultra_low_quota, &pro_high_quota, "claude-opus-4-6"),
        Ordering::Less, // Ultra comes first
        "Opus 4.6 should prefer Ultra account over Pro even with lower quota"
    );

    // High-end models (Opus 4.5): Ultra should be preferred
    assert_eq!(
        compare_tokens_for_model(&ultra_low_quota, &pro_high_quota, "claude-opus-4-5"),
        Ordering::Less,
        "Opus 4.5 should prefer Ultra account over Pro"
    );

    // Normal model (Sonnet): High quota Pro should be prioritized
    assert_eq!(
        compare_tokens_for_model(&ultra_low_quota, &pro_high_quota, "claude-sonnet-4-5"),
        Ordering::Greater, // Pro (high quota) ranked first
        "Sonnet should prefer high-quota Pro over low-quota Ultra"
    );

    // Normal model (Flash): High quota Pro should be given priority
    assert_eq!(
        compare_tokens_for_model(&ultra_low_quota, &pro_high_quota, "gemini-1.5-flash"),
        Ordering::Greater,
        "Flash should prefer high-quota Pro over low-quota Ultra"
    );
}

/// Test sorting: Sort by quota when both are Ultra
#[test]
fn test_ultra_accounts_sorted_by_quota() {
    fn compare_tokens_for_model(a: &ProxyToken, b: &ProxyToken, target_model: &str) -> Ordering {
        const ULTRA_REQUIRED_MODELS: &[&str] = &["claude-opus-4-6", "claude-opus-4-5", "opus"];
        let requires_ultra = {
            let lower = target_model.to_lowercase();
            ULTRA_REQUIRED_MODELS.iter().any(|m| lower.contains(m))
        };

        // Directly reuse the production implementation to avoid writing another "simplified version of the keyword list" in the test
        // (The old version only matches "pro" and misses premium/advanced, causing the test to pass but the production behavior is unverified)
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

    // Opus 4.6: Both are Ultra, high quota is given priority
    assert_eq!(
        compare_tokens_for_model(&ultra_high, &ultra_low, "claude-opus-4-6"),
        Ordering::Less, // ultra_high ranks first
        "Among Ultra accounts, higher quota should come first"
    );
}

/// Test full sorting scenario: mixed account pool
#[test]
fn test_full_sorting_mixed_accounts() {
    fn sort_tokens_for_model(tokens: &mut Vec<ProxyToken>, target_model: &str) {
        const ULTRA_REQUIRED_MODELS: &[&str] = &["claude-opus-4-6", "claude-opus-4-5", "opus"];
        let requires_ultra = {
            let lower = target_model.to_lowercase();
            ULTRA_REQUIRED_MODELS.iter().any(|m| lower.contains(m))
        };

        tokens.sort_by(|a, b| {
            // Directly reuse production implementation
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

    // Create a mixed account pool
    let ultra_high = create_test_token("ultra_high@test.com", Some("ULTRA"), 1.0, None, Some(80));
    let ultra_low = create_test_token("ultra_low@test.com", Some("ULTRA"), 1.0, None, Some(20));
    let pro_high = create_test_token("pro_high@test.com", Some("PRO"), 1.0, None, Some(90));
    let pro_low = create_test_token("pro_low@test.com", Some("PRO"), 1.0, None, Some(30));
    let free = create_test_token("free@test.com", Some("FREE"), 1.0, None, Some(100));

    // High-End Model (Opus 4.6) Sorting
    let mut tokens_opus = vec![
        pro_high.clone(),
        free.clone(),
        ultra_low.clone(),
        pro_low.clone(),
        ultra_high.clone(),
    ];
    sort_tokens_for_model(&mut tokens_opus, "claude-opus-4-6");

    let emails_opus: Vec<&str> = tokens_opus.iter().map(|t| t.email.as_str()).collect();
    // Desired order: Ultra (high quota) > Ultra (low quota) > Pro (high quota) > Pro (low quota) > Free
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

    // Normal model (Sonnet) sorting
    let mut tokens_sonnet = vec![
        pro_high.clone(),
        free.clone(),
        ultra_low.clone(),
        pro_low.clone(),
        ultra_high.clone(),
    ];
    sort_tokens_for_model(&mut tokens_sonnet, "claude-sonnet-4-5");

    let emails_sonnet: Vec<&str> = tokens_sonnet.iter().map(|t| t.email.as_str()).collect();
    // Desired order: Free(100%) > Pro(90%) > Ultra(80%) > Pro(30%) > Ultra(20%) - priority by quota
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

    // 1. Weekly quota is 0, 5H quota is 0, both are exhausted
    // Weekly quota reset_time is after 5 days, 5H reset_time is after 2 hours
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

    // Even if lock_on_zero is false, the weekly quota is exhausted and is still unconditionally locked until the weekly deadline.
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

    // 2. Simulate provider reset in advance: weekly quota is restored to 100%, and 5H quota is still 0
    // If lock_on_zero is turned on, it should automatically align to the shorter 5H deadline (2 hours)
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

    // 3. Simulation 5H is also fully restored (all quotas are positive)
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
