use super::*;

// ==================================================================================
// 测试 1: normalize_to_standard_id 函数正确性
// 验证各种 Claude 模型名称都能正确归一化
// ==================================================================================

#[test]
fn test_normalize_to_standard_id_claude_models() {
    // Claude Sonnet 系列
    assert_eq!(
        normalize_to_standard_id("claude"),
        Some("claude".to_string())
    );
    assert_eq!(
        normalize_to_standard_id("claude-thinking"),
        Some("claude".to_string())
    );

    // Claude Opus 系列 - 这是关键的测试！
    assert_eq!(
        normalize_to_standard_id("claude-opus-4-5-thinking"),
        Some("claude".to_string()),
        "claude-opus-4-5-thinking 应该归一化为 claude"
    );

    // Gemini 系列
    assert_eq!(
        normalize_to_standard_id("gemini-3-flash"),
        Some("gemini-3-flash".to_string())
    );
    assert_eq!(
        normalize_to_standard_id("gemini-3-pro-high"),
        Some("gemini-3-pro-high".to_string())
    );
    assert_eq!(
        normalize_to_standard_id("gemini-3-pro-low"),
        Some("gemini-3-pro-high".to_string())
    );

    // 不支持的模型应返回 None
    assert_eq!(normalize_to_standard_id("gpt-4"), None);
    assert_eq!(normalize_to_standard_id("unknown-model"), None);
}

// ==================================================================================
// 测试 5: monitored_models 配置与归一化一致性
// 验证配置中的 monitored_models 能正确匹配归一化后的模型名
// ==================================================================================

#[test]
fn test_monitored_models_normalization_consistency() {
    let config = QuotaProtectionConfig {
        enabled: true,
        threshold_percentage: 60,
        monitored_models: vec![
            "claude".to_string(),
            "gemini-3-pro-high".to_string(),
            "gemini-3-flash".to_string(),
        ],
    };

    // 测试各种模型名归一化后是否在 monitored_models 中
    let test_cases = vec![
        ("claude-opus-4-5-thinking", true), // 归一化为 claude
        ("claude-thinking", true),          // 归一化为 claude
        ("claude", true),                   // 直接匹配
        ("gemini-3-pro-high", true),        // 直接匹配
        ("gemini-3-pro-low", true),         // 归一化为 gemini-3-pro-high
        ("gemini-3-flash", true),           // 直接匹配
        ("gpt-4", false),                   // 不支持的模型
        ("gemini-2.5-flash", true),         // 在监控列表中 (归一化为 gemini-3-flash)
    ];

    for (model_name, expected_monitored) in test_cases {
        let standard_id = normalize_to_standard_id(model_name);

        let is_monitored = match &standard_id {
            Some(id) => config.monitored_models.contains(id),
            None => false,
        };

        assert_eq!(
            is_monitored, expected_monitored,
            "模型 {} (归一化为 {:?}) 的监控状态应为 {}",
            model_name, standard_id, expected_monitored
        );
    }
}

// ==================================================================================
// 测试 12: 边界情况 - 大小写敏感性
// ==================================================================================

#[test]
fn test_model_name_case_sensitivity() {
    // normalize_to_standard_id 应该是大小写不敏感的
    assert_eq!(
        normalize_to_standard_id("Claude-Opus-4-5-Thinking"),
        Some("claude".to_string())
    );
    assert_eq!(
        normalize_to_standard_id("CLAUDE-OPUS-4-5-THINKING"),
        Some("claude".to_string())
    );
    assert_eq!(
        normalize_to_standard_id("GEMINI-3-FLASH"),
        Some("gemini-3-flash".to_string())
    );
}

// ==================================================================================
// 测试 19: 模型名称归一化后的 quota 匹配
// 验证请求 claude-opus-4-5-thinking 时能正确匹配 claude 的 quota
// ==================================================================================

#[test]
fn test_quota_matching_with_normalized_model_name() {
    // 账号 JSON：只记录标准化后的模型名
    let account_json = serde_json::json!({
        "email": "test@example.com",
        "quota": {
            "models": [
                { "name": "claude", "percentage": 75 },
                { "name": "gemini-3-flash", "percentage": 90 }
            ]
        }
    });

    let temp_dir = std::env::temp_dir();
    let account_path = temp_dir.join(format!("test_normalized_{}.json", uuid::Uuid::new_v4()));
    std::fs::write(&account_path, account_json.to_string()).expect("Failed to write temp file");

    // 请求 claude-opus-4-5-thinking，应该归一化为 claude
    let request_model = "claude-opus-4-5-thinking";
    let normalized =
        normalize_to_standard_id(request_model).unwrap_or_else(|| request_model.to_string());

    assert_eq!(normalized, "claude", "应该归一化为 claude");

    // 读取归一化后模型的 quota
    let quota = crate::proxy::token_manager::TokenManager::get_model_quota_from_json_for_test(
        &account_path,
        &normalized,
    );

    assert_eq!(
        quota,
        Some(75),
        "claude-opus-4-5-thinking 归一化后应该读取 claude 的 quota (75%)"
    );

    // 清理临时文件
    // Justification: best-effort cleanup; a leftover file is harmless
    crate::error::record_ignored(std::fs::remove_file(&account_path), "remove_file");
}
