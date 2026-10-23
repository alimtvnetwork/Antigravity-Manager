//! Unit Tests for Thinking Budget Filter Pipeline Guard (`Imp-203`).

#[cfg(test)]
mod tests {
    use super::super::thinking_budget_filter_pipeline::*;

    #[test]
    fn test_non_thinking_model_with_requested_budget_is_stripped() {
        let model = "gpt-4o";
        let requested_budget = Some(2048);
        let default_budget = 4096;

        let report = filter_and_backfill_thinking_budget(model, requested_budget, default_budget);

        assert!(!report.is_active());
        assert_eq!(report.effective_budget(), None);
        assert!(report.decision().is_stripped());

        match report.decision() {
            ThinkingFilterDecision::ThinkingStripped {
                model_pattern,
                original_budget,
                reason,
            } => {
                assert_eq!(model_pattern, "gpt-4o");
                assert_eq!(*original_budget, 2048);
                assert!(reason.contains("does not support thinking budget"));
            }
            other => panic!("Expected ThinkingStripped, got: {:?}", other),
        }
    }

    #[test]
    fn test_thinking_model_with_requested_budget_is_retained() {
        let model = "gemini-2.0-flash-thinking-exp";
        let requested_budget = Some(4096);
        let default_budget = 1024;

        let report = filter_and_backfill_thinking_budget(model, requested_budget, default_budget);

        assert!(report.is_active());
        assert_eq!(report.effective_budget(), Some(4096));
        assert!(report.decision().is_retained());

        match report.decision() {
            ThinkingFilterDecision::ThinkingRetained {
                model_pattern,
                budget,
            } => {
                assert_eq!(model_pattern, "gemini-2.0-flash-thinking-exp");
                assert_eq!(*budget, 4096);
            }
            other => panic!("Expected ThinkingRetained, got: {:?}", other),
        }
    }

    #[test]
    fn test_thinking_model_without_budget_is_backfilled() {
        let model = "o1-preview";
        let requested_budget = None;
        let default_budget = 8192;

        let report = filter_and_backfill_thinking_budget(model, requested_budget, default_budget);

        assert!(report.is_active());
        assert_eq!(report.effective_budget(), Some(8192));
        assert!(report.decision().is_backfilled());

        match report.decision() {
            ThinkingFilterDecision::ThinkingBackfilled {
                model_pattern,
                backfilled_budget,
            } => {
                assert_eq!(model_pattern, "o1-preview");
                assert_eq!(*backfilled_budget, 8192);
            }
            other => panic!("Expected ThinkingBackfilled, got: {:?}", other),
        }
    }

    #[test]
    fn test_non_thinking_model_without_budget_is_no_thinking() {
        let model = "gpt-4o-mini";
        let requested_budget = None;
        let default_budget = 4096;

        let report = filter_and_backfill_thinking_budget(model, requested_budget, default_budget);

        assert!(!report.is_active());
        assert_eq!(report.effective_budget(), None);
        assert!(report.decision().is_no_thinking());
        assert_eq!(*report.decision(), ThinkingFilterDecision::NoThinkingConfigured);
    }

    #[test]
    fn test_non_thinking_model_with_zero_budget_is_no_thinking() {
        let model = "gemini-1.5-pro";
        let requested_budget = Some(0);
        let default_budget = 4096;

        let report = filter_and_backfill_thinking_budget(model, requested_budget, default_budget);

        assert!(!report.is_active());
        assert_eq!(report.effective_budget(), None);
        assert!(report.decision().is_no_thinking());
    }

    #[test]
    fn test_thinking_model_with_zero_budget_disables_thinking() {
        let model = "o3-mini";
        let requested_budget = Some(0);
        let default_budget = 4096;

        let report = filter_and_backfill_thinking_budget(model, requested_budget, default_budget);

        assert!(!report.is_active());
        assert_eq!(report.effective_budget(), None);
        assert!(report.decision().is_no_thinking());
    }

    #[test]
    fn test_wildcard_model_pattern_matching() {
        // Thinking models
        assert!(is_model_thinking_capable("gemini-2.0-flash-thinking-exp-01-21"));
        assert!(is_model_thinking_capable("gemini-2.5-flash-thinking"));
        assert!(is_model_thinking_capable("deepseek-reasoning-v1"));
        assert!(is_model_thinking_capable("custom-reasoning-agent"));
        assert!(is_model_thinking_capable("o1"));
        assert!(is_model_thinking_capable("o1-mini"));
        assert!(is_model_thinking_capable("o1-preview"));
        assert!(is_model_thinking_capable("openai/o1-mini"));
        assert!(is_model_thinking_capable("o3"));
        assert!(is_model_thinking_capable("o3-mini"));
        assert!(is_model_thinking_capable("azure/o3-mini"));
        assert!(is_model_thinking_capable("claude-3-7-sonnet"));
        assert!(is_model_thinking_capable("claude-3-7-sonnet-20250219"));
        assert!(is_model_thinking_capable("anthropic/claude-3-7-sonnet-thought"));

        // Non-thinking models
        assert!(!is_model_thinking_capable("gpt-4o"));
        assert!(!is_model_thinking_capable("gpt-4o-mini"));
        assert!(!is_model_thinking_capable("claude-3-5-sonnet"));
        assert!(!is_model_thinking_capable("claude-3-opus"));
        assert!(!is_model_thinking_capable("gemini-1.5-pro"));
        assert!(!is_model_thinking_capable("gemini-1.5-flash"));
        assert!(!is_model_thinking_capable("mistral-large"));
        assert!(!is_model_thinking_capable(""));
    }

    #[test]
    fn test_display_formatting_for_decisions() {
        let stripped = ThinkingFilterDecision::ThinkingStripped {
            model_pattern: "gpt-4o".into(),
            original_budget: 1024,
            reason: "Not supported".into(),
        };
        assert_eq!(
            stripped.to_string(),
            "Thinking stripped for gpt-4o: budget 1024 removed (Not supported)"
        );

        let backfilled = ThinkingFilterDecision::ThinkingBackfilled {
            model_pattern: "o1".into(),
            backfilled_budget: 2048,
        };
        assert_eq!(
            backfilled.to_string(),
            "Thinking backfilled for o1: budget set to 2048"
        );

        let retained = ThinkingFilterDecision::ThinkingRetained {
            model_pattern: "o3".into(),
            budget: 4096,
        };
        assert_eq!(
            retained.to_string(),
            "Thinking retained for o3: budget 4096"
        );

        let none = ThinkingFilterDecision::NoThinkingConfigured;
        assert_eq!(none.to_string(), "No thinking configured");
    }

    #[test]
    fn test_match_wildcard_utility() {
        assert!(match_wildcard("*thinking*", "model-thinking-test"));
        assert!(match_wildcard("claude-3-7*", "claude-3-7-sonnet"));
        assert!(match_wildcard("?1", "o1"));
        assert!(!match_wildcard("claude-3-7*", "claude-3-5-sonnet"));
    }
}
