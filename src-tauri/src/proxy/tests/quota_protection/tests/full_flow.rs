use super::*;

// ==================================================================================
// 测试 10: 完整流程模拟（集成测试风格）
// 模拟多账号、配额保护配置、请求轮询的完整流程
// ==================================================================================

#[test]
fn test_full_quota_protection_flow() {
    // 1. 配置配额保护
    let config = QuotaProtectionConfig {
        enabled: true,
        threshold_percentage: 60,
        monitored_models: vec!["claude".to_string(), "gemini-3-flash".to_string()],
    };

    // 2. 创建多个账号，模拟不同配额状态
    let accounts = vec![
        // 账号 A: Claude 配额低（50%），应该被保护
        create_mock_token("account-a", "a@example.com", vec!["claude"], Some(50)),
        // 账号 B: Claude 配额正常（80%），不被保护
        create_mock_token("account-b", "b@example.com", vec![], Some(80)),
        // 账号 C: Claude 和 Gemini 都被保护
        create_mock_token(
            "account-c",
            "c@example.com",
            vec!["claude", "gemini-3-flash"],
            Some(30),
        ),
        // 账号 D: 只有 Gemini 被保护
        create_mock_token(
            "account-d",
            "d@example.com",
            vec!["gemini-3-flash"],
            Some(40),
        ),
    ];

    // 3. 模拟多次请求，验证账号选择逻辑

    // 请求 1: claude-opus-4-5-thinking
    let target_claude = normalize_to_standard_id("claude-opus-4-5-thinking")
        .unwrap_or_else(|| "claude-opus-4-5-thinking".to_string());

    let available_for_claude: Vec<_> = accounts
        .iter()
        .filter(|a| !config.enabled || !a.protected_models.contains(&target_claude))
        .collect();

    // 账号 A 和 C 被过滤，B 和 D 可用
    assert_eq!(available_for_claude.len(), 2);
    let claude_account_ids: Vec<_> = available_for_claude
        .iter()
        .map(|a| a.account_id.as_str())
        .collect();
    assert!(claude_account_ids.contains(&"account-b"));
    assert!(claude_account_ids.contains(&"account-d"));

    // 请求 2: gemini-3-flash
    let target_gemini =
        normalize_to_standard_id("gemini-3-flash").unwrap_or_else(|| "gemini-3-flash".to_string());

    let available_for_gemini: Vec<_> = accounts
        .iter()
        .filter(|a| !config.enabled || !a.protected_models.contains(&target_gemini))
        .collect();

    // 账号 C 和 D 被过滤，A 和 B 可用
    assert_eq!(available_for_gemini.len(), 2);
    let gemini_account_ids: Vec<_> = available_for_gemini
        .iter()
        .map(|a| a.account_id.as_str())
        .collect();
    assert!(gemini_account_ids.contains(&"account-a"));
    assert!(gemini_account_ids.contains(&"account-b"));

    // 请求 3: 未被监控的模型 (gemini-2.5-flash)
    let target_unmonitored = normalize_to_standard_id("gemini-2.5-flash")
        .unwrap_or_else(|| "gemini-2.5-flash".to_string());

    let available_for_unmonitored: Vec<_> = accounts
        .iter()
        .filter(|a| !config.enabled || !a.protected_models.contains(&target_unmonitored))
        .collect();

    // 未被监控的模型 (Gemini 2.5 Flash 实际上被归一化为已监控的 3-flash)
    // 在 4 个测试账号中，账号 C 和 D 开启了 3-flash 保护，而 A 和 B 未开启。
    // 因此，应该有 2 个账号可用。
    assert_eq!(
        available_for_unmonitored.len(),
        2,
        "Gemini 2.5 Flash 共享了 3-flash 的保护状态，应有 2 个账号可用"
    );
}
