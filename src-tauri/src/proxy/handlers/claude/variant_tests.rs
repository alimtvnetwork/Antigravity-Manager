#[cfg(test)]
mod variant_tests {
    use super::*;

    fn request_with_effort(model: &str, effort: &str, budget_tokens: u32) -> ClaudeRequest {
        serde_json::from_value(json!({
            "model": model,
            "messages": [{"role": "user", "content": "test"}],
            "thinking": {"type": "enabled", "budget_tokens": budget_tokens},
            "output_config": {"effort": effort}
        }))
        .expect("test request must deserialize")
    }

    #[test]
    fn applies_flash_low_effort_and_removes_output_config_before_serialization() {
        let mut request = request_with_effort("gemini-3.5-flash", "low", 10_000);
        let effort = crate::proxy::common::variant_mapping::tier_from_effort(
            request
                .output_config
                .as_ref()
                .and_then(|config| config.effort.as_deref()),
        );

        apply_variant(&mut request, effort, Some(10_000)).expect("Gemini 3.5 Flash must resolve");

        assert_eq!(request.model, "gemini-3.5-flash-extra-low");
        assert!(request.output_config.is_none());
        assert!(serde_json::to_value(request)
            .expect("resolved request must serialize")
            .get("output_config")
            .is_none());
    }

    #[test]
    fn applies_pro_high_effort_over_low_budget() {
        let mut request = request_with_effort("gemini-3.1-pro", "high", 1_000);
        let effort = crate::proxy::common::variant_mapping::tier_from_effort(
            request
                .output_config
                .as_ref()
                .and_then(|config| config.effort.as_deref()),
        );

        apply_variant(&mut request, effort, Some(1_000)).expect("Gemini 3.1 Pro must resolve");

        assert_eq!(request.model, "gemini-pro-agent");
    }

    #[test]
    fn invalid_effort_falls_back_to_budget_tokens_for_gemini_3_model() {
        // Given a Gemini 3 model ("gemini-3-flash") with an unrecognized
        // effort value ("unrecognized"), tier_from_effort returns None, so
        // apply_variant falls back to budget-based tier inference.
        let mut request = request_with_effort("gemini-3-flash", "unrecognized", 4_000);
        let effort = crate::proxy::common::variant_mapping::tier_from_effort(
            request
                .output_config
                .as_ref()
                .and_then(|config| config.effort.as_deref()),
        );

        // tier_from_effort(Some("unrecognized")) → None (invalid value)
        assert_eq!(effort, None);

        // With effort=None and budget=4_000, infer_tier → Medium →
        // resolve_with_tier("gemini-3-flash", None, Some(4_000)) →
        // SPEC_35_FLASH_LOW → physical id "gemini-3.5-flash-low"
        apply_variant(&mut request, effort, Some(4_000))
            .expect("gemini-3-flash must resolve even without valid effort");

        assert_eq!(request.model, "gemini-3.5-flash-low");
        assert!(request.output_config.is_none());
        // SPEC_35_FLASH_LOW has thinking_budget=4_000, preserve_client_budget=false
        assert_eq!(
            request.thinking.as_ref().and_then(|t| t.budget_tokens),
            Some(4_000)
        );
    }

    #[test]
    fn max_effort_maps_to_high_tier_for_gemini_3_flash() {
        let mut request = request_with_effort("gemini-3-flash", "max", 1_000);
        let effort = crate::proxy::common::variant_mapping::tier_from_effort(
            request
                .output_config
                .as_ref()
                .and_then(|config| config.effort.as_deref()),
        );
        assert_eq!(
            effort,
            Some(crate::proxy::common::variant_mapping::VariantTier::High)
        );

        apply_variant(&mut request, effort, Some(1_000))
            .expect("gemini-3-flash must resolve with max effort");

        assert_eq!(request.model, "gemini-3-flash-agent");
        assert_eq!(
            request.thinking.as_ref().and_then(|t| t.budget_tokens),
            Some(10_000)
        );
    }

    #[test]
    fn claude_model_without_variant_mapping_preserves_output_config_on_none() {
        // claude-sonnet-4-5 is NOT in GEMINI_FAMILIES and NOT in
        // resolve_non_variant_model, so resolve_with_tier returns None,
        // and apply_variant returns None without mutating the request.
        let mut request = request_with_effort("claude-sonnet-4-5", "high", 10_000);
        let effort = crate::proxy::common::variant_mapping::tier_from_effort(
            request
                .output_config
                .as_ref()
                .and_then(|config| config.effort.as_deref()),
        );

        let result = apply_variant(&mut request, effort, Some(10_000));
        assert!(
            result.is_none(),
            "unregistered Claude model must return None"
        );

        // Model and output_config must remain untouched.
        assert_eq!(request.model, "claude-sonnet-4-5");
        assert_eq!(
            request
                .output_config
                .as_ref()
                .and_then(|c| c.effort.as_deref()),
            Some("high")
        );
    }
}
