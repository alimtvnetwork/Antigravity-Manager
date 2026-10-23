//! Unit Tests for Token Budget Decider & Thinking Allocation Controller (`Imp-617`)

#[cfg(test)]
mod tests {
    use super::super::token_budget_decider::*;

    #[test]
    fn test_thinking_not_supported_clean_passthrough() {
        let req = TokenBudgetRequest::new("gpt-4o", 2_000, 128_000).with_max_output_tokens(4_096);
        let cap = ModelThinkingCapability::ThinkingNotSupported;

        let res = decide_token_budget(&req, &cap).expect("Should allocate budget");
        assert_eq!(res.effective_max_output_tokens, 4_096);
        assert_eq!(res.effective_thinking_budget, None);
        assert!(!res.was_thinking_backfilled);
        assert!(!res.was_thinking_filtered);
        assert_eq!(res.remaining_headroom_tokens, 128_000 - 2_000 - 4_096);
    }

    #[test]
    fn test_thinking_not_supported_filters_requested_budget() {
        let req = TokenBudgetRequest::new("claude-3-5-sonnet", 5_000, 200_000)
            .with_max_output_tokens(8_192)
            .with_thinking_budget(4_096);
        let cap = ModelThinkingCapability::ThinkingNotSupported;

        let res = decide_token_budget(&req, &cap).expect("Should allocate budget");
        assert_eq!(res.effective_max_output_tokens, 8_192);
        assert_eq!(res.effective_thinking_budget, None);
        assert!(!res.was_thinking_backfilled);
        assert!(res.was_thinking_filtered);
        assert_eq!(res.remaining_headroom_tokens, 200_000 - 5_000 - 8_192);
    }

    #[test]
    fn test_requires_thinking_backfills_when_omitted() {
        let req = TokenBudgetRequest::new("gemini-2.0-flash-thinking-exp", 10_000, 1_048_576);
        let cap = ModelThinkingCapability::RequiresThinkingBudget {
            min_budget: 1_024,
            max_budget: 32_768,
            default_budget: 8_192,
        };

        let res = decide_token_budget(&req, &cap).expect("Should backfill thinking");
        assert_eq!(res.effective_thinking_budget, Some(8_192));
        assert!(res.was_thinking_backfilled);
        assert!(!res.was_thinking_filtered);
        assert_eq!(res.effective_max_output_tokens, 16_384);
        assert!(res.effective_max_output_tokens > res.effective_thinking_budget.unwrap());
    }

    #[test]
    fn test_requires_thinking_backfills_zero_budget() {
        let req = TokenBudgetRequest::new("claude-3-7-sonnet-thinking", 1_000, 200_000)
            .with_thinking_budget(0);
        let cap = ModelThinkingCapability::RequiresThinkingBudget {
            min_budget: 1_024,
            max_budget: 32_768,
            default_budget: 4_096,
        };

        let res = decide_token_budget(&req, &cap).expect("Should backfill on zero");
        assert_eq!(res.effective_thinking_budget, Some(4_096));
        assert!(res.was_thinking_backfilled);
    }

    #[test]
    fn test_requires_thinking_clamps_under_min_and_over_max() {
        let cap = ModelThinkingCapability::RequiresThinkingBudget {
            min_budget: 1_024,
            max_budget: 32_768,
            default_budget: 8_192,
        };

        let req_low =
            TokenBudgetRequest::new("gemini-thinking", 1_000, 100_000).with_thinking_budget(512);
        let res_low = decide_token_budget(&req_low, &cap).expect("Should clamp low");
        assert_eq!(res_low.effective_thinking_budget, Some(1_024));
        assert!(!res_low.was_thinking_backfilled);

        let req_high =
            TokenBudgetRequest::new("gemini-thinking", 1_000, 100_000).with_thinking_budget(65_536);
        let res_high = decide_token_budget(&req_high, &cap).expect("Should clamp high");
        assert_eq!(res_high.effective_thinking_budget, Some(32_768));
        assert!(!res_high.was_thinking_backfilled);
    }

    #[test]
    fn test_bumps_output_tokens_to_strictly_exceed_thinking_budget() {
        let req = TokenBudgetRequest::new("claude-3-7-sonnet", 4_000, 200_000)
            .with_max_output_tokens(4_000)
            .with_thinking_budget(16_000);
        let cap = ModelThinkingCapability::RequiresThinkingBudget {
            min_budget: 1_024,
            max_budget: 32_768,
            default_budget: 8_192,
        };

        let res = decide_token_budget(&req, &cap).expect("Should allocate and bump");
        assert_eq!(res.effective_thinking_budget, Some(16_000));
        assert_eq!(res.effective_max_output_tokens, 24_192);
        assert!(res.effective_max_output_tokens > res.effective_thinking_budget.unwrap());
    }

    #[test]
    fn test_clamps_output_tokens_to_tight_context_window() {
        let req = TokenBudgetRequest::new("gpt-4o", 7_000, 10_000).with_max_output_tokens(5_000);
        let cap = ModelThinkingCapability::ThinkingNotSupported;

        let res = decide_token_budget(&req, &cap).expect("Should clamp to headroom");
        assert_eq!(res.effective_max_output_tokens, 3_000);
        assert_eq!(res.remaining_headroom_tokens, 0);
    }

    #[test]
    fn test_rejects_when_prompt_exceeds_context_window() {
        let req = TokenBudgetRequest::new("gpt-4o", 130_000, 128_000);
        let cap = ModelThinkingCapability::ThinkingNotSupported;

        let err = decide_token_budget(&req, &cap).unwrap_err();
        assert_eq!(
            err,
            BudgetViolation::ContextWindowExceeded {
                prompt: 130_000,
                context_window: 128_000,
            }
        );
    }

    #[test]
    fn test_rejects_when_prompt_equals_context_window() {
        let req = TokenBudgetRequest::new("gpt-4o", 128_000, 128_000);
        let cap = ModelThinkingCapability::ThinkingNotSupported;

        let err = decide_token_budget(&req, &cap).unwrap_err();
        assert_eq!(
            err,
            BudgetViolation::ContextWindowExceeded {
                prompt: 128_000,
                context_window: 128_000,
            }
        );
    }

    #[test]
    fn test_rejects_explicit_zero_output_tokens() {
        let req = TokenBudgetRequest::new("gpt-4o", 1_000, 128_000).with_max_output_tokens(0);
        let cap = ModelThinkingCapability::ThinkingNotSupported;

        let err = decide_token_budget(&req, &cap).unwrap_err();
        assert_eq!(err, BudgetViolation::ZeroOutputTokensBudget);
    }

    #[test]
    fn test_rejects_invalid_thinking_budget_range_min_greater_than_max() {
        let req = TokenBudgetRequest::new("model-x", 1_000, 100_000);
        let cap = ModelThinkingCapability::RequiresThinkingBudget {
            min_budget: 16_000,
            max_budget: 8_000,
            default_budget: 10_000,
        };

        let err = decide_token_budget(&req, &cap).unwrap_err();
        assert_eq!(err, BudgetViolation::InvalidThinkingBudgetRange);
    }

    #[test]
    fn test_rejects_invalid_thinking_budget_range_default_out_of_bounds() {
        let req = TokenBudgetRequest::new("model-x", 1_000, 100_000);

        let cap_high = ModelThinkingCapability::RequiresThinkingBudget {
            min_budget: 1_000,
            max_budget: 10_000,
            default_budget: 15_000,
        };
        assert_eq!(
            decide_token_budget(&req, &cap_high).unwrap_err(),
            BudgetViolation::InvalidThinkingBudgetRange
        );

        let cap_low = ModelThinkingCapability::RequiresThinkingBudget {
            min_budget: 2_000,
            max_budget: 10_000,
            default_budget: 500,
        };
        assert_eq!(
            decide_token_budget(&req, &cap_low).unwrap_err(),
            BudgetViolation::InvalidThinkingBudgetRange
        );
    }

    #[test]
    fn test_optional_thinking_behavior() {
        let cap = ModelThinkingCapability::OptionalThinking;

        let req1 =
            TokenBudgetRequest::new("gemini-2.5-flash", 2_000, 100_000).with_thinking_budget(4_096);
        let res1 = decide_token_budget(&req1, &cap).expect("Should allocate");
        assert_eq!(res1.effective_thinking_budget, Some(4_096));
        assert!(!res1.was_thinking_backfilled);
        assert!(!res1.was_thinking_filtered);

        let req2 =
            TokenBudgetRequest::new("gemini-2.5-flash", 2_000, 100_000).with_thinking_budget(0);
        let res2 = decide_token_budget(&req2, &cap).expect("Should allocate");
        assert_eq!(res2.effective_thinking_budget, None);
        assert!(res2.was_thinking_filtered);

        let req3 = TokenBudgetRequest::new("gemini-2.5-flash", 2_000, 100_000);
        let res3 = decide_token_budget(&req3, &cap).expect("Should allocate");
        assert_eq!(res3.effective_thinking_budget, None);
        assert!(!res3.was_thinking_backfilled);
        assert!(!res3.was_thinking_filtered);
    }

    #[test]
    fn test_exact_headroom_arithmetic_invariant() {
        let req = TokenBudgetRequest::new("model", 12_500, 64_000).with_max_output_tokens(10_000);
        let cap = ModelThinkingCapability::ThinkingNotSupported;

        let res = decide_token_budget(&req, &cap).expect("Should allocate");
        assert_eq!(
            req.prompt_tokens + res.effective_max_output_tokens + res.remaining_headroom_tokens,
            req.max_context_window
        );
    }

    #[test]
    fn test_resolve_model_thinking_capability_mappings() {
        assert!(matches!(
            resolve_model_thinking_capability("gemini-2.0-flash-thinking-exp"),
            ModelThinkingCapability::RequiresThinkingBudget { .. }
        ));
        assert!(matches!(
            resolve_model_thinking_capability("claude-3-7-sonnet-20250219"),
            ModelThinkingCapability::RequiresThinkingBudget { .. }
        ));
        assert!(matches!(
            resolve_model_thinking_capability("gemini-2.5-flash-preview"),
            ModelThinkingCapability::OptionalThinking
        ));
        assert!(matches!(
            resolve_model_thinking_capability("gpt-4o-2024-08-06"),
            ModelThinkingCapability::ThinkingNotSupported
        ));
    }

    #[test]
    fn test_decide_token_budget_auto_integration() {
        let req = TokenBudgetRequest::new("gemini-2.0-flash-thinking-exp", 5_000, 100_000);
        let res = decide_token_budget_auto(&req).expect("Auto resolution should succeed");
        assert!(res.was_thinking_backfilled);
        assert!(res.effective_thinking_budget.is_some());
    }

    #[test]
    fn test_decision_summary_helper() {
        let alloc_backfilled = TokenAllocation {
            effective_max_output_tokens: 16_384,
            effective_thinking_budget: Some(8_192),
            remaining_headroom_tokens: 50_000,
            was_thinking_backfilled: true,
            was_thinking_filtered: false,
        };
        assert_eq!(
            alloc_backfilled.decision_summary(),
            ThinkingBudgetDecision::Backfilled(8_192)
        );

        let alloc_filtered = TokenAllocation {
            effective_max_output_tokens: 8_192,
            effective_thinking_budget: None,
            remaining_headroom_tokens: 50_000,
            was_thinking_backfilled: false,
            was_thinking_filtered: true,
        };
        assert_eq!(
            alloc_filtered.decision_summary(),
            ThinkingBudgetDecision::Filtered
        );
    }
}
