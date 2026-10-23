//! Unit tests for Proportional Thinking Budget Allocator & Context Tier Profiler (`Imp-658`).

#[cfg(test)]
mod tests {
    use super::super::thinking_budget_proportional_allocator::*;

    #[test]
    fn test_tier_profile_resolution_wildcards() {
        assert_eq!(
            ContextTierProfile::from_model_name("gemini-2.5-pro").tier,
            ModelContextTier::Large1M
        );
        assert_eq!(
            ContextTierProfile::from_model_name("gemini-2.0-flash-thinking-exp").tier,
            ModelContextTier::Large1M
        );
        assert_eq!(
            ContextTierProfile::from_model_name("gemini-1.5-flash-latest").tier,
            ModelContextTier::Large1M
        );
        assert_eq!(
            ContextTierProfile::from_model_name("gemini-3-pro-preview").tier,
            ModelContextTier::Large1M
        );
        assert_eq!(
            ContextTierProfile::from_model_name("claude-3-7-sonnet-20250219").tier,
            ModelContextTier::Large128k
        );
        assert_eq!(
            ContextTierProfile::from_model_name("claude-3-5-sonnet").tier,
            ModelContextTier::Large128k
        );
        assert_eq!(
            ContextTierProfile::from_model_name("gpt-4o-2024-08-06").tier,
            ModelContextTier::Large128k
        );
        assert_eq!(
            ContextTierProfile::from_model_name("o1-preview").tier,
            ModelContextTier::Large128k
        );
        assert_eq!(
            ContextTierProfile::from_model_name("o3-mini").tier,
            ModelContextTier::Large128k
        );
        assert_eq!(
            ContextTierProfile::from_model_name("claude-2.1").tier,
            ModelContextTier::Medium64k
        );
        assert_eq!(
            ContextTierProfile::from_model_name("custom-model-64k").tier,
            ModelContextTier::Medium64k
        );
        assert_eq!(
            ContextTierProfile::from_model_name("gpt-4-32k").tier,
            ModelContextTier::Medium32k
        );
        assert_eq!(
            ContextTierProfile::from_model_name("gpt-3.5-turbo").tier,
            ModelContextTier::Standard8k
        );
        assert_eq!(
            ContextTierProfile::from_model_name("gpt-4").tier,
            ModelContextTier::Standard8k
        );
    }

    #[test]
    fn test_wildcard_matching_helper() {
        assert!(wildcard_match("gemini-*-flash*", "gemini-2.0-flash-exp"));
        assert!(wildcard_match("claude-3-*", "claude-3-haiku-20240307"));
        assert!(wildcard_match("*-32k*", "my-custom-32k-model"));
        assert!(!wildcard_match("gemini-*-pro", "gemini-2.0-flash"));
        assert!(wildcard_match("gpt-4?", "gpt-4o"));
    }

    #[test]
    fn test_default_proportional_allocation_75_percent_large1m() {
        let req = ProportionalBudgetRequest::new("gemini-2.5-pro", 5_000)
            .with_max_output_tokens(65_536);

        let outcome = allocate_proportional_thinking_budget(&req)
            .expect("Allocation should succeed");

        assert_eq!(outcome.effective_max_output_tokens, 65_536);
        assert_eq!(outcome.effective_thinking_budget, Some(49_152)); // 75% of 65536
        assert_eq!(outcome.headroom_tokens, 16_384); // 25% of 65536
        assert!(outcome.has_thinking());
        assert!(!outcome.is_clamped());
        assert_eq!(
            outcome.action,
            ProportionalAllocationAction::DefaultProportionalAllocated {
                ratio: 0.75,
                thinking_budget: 49_152,
                headroom: 16_384,
            }
        );
    }

    #[test]
    fn test_default_proportional_allocation_75_percent_standard8k() {
        let req = ProportionalBudgetRequest::new("gpt-3.5-turbo", 1_000)
            .with_max_output_tokens(4_096);

        let outcome = allocate_proportional_thinking_budget(&req)
            .expect("Allocation should succeed");

        assert_eq!(outcome.effective_max_output_tokens, 4_096);
        assert_eq!(outcome.effective_thinking_budget, Some(3_072)); // 75% of 4096
        assert_eq!(outcome.headroom_tokens, 1_024); // 25% of 4096
        assert!(outcome.has_thinking());
    }

    #[test]
    fn test_requested_thinking_clamped_down_for_headroom() {
        let req = ProportionalBudgetRequest::new("gemini-2.5-pro", 1_000)
            .with_max_output_tokens(10_000)
            .with_thinking_budget(9_500); // Exceeds required 2048 headroom

        let outcome = allocate_proportional_thinking_budget(&req)
            .expect("Allocation should succeed");

        assert_eq!(outcome.effective_max_output_tokens, 10_000);
        // Min headroom for Large1M is 2048, so max allowable is 10000 - 2048 = 7952
        assert_eq!(outcome.effective_thinking_budget, Some(7_952));
        assert_eq!(outcome.headroom_tokens, 2_048);
        assert!(outcome.is_clamped());
        assert!(matches!(
            outcome.action,
            ProportionalAllocationAction::ClampedDownForHeadroom { .. }
        ));
    }

    #[test]
    fn test_requested_thinking_clamped_up_to_floor() {
        let req = ProportionalBudgetRequest::new("gemini-2.5-pro", 1_000)
            .with_max_output_tokens(32_768)
            .with_thinking_budget(500); // Below Large1M floor of 2048

        let outcome = allocate_proportional_thinking_budget(&req)
            .expect("Allocation should succeed");

        assert_eq!(outcome.effective_thinking_budget, Some(2_048));
        assert!(outcome.is_clamped());
        assert!(matches!(
            outcome.action,
            ProportionalAllocationAction::ClampedUpToFloor {
                requested: 500,
                clamped: 2_048,
                floor: 2_048,
            }
        ));
    }

    #[test]
    fn test_thinking_disabled_when_zero_requested() {
        let req = ProportionalBudgetRequest::new("gemini-2.5-flash", 2_000)
            .with_max_output_tokens(8_192)
            .with_thinking_budget(0);

        let outcome = allocate_proportional_thinking_budget(&req)
            .expect("Allocation should succeed");

        assert_eq!(outcome.effective_thinking_budget, None);
        assert_eq!(outcome.headroom_tokens, 8_192);
        assert!(!outcome.has_thinking());
        assert_eq!(
            outcome.action,
            ProportionalAllocationAction::ThinkingDisabled
        );
    }

    #[test]
    fn test_custom_ratio_allocation() {
        let req = ProportionalBudgetRequest::new("claude-3-7-sonnet", 2_000)
            .with_max_output_tokens(8_000)
            .with_custom_ratio(0.50); // 50% ratio

        let outcome = allocate_proportional_thinking_budget(&req)
            .expect("Allocation should succeed");

        assert_eq!(outcome.applied_ratio, 0.50);
        assert_eq!(outcome.effective_thinking_budget, Some(4_000));
        assert_eq!(outcome.headroom_tokens, 4_000);
    }

    #[test]
    fn test_invalid_ratio_rejected() {
        let req_zero = ProportionalBudgetRequest::new("claude-3-7-sonnet", 1_000)
            .with_custom_ratio(0.0);
        assert!(allocate_proportional_thinking_budget(&req_zero).is_err());

        let req_over = ProportionalBudgetRequest::new("claude-3-7-sonnet", 1_000)
            .with_custom_ratio(1.5);
        assert!(allocate_proportional_thinking_budget(&req_over).is_err());
    }

    #[test]
    fn test_context_window_exhaustion_detected() {
        // Standard8k context window is 8192
        let req = ProportionalBudgetRequest::new("gpt-3.5-turbo", 9_000);
        let err = allocate_proportional_thinking_budget(&req)
            .expect_err("Should fail with context window exhausted");

        assert!(matches!(
            err,
            ProportionalBudgetError::ContextWindowExhausted { .. }
        ));
    }

    #[test]
    fn test_context_window_clamping_for_max_output() {
        // Standard8k context window is 8192
        // Prompt tokens: 6000, requested max output: 4096 (total 10096 > 8192)
        // Max output clamped to 8192 - 6000 = 2192
        let req = ProportionalBudgetRequest::new("gpt-3.5-turbo", 6_000)
            .with_max_output_tokens(4_096);

        let outcome = allocate_proportional_thinking_budget(&req)
            .expect("Allocation should succeed with clamped output");

        assert_eq!(outcome.effective_max_output_tokens, 2_192);
        assert!(outcome.is_clamped());
        assert!(matches!(
            outcome.action,
            ProportionalAllocationAction::ClampedDueToContextOverflow { .. }
        ));
    }

    #[test]
    fn test_evaluate_proportional_budget_default_helper() {
        let outcome = evaluate_proportional_budget_default("gemini-2.5-pro", 10_000, Some(8_192), None)
            .expect("Default evaluation should succeed");

        assert_eq!(outcome.effective_max_output_tokens, 8_192);
        assert!(outcome.has_thinking());
    }
}
