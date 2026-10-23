use super::*;

// ==================================================================================
// Test 1: normalize_to_standard_id function correctness
// Verify that various Claude model names are normalized correctly
// ==================================================================================

#[test]
fn test_normalize_to_standard_id_claude_models() {
    // Claude Sonnet Collection
    assert_eq!(
        normalize_to_standard_id("claude"),
        Some("claude".to_string())
    );
    assert_eq!(
        normalize_to_standard_id("claude-thinking"),
        Some("claude".to_string())
    );

    // Claude Opus Collection - This is the crucial test!
    assert_eq!(
        normalize_to_standard_id("claude-opus-4-5-thinking"),
        Some("claude".to_string()),
        "claude-opus-4-5-thinking should be normalized to claude"
    );

    // Gemini series
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

    // Unsupported models should return None
    assert_eq!(normalize_to_standard_id("gpt-4"), None);
    assert_eq!(normalize_to_standard_id("unknown-model"), None);
}

// ==================================================================================
// Test 5: monitored_models configuration and normalization consistency
// Verify that monitored_models in the configuration can correctly match the normalized model name
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

    // Test whether various model names are in monitored_models after normalization
    let test_cases = vec![
        ("claude-opus-4-5-thinking", true), // normalized to claude
        ("claude-thinking", true),          // normalized to claude
        ("claude", true),                   // direct match
        ("gemini-3-pro-high", true),        // direct match
        ("gemini-3-pro-low", true),         // Normalized to gemini-3-pro-high
        ("gemini-3-flash", true),           // direct match
        ("gpt-4", false),                   // Unsupported model
        ("gemini-2.5-flash", true),         // in watchlist (normalized to gemini-3-flash)
    ];

    for (model_name, expected_monitored) in test_cases {
        let standard_id = normalize_to_standard_id(model_name);

        let is_monitored = match &standard_id {
            Some(id) => config.monitored_models.contains(id),
            None => false,
        };

        assert_eq!(
            is_monitored, expected_monitored,
            "The monitoring status of model {} (normalized to {:?} ) should be {}",
            model_name, standard_id, expected_monitored
        );
    }
}

// ==================================================================================
// Test 12: Edge Case - Case Sensitivity
// ==================================================================================

#[test]
fn test_model_name_case_sensitivity() {
    // normalize_to_standard_id should be case insensitive
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
// Test 19: Quota matching after model name normalization
// Verify that claude's quota can be correctly matched when requesting claude-opus-4-5-thinking
// ==================================================================================

#[test]
fn test_quota_matching_with_normalized_model_name() {
    // Account JSON: Only record the standardized model name
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

    // Request claude-opus-4-5-thinking, which should be normalized to claude
    let request_model = "claude-opus-4-5-thinking";
    let normalized =
        normalize_to_standard_id(request_model).unwrap_or_else(|| request_model.to_string());

    assert_eq!(normalized, "claude", "should be normalized to claude");

    // Read the quota of the normalized model
    let quota = crate::proxy::token_manager::TokenManager::get_model_quota_from_json_for_test(
        &account_path,
        &normalized,
    );

    assert_eq!(
        quota,
        Some(75),
        "claude-opus-4-5-thinking should read claude's quota (75%) after normalization"
    );

    // Clean temporary files
    // Justification: best-effort cleanup; a leftover file is harmless
    crate::error::record_ignored(std::fs::remove_file(&account_path), "remove_file");
}
