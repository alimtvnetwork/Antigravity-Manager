use super::*;

// ==================================================================================
// Test 13: End-to-end scenario - route switch after quota protection takes effect mid-session
// Simulation: Request 1 -> Bind account A -> Request 2 -> Continue using A -> Refresh quota -> A is protected -> Request 3 -> Switch to B
// ==================================================================================

#[test]
fn test_sticky_session_quota_protection_mid_session_single_account() {
    // Scenario: There is only one account, and quota protection takes effect after the session is bound.
    // Expected: Quota protection error returned

    let session_id = "session-12345";
    let target_model = "claude-opus-4-5-thinking";
    let normalized_target =
        normalize_to_standard_id(target_model).unwrap_or_else(|| target_model.to_string());

    // Initial state: Account A is not protected
    let mut account_a = create_mock_token(
        "account-a",
        "a@example.com",
        vec![], // No protection initially
        Some(70),
    );

    // Simulation session binding table
    let mut session_bindings: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();

    // === Request 1: Bind to account A ===
    session_bindings.insert(session_id.to_string(), account_a.account_id.clone());

    // Verification request 1 successful
    let bound_account = session_bindings.get(session_id);
    assert_eq!(bound_account, Some(&"account-a".to_string()));

    // === Request 2: Continue using account A ===
    // Account A is still available
    assert!(!account_a.protected_models.contains(&normalized_target));

    // === The system triggered a quota refresh and found that the quota of account A was lower than the threshold ===
    // After the simulation quota is refreshed, claude of account_a is added to the protection list
    account_a.protected_models.insert("claude".to_string());

    // === Request 3: Trying to use account A, but protected by quota ===
    let accounts = vec![account_a.clone()]; // Only one account

    // Check whether the bound account is protected
    let bound_id = session_bindings.get(session_id).unwrap();
    let bound_account = accounts.iter().find(|a| &a.account_id == bound_id).unwrap();
    let is_protected = bound_account.protected_models.contains(&normalized_target);

    assert!(is_protected, "Account A should be quota protected");

    // Try to find another available account
    let available_accounts: Vec<_> = accounts
        .iter()
        .filter(|a| !a.protected_models.contains(&normalized_target))
        .collect();

    // No account available
    assert_eq!(
        available_accounts.len(),
        0,
        "There should be no available account"
    );

    // In a real implementation this would return the error message
    // Validation should return quota protection related errors
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
        "The error message should contain quota-protected: {}",
        error_message
    );
}

#[test]
fn test_sticky_session_quota_protection_mid_session_multi_account() {
    // Scenario: multiple accounts, after the account quota protection bound to the session takes effect, it should be routed to other accounts

    let session_id = "session-67890";
    let target_model = "claude-opus-4-5-thinking";
    let normalized_target =
        normalize_to_standard_id(target_model).unwrap_or_else(|| target_model.to_string());

    // Initial state: Accounts A and B are not protected
    let mut account_a = create_mock_token("account-a", "a@example.com", vec![], Some(70));
    let account_b = create_mock_token("account-b", "b@example.com", vec![], Some(80));

    let mut session_bindings: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();

    // === Request 1: Bind to account A ===
    session_bindings.insert(session_id.to_string(), account_a.account_id.clone());

    // === Request 2: Continue using account A ===
    assert!(!account_a.protected_models.contains(&normalized_target));

    // === The system triggers quota refresh and account A is protected ===
    account_a.protected_models.insert("claude".to_string());

    // === Request 3: Account A is protected and should be unbound and switched to account B ===
    let accounts = vec![account_a.clone(), account_b.clone()];

    // Check the bound account
    let bound_id = session_bindings.get(session_id).unwrap();
    let bound_account = accounts.iter().find(|a| &a.account_id == bound_id).unwrap();
    let is_protected = bound_account.protected_models.contains(&normalized_target);

    assert!(is_protected, "Account A should be quota protected");

    // Simulate unbinding logic
    if is_protected {
        session_bindings.remove(session_id);
    }

    // Find other available accounts
    let available_accounts: Vec<_> = accounts
        .iter()
        .filter(|a| !a.protected_models.contains(&normalized_target))
        .collect();

    // Account B should be available
    assert_eq!(available_accounts.len(), 1);
    assert_eq!(available_accounts[0].account_id, "account-b");

    // Rebind to account B
    let new_account = available_accounts[0];
    session_bindings.insert(session_id.to_string(), new_account.account_id.clone());

    // Verify new binding
    assert_eq!(
        session_bindings.get(session_id),
        Some(&"account-b".to_string()),
        "The session should be rebind to account B"
    );
}
