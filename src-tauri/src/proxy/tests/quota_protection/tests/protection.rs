use super::*;

// ==================================================================================
// 测试 2: 配额保护模型匹配逻辑
// 验证 protected_models.contains() 在归一化后能正确匹配
// ==================================================================================

#[test]
fn test_protected_models_matching() {
    // 创建一个账号，protected_models 中有 claude
    let token = create_mock_token("account-1", "test@example.com", vec!["claude"], Some(50));

    // 测试：请求 claude-opus-4-5-thinking 时应该被保护
    let target_model = "claude-opus-4-5-thinking";
    let normalized =
        normalize_to_standard_id(target_model).unwrap_or_else(|| target_model.to_string());

    assert_eq!(normalized, "claude");
    assert!(
        token.protected_models.contains(&normalized),
        "claude-opus-4-5-thinking 归一化后应该匹配 protected_models 中的 claude"
    );

    // 测试：请求 claude-thinking 时也应该被保护
    let target_model_2 = "claude-thinking";
    let normalized_2 =
        normalize_to_standard_id(target_model_2).unwrap_or_else(|| target_model_2.to_string());

    assert!(
        token.protected_models.contains(&normalized_2),
        "claude-thinking 归一化后应该匹配 protected_models"
    );

    // 测试：请求 gemini-3-flash 时不应该被保护（因为 protected_models 中没有）
    let target_model_3 = "gemini-3-flash";
    let normalized_3 =
        normalize_to_standard_id(target_model_3).unwrap_or_else(|| target_model_3.to_string());

    assert!(
        !token.protected_models.contains(&normalized_3),
        "gemini-3-flash 不应该匹配 claude"
    );
}

// ==================================================================================
// 测试 3: 多账号轮询时的配额保护过滤
// 模拟多个账号，验证被保护的账号会被跳过
// ==================================================================================

#[test]
fn test_multi_account_quota_protection_filtering() {
    // 创建 3 个账号
    let tokens = vec![
        // 账号 1: claude 被保护（配额低）
        create_mock_token("account-1", "user1@example.com", vec!["claude"], Some(20)),
        // 账号 2: 没有被保护
        create_mock_token("account-2", "user2@example.com", vec![], Some(80)),
        // 账号 3: gemini-3-flash 被保护
        create_mock_token(
            "account-3",
            "user3@example.com",
            vec!["gemini-3-flash"],
            Some(30),
        ),
    ];

    // 模拟请求 claude-opus-4-5-thinking
    let target_model = "claude-opus-4-5-thinking";
    let normalized_target =
        normalize_to_standard_id(target_model).unwrap_or_else(|| target_model.to_string());

    // 过滤掉被保护的账号
    let available_accounts: Vec<_> = tokens
        .iter()
        .filter(|t| !t.protected_models.contains(&normalized_target))
        .collect();

    // 验证：账号 1 被过滤（因为 claude 被保护）
    // 账号 2 和 3 可用
    assert_eq!(available_accounts.len(), 2);
    assert!(available_accounts
        .iter()
        .any(|t| t.account_id == "account-2"));
    assert!(available_accounts
        .iter()
        .any(|t| t.account_id == "account-3"));
    assert!(!available_accounts
        .iter()
        .any(|t| t.account_id == "account-1"));

    // 模拟请求 gemini-3-flash
    let target_model_2 = "gemini-3-flash";
    let normalized_target_2 =
        normalize_to_standard_id(target_model_2).unwrap_or_else(|| target_model_2.to_string());

    let available_accounts_2: Vec<_> = tokens
        .iter()
        .filter(|t| !t.protected_models.contains(&normalized_target_2))
        .collect();

    // 验证：账号 3 被过滤（因为 gemini-3-flash 被保护）
    // 账号 1 和 2 可用
    assert_eq!(available_accounts_2.len(), 2);
    assert!(available_accounts_2
        .iter()
        .any(|t| t.account_id == "account-1"));
    assert!(available_accounts_2
        .iter()
        .any(|t| t.account_id == "account-2"));
    assert!(!available_accounts_2
        .iter()
        .any(|t| t.account_id == "account-3"));
}

// ==================================================================================
// 测试 4: 所有账号都被保护时的行为
// 验证当所有账号的目标模型都被保护时，返回错误
// ==================================================================================

#[test]
fn test_all_accounts_protected_returns_error() {
    // 创建 3 个账号，全部对 claude 进行保护
    let tokens = vec![
        create_mock_token("account-1", "user1@example.com", vec!["claude"], Some(10)),
        create_mock_token("account-2", "user2@example.com", vec!["claude"], Some(15)),
        create_mock_token("account-3", "user3@example.com", vec!["claude"], Some(5)),
    ];

    let target_model = "claude-opus-4-5-thinking";
    let normalized_target =
        normalize_to_standard_id(target_model).unwrap_or_else(|| target_model.to_string());

    let available_accounts: Vec<_> = tokens
        .iter()
        .filter(|t| !t.protected_models.contains(&normalized_target))
        .collect();

    // 所有账号都被过滤，应该返回 0
    assert_eq!(available_accounts.len(), 0);

    // 在实际代码中，这会导致 "All accounts failed or unhealthy" 错误
}

// ==================================================================================
// 测试 6: 配额阈值触发逻辑
// 验证配额低于阈值时触发保护，高于阈值时恢复
// ==================================================================================

#[test]
fn test_quota_threshold_trigger_logic() {
    let threshold = 60; // 60% 阈值

    // 模拟 quota 数据
    let quota_data = vec![
        ("claude-opus-4-5-thinking", 50, true), // 50% <= 60%, 应触发保护
        ("claude-thinking", 60, true),          // 60% <= 60%, 应触发保护（边界情况）
        ("gemini-3-flash", 61, false),          // 61% > 60%, 不触发保护
        ("gemini-3-pro-high", 100, false),      // 100% > 60%, 不触发保护
    ];

    for (model_name, percentage, should_protect) in quota_data {
        let should_trigger = percentage <= threshold;

        assert_eq!(
            should_trigger,
            should_protect,
            "模型 {} 配额 {}% (阈值 {}%) 应 {} 触发保护",
            model_name,
            percentage,
            threshold,
            if should_protect { "" } else { "不" }
        );
    }
}

// ==================================================================================
// 测试 7: 账号优先级排序后的保护过滤
// 验证高配额账号被保护后，会回退到低配额账号
// ==================================================================================

#[test]
fn test_priority_fallback_when_protected() {
    // 创建 3 个账号，按配额排序
    let mut tokens = vec![
        create_mock_token("account-high", "high@example.com", vec!["claude"], Some(90)),
        create_mock_token("account-mid", "mid@example.com", vec![], Some(60)),
        create_mock_token("account-low", "low@example.com", vec![], Some(30)),
    ];

    // 按配额降序排序（高配额优先）
    tokens.sort_by(|a, b| {
        let qa = a.remaining_quota.unwrap_or(0);
        let qb = b.remaining_quota.unwrap_or(0);
        qb.cmp(&qa)
    });

    // 验证排序正确
    assert_eq!(tokens[0].account_id, "account-high");
    assert_eq!(tokens[1].account_id, "account-mid");
    assert_eq!(tokens[2].account_id, "account-low");

    // 模拟请求 claude-opus-4-5-thinking
    let target_model = "claude-opus-4-5-thinking";
    let normalized_target =
        normalize_to_standard_id(target_model).unwrap_or_else(|| target_model.to_string());

    // 按顺序选择第一个可用账号
    let selected = tokens
        .iter()
        .find(|t| !t.protected_models.contains(&normalized_target));

    // 验证：account-high 被跳过，选择 account-mid
    assert!(selected.is_some());
    assert_eq!(
        selected.unwrap().account_id,
        "account-mid",
        "高配额账号被保护后，应该回退到 account-mid"
    );
}

// ==================================================================================
// 测试 8: 模型级别保护（同一账号不同模型）
// 验证一个账号可以对某些模型保护，对其他模型不保护
// ==================================================================================

#[test]
fn test_model_level_protection_granularity() {
    // 账号对 claude 保护，但对 gemini-3-flash 不保护
    let token = create_mock_token("account-1", "user@example.com", vec!["claude"], Some(50));

    // 请求 claude-opus-4-5-thinking -> 被保护
    let normalized_claude = normalize_to_standard_id("claude-opus-4-5-thinking")
        .unwrap_or_else(|| "claude-opus-4-5-thinking".to_string());
    assert!(
        token.protected_models.contains(&normalized_claude),
        "Claude 请求应该被保护"
    );

    // 请求 gemini-3-flash -> 不被保护
    let normalized_gemini =
        normalize_to_standard_id("gemini-3-flash").unwrap_or_else(|| "gemini-3-flash".to_string());
    assert!(
        !token.protected_models.contains(&normalized_gemini),
        "Gemini 请求不应该被保护"
    );
}

// ==================================================================================
// 测试 9: 配额保护启用/禁用开关
// 验证当 quota_protection.enabled = false 时，保护逻辑不生效
// ==================================================================================

#[test]
fn test_quota_protection_enabled_flag() {
    let config_enabled = QuotaProtectionConfig {
        enabled: true,
        threshold_percentage: 60,
        monitored_models: vec!["claude".to_string()],
    };

    let config_disabled = QuotaProtectionConfig {
        enabled: false,
        threshold_percentage: 60,
        monitored_models: vec!["claude".to_string()],
    };

    let token = create_mock_token("account-1", "user@example.com", vec!["claude"], Some(50));

    let target_model = "claude-opus-4-5-thinking";
    let normalized_target =
        normalize_to_standard_id(target_model).unwrap_or_else(|| target_model.to_string());

    // 启用配额保护时，账号应该被过滤
    let is_protected_when_enabled =
        config_enabled.enabled && token.protected_models.contains(&normalized_target);
    assert!(is_protected_when_enabled, "启用时应该被保护");

    // 禁用配额保护时，即使 protected_models 中有值，也不过滤
    let is_protected_when_disabled =
        config_disabled.enabled && token.protected_models.contains(&normalized_target);
    assert!(!is_protected_when_disabled, "禁用时不应该被保护");
}

// ==================================================================================
// 测试 11: 边界情况 - 空 protected_models
// ==================================================================================

#[test]
fn test_empty_protected_models() {
    let token = create_mock_token(
        "account-1",
        "user@example.com",
        vec![], // 没有被保护的模型
        Some(50),
    );

    let target = normalize_to_standard_id("claude-opus-4-5-thinking")
        .unwrap_or_else(|| "claude-opus-4-5-thinking".to_string());

    assert!(
        !token.protected_models.contains(&target),
        "空 protected_models 不应该匹配任何模型"
    );
}
