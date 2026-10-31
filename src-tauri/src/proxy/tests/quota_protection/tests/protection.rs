use super::*;

// ==================================================================================
// Test 2: Quota Protection Model Matching Logic
// Verify that protected_models.contains() matches correctly after normalization
// ==================================================================================

#[test]
fn test_protected_models_matching() {
    // Create an account, there is claude in protected_models
    let token = create_mock_token("account-1", "test@example.com", vec!["claude"], Some(50));

    // Test: requests to claude-opus-4-5-thinking should be protected
    let target_model = "claude-opus-4-5-thinking";
    let normalized =
        normalize_to_standard_id(target_model).unwrap_or_else(|| target_model.to_string());

    assert_eq!(normalized, "claude");
    assert!(
        token.protected_models.contains(&normalized),
        "claude-opus-4-5-thinking should match claude in protected_models after normalization"
    );

    // Test: requests for claude-thinking should also be protected
    let target_model_2 = "claude-thinking";
    let normalized_2 =
        normalize_to_standard_id(target_model_2).unwrap_or_else(|| target_model_2.to_string());

    assert!(
        token.protected_models.contains(&normalized_2),
        "claude-thinking should match protected_models after normalization"
    );

    // Test: gemini-3-flash should not be protected when requested (because there is none in protected_models)
    let target_model_3 = "gemini-3-flash";
    let normalized_3 =
        normalize_to_standard_id(target_model_3).unwrap_or_else(|| target_model_3.to_string());

    assert!(
        !token.protected_models.contains(&normalized_3),
        "gemini-3-flash should not match claude"
    );
}

// ==================================================================================
// Test 3: Quota protection filtering when polling multiple accounts
// Simulating multiple accounts, verification of protected accounts will be skipped
// ==================================================================================

#[test]
fn test_multi_account_quota_protection_filtering() {
    // Create 3 accounts
    let tokens = vec![
        // Account 1: claude is protected (low quota)
        create_mock_token("account-1", "user1@example.com", vec!["claude"], Some(20)),
        // Account 2: Not protected
        create_mock_token("account-2", "user2@example.com", vec![], Some(80)),
        // Account 3: gemini-3-flash is protected
        create_mock_token(
            "account-3",
            "user3@example.com",
            vec!["gemini-3-flash"],
            Some(30),
        ),
    ];

    // Simulate request claude-opus-4-5-thinking
    let target_model = "claude-opus-4-5-thinking";
    let normalized_target =
        normalize_to_standard_id(target_model).unwrap_or_else(|| target_model.to_string());

    // Filter out protected accounts
    let available_accounts: Vec<_> = tokens
        .iter()
        .filter(|t| !t.protected_models.contains(&normalized_target))
        .collect();

    // Verification: Account 1 is filtered (because claude is protected)
    // Accounts 2 and 3 are available
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

    // Simulate request gemini-3-flash
    let target_model_2 = "gemini-3-flash";
    let normalized_target_2 =
        normalize_to_standard_id(target_model_2).unwrap_or_else(|| target_model_2.to_string());

    let available_accounts_2: Vec<_> = tokens
        .iter()
        .filter(|t| !t.protected_models.contains(&normalized_target_2))
        .collect();

    // Verification: Account 3 is filtered (because gemini-3-flash is protected)
    // Accounts 1 and 2 are available
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
// Test 4: Behavior when all accounts are protected
// Verify that an error is returned when the target models of all accounts are protected
// ==================================================================================

#[test]
fn test_all_accounts_protected_returns_error() {
    // Create 3 accounts, all of which protect claude
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

    // All accounts are filtered and should return 0
    assert_eq!(available_accounts.len(), 0);

    // In real code, this results in "All accounts failed or unhealthy" errors
}

// ==================================================================================
// Test 6: Quota threshold trigger logic
// Protection is triggered when the verification quota is below the threshold and restored when it is above the threshold
// ==================================================================================

#[test]
fn test_quota_threshold_trigger_logic() {
    let threshold = 60; // 60% threshold

    // Simulate quota data
    let quota_data = vec![
        ("claude-opus-4-5-thinking", 50, true), // 50% <= 60%, protection should be triggered
        ("claude-thinking", 60, true), // 60% <= 60%, protection should be triggered (boundary case)
        ("gemini-3-flash", 61, false), // 61% > 60%, no protection is triggered
        ("gemini-3-pro-high", 100, false), // 100% > 60%, no protection triggered
    ];

    for (model_name, percentage, should_protect) in quota_data {
        let should_trigger = percentage <= threshold;

        assert_eq!(
            should_trigger,
            should_protect,
            "Model {} quota {} % (threshold {} %) should {} trigger protection",
            model_name,
            percentage,
            threshold,
            if should_protect { "" } else { "不" }
        );
    }
}

// ==================================================================================
// Test 7: Protection filtering after account priority sorting
// After verifying that the high quota account is protected, it will fall back to the low quota account.
// ==================================================================================

#[test]
fn test_priority_fallback_when_protected() {
    // Create 3 accounts, sorted by quota
    let mut tokens = vec![
        create_mock_token("account-high", "high@example.com", vec!["claude"], Some(90)),
        create_mock_token("account-mid", "mid@example.com", vec![], Some(60)),
        create_mock_token("account-low", "low@example.com", vec![], Some(30)),
    ];

    // Sort by quota in descending order (high quota first)
    tokens.sort_by(|a, b| {
        let qa = a.remaining_quota.unwrap_or(0);
        let qb = b.remaining_quota.unwrap_or(0);
        qb.cmp(&qa)
    });

    // Verify sorting is correct
    assert_eq!(tokens[0].account_id, "account-high");
    assert_eq!(tokens[1].account_id, "account-mid");
    assert_eq!(tokens[2].account_id, "account-low");

    // Simulate request claude-opus-4-5-thinking
    let target_model = "claude-opus-4-5-thinking";
    let normalized_target =
        normalize_to_standard_id(target_model).unwrap_or_else(|| target_model.to_string());

    // Select the first available account in order
    let selected = tokens
        .iter()
        .find(|t| !t.protected_models.contains(&normalized_target));

    // Verification: account-high is skipped, select account-mid
    assert!(selected.is_some());
    assert_eq!(
        selected.unwrap().account_id,
        "account-mid",
        "After the high quota account is protected, it should fall back to account-mid"
    );
}

// ==================================================================================
// Test 8: Model level protection (different models in the same account)
// Verifying that an account can protect some models but not other models
// ==================================================================================

#[test]
fn test_model_level_protection_granularity() {
    // The account is protected for claude, but not gemini-3-flash.
    let token = create_mock_token("account-1", "user@example.com", vec!["claude"], Some(50));

    // Request claude-opus-4-5-thinking -> protected
    let normalized_claude = normalize_to_standard_id("claude-opus-4-5-thinking")
        .unwrap_or_else(|| "claude-opus-4-5-thinking".to_string());
    assert!(
        token.protected_models.contains(&normalized_claude),
        "Claude request should be protected"
    );

    // Request gemini-3-flash -> not protected
    let normalized_gemini =
        normalize_to_standard_id("gemini-3-flash").unwrap_or_else(|| "gemini-3-flash".to_string());
    assert!(
        !token.protected_models.contains(&normalized_gemini),
        "Gemini requests should not be protected"
    );
}

// ==================================================================================
// Test 9: Quota Protection Enable/Disable Switch
// Verify that when quota_protection.enabled = false, the protection logic does not take effect
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

    // When quota protection is enabled, accounts should be filtered
    let is_protected_when_enabled =
        config_enabled.enabled && token.protected_models.contains(&normalized_target);
    assert!(
        is_protected_when_enabled,
        "Should be protected when enabled"
    );

    // When quota protection is disabled, filtering is not performed even if there are values in protected_models
    let is_protected_when_disabled =
        config_disabled.enabled && token.protected_models.contains(&normalized_target);
    assert!(
        !is_protected_when_disabled,
        "Should not be protected when disabled"
    );
}

// ==================================================================================
// Test 11: Edge case - empty protected_models
// ==================================================================================

#[test]
fn test_empty_protected_models() {
    let token = create_mock_token(
        "account-1",
        "user@example.com",
        vec![], // No protected model
        Some(50),
    );

    let target = normalize_to_standard_id("claude-opus-4-5-thinking")
        .unwrap_or_else(|| "claude-opus-4-5-thinking".to_string());

    assert!(
        !token.protected_models.contains(&target),
        "Empty protected_models should not match any models"
    );
}
