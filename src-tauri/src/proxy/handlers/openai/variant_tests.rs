// Variant-mapping unit tests (no openai.rs code under test; kept as a module).
use crate::proxy::common::variant_mapping;
use crate::proxy::mappers::openai::models::ThinkingConfig;

#[test]
fn openai_opus_preserves_client_budget_when_present() {
    let client_budget = Some(32_768);
    let spec = variant_mapping::resolve("claude-opus-4-6-thinking", client_budget)
        .expect("Claude Opus 4.6 thinking must resolve");
    let request_thinking = ThinkingConfig {
        thinking_type: Some("enabled".to_string()),
        budget_tokens: Some(spec.effective_thinking_budget(client_budget)),
        effort: None,
    };

    assert_eq!(request_thinking.budget_tokens, client_budget);
}

#[test]
fn openai_opus_falls_back_to_spec_budget_when_client_budget_is_absent() {
    let client_budget = None;
    let spec = variant_mapping::resolve("claude-opus-4-6-thinking", client_budget)
        .expect("Claude Opus 4.6 thinking must resolve");
    let request_thinking = ThinkingConfig {
        thinking_type: Some("enabled".to_string()),
        budget_tokens: Some(spec.effective_thinking_budget(client_budget)),
        effort: None,
    };

    assert_eq!(request_thinking.budget_tokens, Some(1_024));
}
