use super::*;

// ==================================================================================
// 测试 13: 端到端场景 - 会话中途配额保护生效后的路由切换
// 模拟：请求1 -> 绑定账号A -> 请求2 -> 继续用A -> 刷新配额 -> A被保护 -> 请求3 -> 切换到B
// ==================================================================================

#[test]
fn test_sticky_session_quota_protection_mid_session_single_account() {
    // 场景：只有一个账号，会话绑定后配额保护生效
    // 预期：返回配额保护错误

    let session_id = "session-12345";
    let target_model = "claude-opus-4-5-thinking";
    let normalized_target =
        normalize_to_standard_id(target_model).unwrap_or_else(|| target_model.to_string());

    // 初始状态：账号 A 没有被保护
    let mut account_a = create_mock_token(
        "account-a",
        "a@example.com",
        vec![], // 初始没有保护
        Some(70),
    );

    // 模拟会话绑定表
    let mut session_bindings: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();

    // === 请求 1: 绑定到账号 A ===
    session_bindings.insert(session_id.to_string(), account_a.account_id.clone());

    // 验证请求 1 成功
    let bound_account = session_bindings.get(session_id);
    assert_eq!(bound_account, Some(&"account-a".to_string()));

    // === 请求 2: 继续使用账号 A ===
    // 账号 A 仍然可用
    assert!(!account_a.protected_models.contains(&normalized_target));

    // === 系统触发配额刷新，发现账号 A 配额低于阈值 ===
    // 模拟配额刷新后，account_a 的 claude 被加入保护列表
    account_a.protected_models.insert("claude".to_string());

    // === 请求 3: 尝试使用账号 A，但被配额保护 ===
    let accounts = vec![account_a.clone()]; // 只有一个账号

    // 检查绑定的账号是否被保护
    let bound_id = session_bindings.get(session_id).unwrap();
    let bound_account = accounts.iter().find(|a| &a.account_id == bound_id).unwrap();
    let is_protected = bound_account.protected_models.contains(&normalized_target);

    assert!(is_protected, "账号 A 应该被配额保护");

    // 尝试找其他可用账号
    let available_accounts: Vec<_> = accounts
        .iter()
        .filter(|a| !a.protected_models.contains(&normalized_target))
        .collect();

    // 没有可用账号
    assert_eq!(available_accounts.len(), 0, "应该没有可用账号");

    // 在实际实现中，这会返回错误消息
    // 验证应该返回配额保护相关的错误
    let error_message = if available_accounts.is_empty() {
        if accounts
            .iter()
            .all(|a| a.protected_models.contains(&normalized_target))
        {
            format!(
                "All accounts quota-protected for model {}",
                normalized_target
            )
        } else {
            "All accounts failed or unhealthy.".to_string()
        }
    } else {
        "OK".to_string()
    };

    assert!(
        error_message.contains("quota-protected"),
        "错误消息应该包含 quota-protected: {}",
        error_message
    );
}

#[test]
fn test_sticky_session_quota_protection_mid_session_multi_account() {
    // 场景：多个账号，会话绑定的账号配额保护生效后，应该路由到其他账号

    let session_id = "session-67890";
    let target_model = "claude-opus-4-5-thinking";
    let normalized_target =
        normalize_to_standard_id(target_model).unwrap_or_else(|| target_model.to_string());

    // 初始状态：账号 A 和 B 都没有被保护
    let mut account_a = create_mock_token("account-a", "a@example.com", vec![], Some(70));
    let account_b = create_mock_token("account-b", "b@example.com", vec![], Some(80));

    let mut session_bindings: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();

    // === 请求 1: 绑定到账号 A ===
    session_bindings.insert(session_id.to_string(), account_a.account_id.clone());

    // === 请求 2: 继续使用账号 A ===
    assert!(!account_a.protected_models.contains(&normalized_target));

    // === 系统触发配额刷新，账号 A 被保护 ===
    account_a.protected_models.insert("claude".to_string());

    // === 请求 3: 账号 A 被保护，应该解绑并切换到账号 B ===
    let accounts = vec![account_a.clone(), account_b.clone()];

    // 检查绑定的账号
    let bound_id = session_bindings.get(session_id).unwrap();
    let bound_account = accounts.iter().find(|a| &a.account_id == bound_id).unwrap();
    let is_protected = bound_account.protected_models.contains(&normalized_target);

    assert!(is_protected, "账号 A 应该被配额保护");

    // 模拟解绑逻辑
    if is_protected {
        session_bindings.remove(session_id);
    }

    // 寻找其他可用账号
    let available_accounts: Vec<_> = accounts
        .iter()
        .filter(|a| !a.protected_models.contains(&normalized_target))
        .collect();

    // 应该有账号 B 可用
    assert_eq!(available_accounts.len(), 1);
    assert_eq!(available_accounts[0].account_id, "account-b");

    // 重新绑定到账号 B
    let new_account = available_accounts[0];
    session_bindings.insert(session_id.to_string(), new_account.account_id.clone());

    // 验证新绑定
    assert_eq!(
        session_bindings.get(session_id),
        Some(&"account-b".to_string()),
        "会话应该重新绑定到账号 B"
    );
}
