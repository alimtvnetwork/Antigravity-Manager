//! Unit tests for AccountTierFilter (Imp-575)

use std::collections::HashSet;

use super::account_tier_filter::{
    filter_eligible_candidates, is_account_eligible, AccountCapabilities,
    ModelCapabilityRequirement,
};

fn create_test_account(
    id: &str,
    tier: &str,
    is_active: bool,
    thinking: bool,
    vision: bool,
    models: &[&str],
) -> AccountCapabilities {
    let mut supported = HashSet::new();
    for &m in models {
        supported.insert(m.to_string());
    }

    AccountCapabilities {
        account_id: id.to_string(),
        tier: tier.to_string(),
        supported_models: supported,
        supports_thinking: thinking,
        supports_vision: vision,
        is_active,
    }
}

#[test]
fn test_inactive_account_rejected() {
    let acc = create_test_account("acc-1", "pro", false, true, true, &["gemini-2.0-flash"]);
    let req = ModelCapabilityRequirement {
        target_model: "gemini-2.0-flash".to_string(),
        requires_thinking: false,
        requires_vision: false,
        min_tier: None,
    };

    assert!(!is_account_eligible(&acc, &req));
}

#[test]
fn test_thinking_capability_required() {
    let acc_no_thinking = create_test_account("acc-2", "standard", true, false, true, &["claude-3-5-sonnet"]);
    let acc_with_thinking = create_test_account("acc-3", "standard", true, true, true, &["claude-3-5-sonnet"]);

    let req = ModelCapabilityRequirement {
        target_model: "claude-3-5-sonnet".to_string(),
        requires_thinking: true,
        requires_vision: false,
        min_tier: None,
    };

    assert!(!is_account_eligible(&acc_no_thinking, &req));
    assert!(is_account_eligible(&acc_with_thinking, &req));
}

#[test]
fn test_vision_capability_required() {
    let acc_no_vision = create_test_account("acc-4", "pro", true, true, false, &["gemini-1.5-pro"]);
    let acc_with_vision = create_test_account("acc-5", "pro", true, true, true, &["gemini-1.5-pro"]);

    let req = ModelCapabilityRequirement {
        target_model: "gemini-1.5-pro".to_string(),
        requires_thinking: false,
        requires_vision: true,
        min_tier: None,
    };

    assert!(!is_account_eligible(&acc_no_vision, &req));
    assert!(is_account_eligible(&acc_with_vision, &req));
}

#[test]
fn test_model_support_matching() {
    let acc = create_test_account("acc-6", "pro", true, true, true, &["gemini-2.0-flash", "gpt-4o"]);

    let req_supported = ModelCapabilityRequirement {
        target_model: "gemini-2.0-flash-exp".to_string(),
        requires_thinking: false,
        requires_vision: false,
        min_tier: None,
    };
    assert!(is_account_eligible(&acc, &req_supported));

    let req_unsupported = ModelCapabilityRequirement {
        target_model: "claude-3-opus".to_string(),
        requires_thinking: false,
        requires_vision: false,
        min_tier: None,
    };
    assert!(!is_account_eligible(&acc, &req_unsupported));
}

#[test]
fn test_filter_eligible_candidates() {
    let accounts = vec![
        create_test_account("acc-a", "tier1", true, true, true, &["gemini-2.0-flash"]),
        create_test_account("acc-b", "tier1", false, true, true, &["gemini-2.0-flash"]), // Inactive
        create_test_account("acc-c", "tier1", true, false, true, &["gemini-2.0-flash"]), // Missing thinking
    ];

    let req = ModelCapabilityRequirement {
        target_model: "gemini-2.0-flash".to_string(),
        requires_thinking: true,
        requires_vision: false,
        min_tier: None,
    };

    let eligible = filter_eligible_candidates(&accounts, &req);
    assert_eq!(eligible.len(), 1);
    assert_eq!(eligible[0].account_id, "acc-a");
}
