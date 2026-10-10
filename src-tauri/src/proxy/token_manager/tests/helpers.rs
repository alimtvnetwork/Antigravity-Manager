//! tests helpers shared across test modules.

use super::super::ProxyToken;
use std::cmp::Ordering;

pub(crate) fn weekly_quota_account(now: i64) -> serde_json::Value {
    let reset = |seconds| {
        chrono::DateTime::from_timestamp(now + seconds, 0)
            .unwrap()
            .to_rfc3339()
    };
    serde_json::json!({
        "id": "weekly-test", "email": "quota@test.invalid", "created_at": now, "last_used": now,
        "token": {"access_token": "test", "refresh_token": "test", "token_type": "Bearer",
            "expires_in": 3600, "expiry_timestamp": now + 3600, "project_id": "test"},
        "quota": {"last_updated": now, "models": [
            {"name": "gemini-3.1-pro-high", "percentage": 0, "reset_time": reset(7200)},
            {"name": "claude-sonnet-4-6", "percentage": 100, "reset_time": reset(1800)}
        ], "quota_groups": [
            {"display_name": "Gemini Models", "buckets": [
                {"bucket_id": "gemini-weekly", "window": "weekly", "remaining_fraction": 0.0, "reset_time": reset(7200)},
                {"bucket_id": "gemini-5h", "window": "5h", "remaining_fraction": 1.0, "reset_time": reset(1800)}
            ]},
            {"display_name": "Claude and GPT models", "buckets": [
                {"bucket_id": "3p-weekly", "window": "weekly", "remaining_fraction": 1.0, "reset_time": reset(7200)}
            ]}
        ]}
    })
}

/// 创建测试用的 ProxyToken
fn create_test_token(
    email: &str,
    tier: Option<&str>,
    health_score: f32,
    reset_time: Option<i64>,
    remaining_quota: Option<i32>,
) -> ProxyToken {
    ProxyToken {
        account_id: email.to_string(),
        priority: crate::models::account::default_priority(),
        access_token: "test_token".to_string(),
        refresh_token: "test_refresh".to_string(),
        expires_in: 3600,
        timestamp: chrono::Utc::now().timestamp() + 3600,
        email: email.to_string(),
        account_path: PathBuf::from("/tmp/test"),
        project_id: None,
        subscription_tier: tier.map(|s| s.to_string()),
        remaining_quota,
        protected_models: HashSet::new(),
        health_score,
        reset_time,
        validation_blocked: false,
        validation_blocked_until: 0,
        validation_url: None,
        model_quotas: HashMap::new(),
        model_limits: HashMap::new(),
    }
}

/// 测试排序比较函数（与 get_token_internal 中的逻辑一致）
fn compare_tokens(a: &ProxyToken, b: &ProxyToken) -> Ordering {
    const RESET_TIME_THRESHOLD_SECS: i64 = 600; // 10 分钟阈值

    // 统一走 models::quota::tier_priority（与生产排序逻辑共用同一实现）
    let tier_priority =
        |tier: &Option<String>| crate::models::quota::tier_priority(tier.as_deref());

    // First: compare by subscription tier
    let tier_cmp = tier_priority(&a.subscription_tier).cmp(&tier_priority(&b.subscription_tier));
    if tier_cmp != Ordering::Equal {
        return tier_cmp;
    }

    // Second: compare by health score (higher is better)
    let health_cmp = b
        .health_score
        .partial_cmp(&a.health_score)
        .unwrap_or(Ordering::Equal);
    if health_cmp != Ordering::Equal {
        return health_cmp;
    }

    // Third: compare by reset time (earlier/closer is better)
    let reset_a = a.reset_time.unwrap_or(i64::MAX);
    let reset_b = b.reset_time.unwrap_or(i64::MAX);
    let reset_diff = (reset_a - reset_b).abs();

    if reset_diff >= RESET_TIME_THRESHOLD_SECS {
        let reset_cmp = reset_a.cmp(&reset_b);
        if reset_cmp != Ordering::Equal {
            return reset_cmp;
        }
    }

    // Fourth: compare by remaining quota percentage (higher is better)
    let quota_a = a.remaining_quota.unwrap_or(0);
    let quota_b = b.remaining_quota.unwrap_or(0);
    quota_b.cmp(&quota_a)
}

/// 创建带 protected_models 的测试 Token
fn create_test_token_with_protected(
    email: &str,
    remaining_quota: Option<i32>,
    protected_models: HashSet<String>,
) -> ProxyToken {
    ProxyToken {
        account_id: email.to_string(),
        priority: crate::models::account::default_priority(),
        access_token: "test_token".to_string(),
        refresh_token: "test_refresh".to_string(),
        expires_in: 3600,
        timestamp: chrono::Utc::now().timestamp() + 3600,
        email: email.to_string(),
        account_path: PathBuf::from("/tmp/test"),
        project_id: None,
        subscription_tier: Some("PRO".to_string()),
        remaining_quota,
        protected_models,
        health_score: 1.0,
        reset_time: None,
        validation_blocked: false,
        validation_blocked_until: 0,
        validation_url: None,
        model_quotas: HashMap::new(),
        model_limits: HashMap::new(),
    }
}
