use super::*;

// ==================================================================================
// Test 10: Complete process simulation (integration testing style)
// Complete process of simulating multiple accounts, quota protection configuration, and request polling
// ==================================================================================

#[test]
fn test_full_quota_protection_flow() {
    // 1. Configure quota protection
    let config = QuotaProtectionConfig {
        enabled: true,
        threshold_percentage: 60,
        monitored_models: vec!["claude".to_string(), "gemini-3-flash".to_string()],
    };

    // 2. Create multiple accounts to simulate different quota states
    let accounts = vec![
        // Account A: Claude has low quota (50%) and should be protected
        create_mock_token("account-a", "a@example.com", vec!["claude"], Some(50)),
        // Account B: Claude has normal quota (80%) and is not protected.
        create_mock_token("account-b", "b@example.com", vec![], Some(80)),
        // Account C: Both Claude and Gemini are protected
        create_mock_token(
            "account-c",
            "c@example.com",
            vec!["claude", "gemini-3-flash"],
            Some(30),
        ),
        // Account D: Only Gemini is protected
        create_mock_token(
            "account-d",
            "d@example.com",
            vec!["gemini-3-flash"],
            Some(40),
        ),
    ];

    // 3. Simulate multiple requests to verify the account selection logic

    // Request 1: claude-opus-4-5-thinking
    let target_claude = normalize_to_standard_id("claude-opus-4-5-thinking")
        .unwrap_or_else(|| "claude-opus-4-5-thinking".to_string());

    let available_for_claude: Vec<_> = accounts
        .iter()
        .filter(|a| !config.enabled || !a.protected_models.contains(&target_claude))
        .collect();

    // Accounts A and C are filtered, B and D are available
    assert_eq!(available_for_claude.len(), 2);
    let claude_account_ids: Vec<_> = available_for_claude
        .iter()
        .map(|a| a.account_id.as_str())
        .collect();
    assert!(claude_account_ids.contains(&"account-b"));
    assert!(claude_account_ids.contains(&"account-d"));

    // Request 2: gemini-3-flash
    let target_gemini =
        normalize_to_standard_id("gemini-3-flash").unwrap_or_else(|| "gemini-3-flash".to_string());

    let available_for_gemini: Vec<_> = accounts
        .iter()
        .filter(|a| !config.enabled || !a.protected_models.contains(&target_gemini))
        .collect();

    // Accounts C and D are filtered, A and B are available
    assert_eq!(available_for_gemini.len(), 2);
    let gemini_account_ids: Vec<_> = available_for_gemini
        .iter()
        .map(|a| a.account_id.as_str())
        .collect();
    assert!(gemini_account_ids.contains(&"account-a"));
    assert!(gemini_account_ids.contains(&"account-b"));

    // Request 3: Unmonitored model (gemini-2.5-flash)
    let target_unmonitored = normalize_to_standard_id("gemini-2.5-flash")
        .unwrap_or_else(|| "gemini-2.5-flash".to_string());

    let available_for_unmonitored: Vec<_> = accounts
        .iter()
        .filter(|a| !config.enabled || !a.protected_models.contains(&target_unmonitored))
        .collect();

    // Unmonitored model (Gemini 2.5 Flash is actually normalized to monitored 3-flash)
    // Among the 4 test accounts, accounts C and D have 3-flash protection enabled, while accounts A and B have not.
    // Therefore, there should be 2 accounts available.
    assert_eq!(
        available_for_unmonitored.len(),
        2,
        "Gemini 2.5 Flash shares the 3-flash protection status and should have 2 accounts available"
    );
}
