use super::*;

// ==================================================================================
// Test 14: Quota protection real-time synchronization test
// Simulation: protected_models is updated after quota refresh, TokenManager memory should be synchronized
// ==================================================================================

#[test]
fn test_quota_protection_sync_after_refresh() {
    // This test simulates the scenario where update_account_quota triggers TokenManager reloading

    // initial memory state
    let mut tokens_in_memory = vec![create_mock_token(
        "account-a",
        "a@example.com",
        vec![],
        Some(70),
    )];

    // Account data on simulated disk (updated after quota refresh)
    let mut account_on_disk = create_mock_token("account-a", "a@example.com", vec![], Some(50));

    // Simulate quota refresh: It is detected that the quota is lower than the threshold, triggering protection
    let threshold = 60;
    if account_on_disk.remaining_quota.unwrap_or(100) <= threshold {
        account_on_disk
            .protected_models
            .insert("claude".to_string());
    }

    // Verify disk data has been updated
    assert!(
        account_on_disk.protected_models.contains("claude"),
        "The account on disk should be protected"
    );

    // At this time, the memory data is still old.
    assert!(
        !tokens_in_memory[0].protected_models.contains("claude"),
        "The account in memory has not been synchronized yet"
    );

    // Simulate trigger_account_reload -> reload_account synchronization
    tokens_in_memory[0] = account_on_disk.clone();

    // Verify memory data is synchronized
    assert!(
        tokens_in_memory[0].protected_models.contains("claude"),
        "Accounts in memory should be protected after synchronization"
    );

    // Requests should now be properly filtered
    let target = normalize_to_standard_id("claude-opus-4-5-thinking")
        .unwrap_or_else(|| "claude-opus-4-5-thinking".to_string());

    let available: Vec<_> = tokens_in_memory
        .iter()
        .filter(|t| !t.protected_models.contains(&target))
        .collect();

    assert_eq!(
        available.len(),
        0,
        "Accounts should be filtered after synchronization"
    );
}

// ==================================================================================
// Test 15: Dynamic changes in quota protection across multiple rounds of requests
// Simulate a complete request sequence, including triggering and recovery of quota protection
// ==================================================================================

#[test]
fn test_quota_protection_dynamic_changes() {
    let target_model = "claude-opus-4-5-thinking";
    let normalized_target =
        normalize_to_standard_id(target_model).unwrap_or_else(|| target_model.to_string());

    // Account pool
    let mut account_a = create_mock_token("account-a", "a@example.com", vec![], Some(70));
    let mut account_b = create_mock_token("account-b", "b@example.com", vec![], Some(80));

    // === Phase 1: Initial state, both accounts available ===
    let accounts = vec![account_a.clone(), account_b.clone()];
    let available: Vec<_> = accounts
        .iter()
        .filter(|t| !t.protected_models.contains(&normalized_target))
        .collect();
    assert_eq!(available.len(), 2, "Stage 1: Both accounts are available");

    // === Phase 2: Account A’s quota is reduced and protection is triggered ===
    account_a.remaining_quota = Some(40);
    account_a.protected_models.insert("claude".to_string());

    let accounts = vec![account_a.clone(), account_b.clone()];
    let available: Vec<_> = accounts
        .iter()
        .filter(|t| !t.protected_models.contains(&normalized_target))
        .collect();
    assert_eq!(available.len(), 1, "Stage 2: Only account B is available");
    assert_eq!(available[0].account_id, "account-b");

    // === Stage 3: Account B also triggers protection ===
    account_b.remaining_quota = Some(30);
    account_b.protected_models.insert("claude".to_string());

    let accounts = vec![account_a.clone(), account_b.clone()];
    let available: Vec<_> = accounts
        .iter()
        .filter(|t| !t.protected_models.contains(&normalized_target))
        .collect();
    assert_eq!(available.len(), 0, "Stage 3: No account available");

    // === Stage 4: Account A quota restored (reset), protection lifted ===
    account_a.remaining_quota = Some(100);
    account_a.protected_models.remove("claude");

    let accounts = vec![account_a.clone(), account_b.clone()];
    let available: Vec<_> = accounts
        .iter()
        .filter(|t| !t.protected_models.contains(&normalized_target))
        .collect();
    assert_eq!(
        available.len(),
        1,
        "Stage 4: Account A is restored and available"
    );
    assert_eq!(available[0].account_id, "account-a");
}

// ==================================================================================
// Test 16: Full error message verification
// Verify whether the error messages returned in different scenarios are correct
// ==================================================================================

#[test]
fn test_error_messages_for_quota_protection() {
    let target_model = "claude-opus-4-5-thinking";
    let normalized_target =
        normalize_to_standard_id(target_model).unwrap_or_else(|| target_model.to_string());

    // Scenario 1: All accounts are unavailable due to quota protection
    let all_protected = vec![
        create_mock_token("a1", "a1@example.com", vec!["claude"], Some(30)),
        create_mock_token("a2", "a2@example.com", vec!["claude"], Some(20)),
    ];

    let all_are_quota_protected = all_protected
        .iter()
        .all(|a| a.protected_models.contains(&normalized_target));

    assert!(
        all_are_quota_protected,
        "All accounts are protected by quotas"
    );

    // Generate error message
    let error = format!(
        "All {} accounts are quota-protected for model '{}'. Wait for quota reset or adjust protection threshold.",
        all_protected.len(),
        normalized_target
    );

    assert!(error.contains("quota-protected"));
    assert!(error.contains("claude"));

    // Scenario 2: Mixed situation (partial current limiting, partial quota protection)
    let mixed = [
        create_mock_token("a1", "a1@example.com", vec!["claude"], Some(30)),
        create_mock_token("a2", "a2@example.com", vec![], Some(20)), // This assumption is limited
    ];

    let quota_protected_count = mixed
        .iter()
        .filter(|a| a.protected_models.contains(&normalized_target))
        .count();

    assert_eq!(quota_protected_count, 1);
}

// ==================================================================================
// Test 17: get_model_quota_from_json function correctness
// Verify that specific model quota is read from disk instead of max (all models)
// ==================================================================================

#[test]
fn test_get_model_quota_from_json_reads_correct_model() {
    // Create a simulated account JSON file containing quotas for multiple models
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

    // Create temporary files using std::env::temp_dir()
    let temp_dir = std::env::temp_dir();
    let account_path = temp_dir.join(format!("test_quota_{}.json", uuid::Uuid::new_v4()));
    std::fs::write(&account_path, account_json.to_string()).expect("Failed to write temp file");

    // Test reading claude's quota
    let sonnet_quota =
        crate::proxy::token_manager::TokenManager::get_model_quota_from_json_for_test(
            &account_path,
            "claude",
        );
    assert_eq!(
        sonnet_quota,
        Some(60),
        "claude should return 60%, not max(100%)"
    );

    // Test reading the quota of gemini-3-flash
    let gemini_quota =
        crate::proxy::token_manager::TokenManager::get_model_quota_from_json_for_test(
            &account_path,
            "gemini-3-flash",
        );
    assert_eq!(gemini_quota, Some(100), "gemini-3-flash should return 100%");

    // Test reading a non-existent model
    let unknown_quota =
        crate::proxy::token_manager::TokenManager::get_model_quota_from_json_for_test(
            &account_path,
            "unknown-model",
        );
    assert_eq!(
        unknown_quota, None,
        "Models that do not exist should return None"
    );

    // Clean temporary files
    // Justification: best-effort cleanup; a leftover file is harmless
    crate::error::record_ignored(std::fs::remove_file(&account_path), "remove_file");
}

// ==================================================================================
// Test 18: Sorting uses target model quota instead of max quota
// Verify the correctness of the sorting logic after repair
// ==================================================================================

#[test]
fn test_sorting_uses_target_model_quota_not_max() {
    // Create a temporary directory using std::env::temp_dir()
    let temp_dir = std::env::temp_dir().join(format!("test_sorting_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

    // Account A: max=100 (gemini), sonnet=40
    let account_a_json = serde_json::json!({
        "email": "carmelioventori@example.com",
        "quota": {
            "models": [
                { "name": "claude", "percentage": 40 },
                { "name": "gemini-3-flash", "percentage": 100 }
            ]
        }
    });

    // Account B: max=100 (gemini), sonnet=100
    let account_b_json = serde_json::json!({
        "email": "kiriyamaleo@example.com",
        "quota": {
            "models": [
                { "name": "claude", "percentage": 100 },
                { "name": "gemini-3-flash", "percentage": 100 }
            ]
        }
    });

    // Account C: max=100 (gemini), sonnet=60
    let account_c_json = serde_json::json!({
        "email": "mizusawakai9@example.com",
        "quota": {
            "models": [
                { "name": "claude", "percentage": 60 },
                { "name": "gemini-3-flash", "percentage": 100 }
            ]
        }
    });

    // Write to temporary file
    let path_a = temp_dir.join("account_a.json");
    let path_b = temp_dir.join("account_b.json");
    let path_c = temp_dir.join("account_c.json");

    std::fs::write(&path_a, account_a_json.to_string()).unwrap();
    std::fs::write(&path_b, account_b_json.to_string()).unwrap();
    std::fs::write(&path_c, account_c_json.to_string()).unwrap();

    // Create tokens, remaining_quota uses max value (emulates old logic)
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

    // Target model: claude
    let target_model = "claude";

    // Use the fixed sorting logic: read the quota of the target model
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
        quota_b.cmp(&quota_a) // High quota priority
    });

    // Verify the sorting result: sonnet quota 100% > 60% > 40%
    assert_eq!(
        tokens[0].email, "kiriyamaleo@example.com",
        "The account with sonnet=100% should be ranked first"
    );
    assert_eq!(
        tokens[1].email, "mizusawakai9@example.com",
        "Accounts with sonnet=60% should be ranked second"
    );
    assert_eq!(
        tokens[2].email, "carmelioventori@example.com",
        "Accounts with sonnet=40% should be ranked third"
    );

    // Clean up temporary directory
    // Justification: best-effort cleanup; a leftover directory is harmless
    crate::error::record_ignored(std::fs::remove_dir_all(&temp_dir), "remove_dir_all");
}
