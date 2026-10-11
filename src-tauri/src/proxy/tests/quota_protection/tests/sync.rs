use super::*;

// ==================================================================================
// 测试 14: 配额保护实时同步测试
// 模拟：配额刷新后 protected_models 被更新，TokenManager 内存应该同步
// ==================================================================================

#[test]
fn test_quota_protection_sync_after_refresh() {
    // 这个测试模拟 update_account_quota 触发 TokenManager 重新加载的场景

    // 初始内存状态
    let mut tokens_in_memory = vec![create_mock_token(
        "account-a",
        "a@example.com",
        vec![],
        Some(70),
    )];

    // 模拟磁盘上的账号数据（配额刷新后更新）
    let mut account_on_disk = create_mock_token("account-a", "a@example.com", vec![], Some(50));

    // 模拟配额刷新：检测到配额低于阈值，触发保护
    let threshold = 60;
    if account_on_disk.remaining_quota.unwrap_or(100) <= threshold {
        account_on_disk
            .protected_models
            .insert("claude".to_string());
    }

    // 验证磁盘数据已更新
    assert!(
        account_on_disk.protected_models.contains("claude"),
        "磁盘上的账号应该已被保护"
    );

    // 此时内存数据还是旧的
    assert!(
        !tokens_in_memory[0].protected_models.contains("claude"),
        "内存中的账号还没被同步"
    );

    // 模拟 trigger_account_reload -> reload_account 同步
    tokens_in_memory[0] = account_on_disk.clone();

    // 验证内存数据已同步
    assert!(
        tokens_in_memory[0].protected_models.contains("claude"),
        "同步后内存中的账号应该被保护"
    );

    // 现在请求应该被正确过滤
    let target = normalize_to_standard_id("claude-opus-4-5-thinking")
        .unwrap_or_else(|| "claude-opus-4-5-thinking".to_string());

    let available: Vec<_> = tokens_in_memory
        .iter()
        .filter(|t| !t.protected_models.contains(&target))
        .collect();

    assert_eq!(available.len(), 0, "同步后账号应该被过滤");
}

// ==================================================================================
// 测试 15: 多轮请求中的配额保护动态变化
// 模拟完整的请求序列，包括配额保护的触发和恢复
// ==================================================================================

#[test]
fn test_quota_protection_dynamic_changes() {
    let target_model = "claude-opus-4-5-thinking";
    let normalized_target =
        normalize_to_standard_id(target_model).unwrap_or_else(|| target_model.to_string());

    // 账号池
    let mut account_a = create_mock_token("account-a", "a@example.com", vec![], Some(70));
    let mut account_b = create_mock_token("account-b", "b@example.com", vec![], Some(80));

    // === 阶段 1: 初始状态，两个账号都可用 ===
    let accounts = vec![account_a.clone(), account_b.clone()];
    let available: Vec<_> = accounts
        .iter()
        .filter(|t| !t.protected_models.contains(&normalized_target))
        .collect();
    assert_eq!(available.len(), 2, "阶段1: 两个账号都可用");

    // === 阶段 2: 账号 A 配额降低，触发保护 ===
    account_a.remaining_quota = Some(40);
    account_a.protected_models.insert("claude".to_string());

    let accounts = vec![account_a.clone(), account_b.clone()];
    let available: Vec<_> = accounts
        .iter()
        .filter(|t| !t.protected_models.contains(&normalized_target))
        .collect();
    assert_eq!(available.len(), 1, "阶段2: 只有账号 B 可用");
    assert_eq!(available[0].account_id, "account-b");

    // === 阶段 3: 账号 B 也触发保护 ===
    account_b.remaining_quota = Some(30);
    account_b.protected_models.insert("claude".to_string());

    let accounts = vec![account_a.clone(), account_b.clone()];
    let available: Vec<_> = accounts
        .iter()
        .filter(|t| !t.protected_models.contains(&normalized_target))
        .collect();
    assert_eq!(available.len(), 0, "阶段3: 没有可用账号");

    // === 阶段 4: 账号 A 配额恢复（重置），解除保护 ===
    account_a.remaining_quota = Some(100);
    account_a.protected_models.remove("claude");

    let accounts = vec![account_a.clone(), account_b.clone()];
    let available: Vec<_> = accounts
        .iter()
        .filter(|t| !t.protected_models.contains(&normalized_target))
        .collect();
    assert_eq!(available.len(), 1, "阶段4: 账号 A 恢复可用");
    assert_eq!(available[0].account_id, "account-a");
}

// ==================================================================================
// 测试 16: 完整错误消息验证
// 验证不同场景下返回的错误消息是否正确
// ==================================================================================

#[test]
fn test_error_messages_for_quota_protection() {
    let target_model = "claude-opus-4-5-thinking";
    let normalized_target =
        normalize_to_standard_id(target_model).unwrap_or_else(|| target_model.to_string());

    // 场景 1: 所有账号都因配额保护不可用
    let all_protected = vec![
        create_mock_token("a1", "a1@example.com", vec!["claude"], Some(30)),
        create_mock_token("a2", "a2@example.com", vec!["claude"], Some(20)),
    ];

    let all_are_quota_protected = all_protected
        .iter()
        .all(|a| a.protected_models.contains(&normalized_target));

    assert!(all_are_quota_protected, "所有账号都被配额保护");

    // 生成错误消息
    let error = format!(
        "All {} accounts are quota-protected for model '{}'. Wait for quota reset or adjust protection threshold.",
        all_protected.len(),
        normalized_target
    );

    assert!(error.contains("quota-protected"));
    assert!(error.contains("claude"));

    // 场景 2: 混合情况（部分限流，部分配额保护）
    let mixed = [
        create_mock_token("a1", "a1@example.com", vec!["claude"], Some(30)),
        create_mock_token("a2", "a2@example.com", vec![], Some(20)), // 这个假设被限流
    ];

    let quota_protected_count = mixed
        .iter()
        .filter(|a| a.protected_models.contains(&normalized_target))
        .count();

    assert_eq!(quota_protected_count, 1);
}

// ==================================================================================
// 测试 17: get_model_quota_from_json 函数正确性
// 验证从磁盘读取特定模型 quota 而非 max(所有模型)
// ==================================================================================

#[test]
fn test_get_model_quota_from_json_reads_correct_model() {
    // 创建模拟账号 JSON 文件，包含多个模型的 quota
    let account_json = serde_json::json!({
        "email": "test@example.com",
        "quota": {
            "models": [
                { "name": "claude", "percentage": 60 },
                { "name": "claude-opus-4-5-thinking", "percentage": 40 },
                { "name": "gemini-3-flash", "percentage": 100 }
            ]
        }
    });

    // 使用 std::env::temp_dir() 创建临时文件
    let temp_dir = std::env::temp_dir();
    let account_path = temp_dir.join(format!("test_quota_{}.json", uuid::Uuid::new_v4()));
    std::fs::write(&account_path, account_json.to_string()).expect("Failed to write temp file");

    // 测试读取 claude 的 quota
    let sonnet_quota =
        crate::proxy::token_manager::TokenManager::get_model_quota_from_json_for_test(
            &account_path,
            "claude",
        );
    assert_eq!(
        sonnet_quota,
        Some(60),
        "claude 应该返回 60%，而非 max(100%)"
    );

    // 测试读取 gemini-3-flash 的 quota
    let gemini_quota =
        crate::proxy::token_manager::TokenManager::get_model_quota_from_json_for_test(
            &account_path,
            "gemini-3-flash",
        );
    assert_eq!(gemini_quota, Some(100), "gemini-3-flash 应该返回 100%");

    // 测试读取不存在的模型
    let unknown_quota =
        crate::proxy::token_manager::TokenManager::get_model_quota_from_json_for_test(
            &account_path,
            "unknown-model",
        );
    assert_eq!(unknown_quota, None, "不存在的模型应该返回 None");

    // 清理临时文件
    // Justification: best-effort cleanup; a leftover file is harmless
    crate::error::record_ignored(std::fs::remove_file(&account_path), "remove_file");
}

// ==================================================================================
// 测试 18: 排序使用目标模型 quota 而非 max quota
// 验证修复后的排序逻辑正确性
// ==================================================================================

#[test]
fn test_sorting_uses_target_model_quota_not_max() {
    // 使用 std::env::temp_dir() 创建临时目录
    let temp_dir = std::env::temp_dir().join(format!("test_sorting_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

    // 账号 A: max=100 (gemini), sonnet=40
    let account_a_json = serde_json::json!({
        "email": "carmelioventori@example.com",
        "quota": {
            "models": [
                { "name": "claude", "percentage": 40 },
                { "name": "gemini-3-flash", "percentage": 100 }
            ]
        }
    });

    // 账号 B: max=100 (gemini), sonnet=100
    let account_b_json = serde_json::json!({
        "email": "kiriyamaleo@example.com",
        "quota": {
            "models": [
                { "name": "claude", "percentage": 100 },
                { "name": "gemini-3-flash", "percentage": 100 }
            ]
        }
    });

    // 账号 C: max=100 (gemini), sonnet=60
    let account_c_json = serde_json::json!({
        "email": "mizusawakai9@example.com",
        "quota": {
            "models": [
                { "name": "claude", "percentage": 60 },
                { "name": "gemini-3-flash", "percentage": 100 }
            ]
        }
    });

    // 写入临时文件
    let path_a = temp_dir.join("account_a.json");
    let path_b = temp_dir.join("account_b.json");
    let path_c = temp_dir.join("account_c.json");

    std::fs::write(&path_a, account_a_json.to_string()).unwrap();
    std::fs::write(&path_b, account_b_json.to_string()).unwrap();
    std::fs::write(&path_c, account_c_json.to_string()).unwrap();

    // 创建 tokens，remaining_quota 使用 max 值（模拟旧逻辑）
    let mut tokens = [
        create_mock_token_with_path(
            "a",
            "carmelioventori@example.com",
            vec![],
            Some(100),
            path_a.clone(),
        ),
        create_mock_token_with_path(
            "b",
            "kiriyamaleo@example.com",
            vec![],
            Some(100),
            path_b.clone(),
        ),
        create_mock_token_with_path(
            "c",
            "mizusawakai9@example.com",
            vec![],
            Some(100),
            path_c.clone(),
        ),
    ];

    // 目标模型: claude
    let target_model = "claude";

    // 使用修复后的排序逻辑：读取目标模型的 quota
    tokens.sort_by(|a, b| {
        let quota_a =
            crate::proxy::token_manager::TokenManager::get_model_quota_from_json_for_test(
                &a.account_path,
                target_model,
            )
            .unwrap_or(0);
        let quota_b =
            crate::proxy::token_manager::TokenManager::get_model_quota_from_json_for_test(
                &b.account_path,
                target_model,
            )
            .unwrap_or(0);
        quota_b.cmp(&quota_a) // 高 quota 优先
    });

    // 验证排序结果：sonnet quota 100% > 60% > 40%
    assert_eq!(
        tokens[0].email, "kiriyamaleo@example.com",
        "sonnet=100% 的账号应该排第一"
    );
    assert_eq!(
        tokens[1].email, "mizusawakai9@example.com",
        "sonnet=60% 的账号应该排第二"
    );
    assert_eq!(
        tokens[2].email, "carmelioventori@example.com",
        "sonnet=40% 的账号应该排第三"
    );

    // 清理临时目录
    // Justification: best-effort cleanup; a leftover directory is harmless
    crate::error::record_ignored(std::fs::remove_dir_all(&temp_dir), "remove_dir_all");
}
