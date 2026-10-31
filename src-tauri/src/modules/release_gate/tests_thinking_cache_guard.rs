#[cfg(test)]
mod tests {
    use super::super::thinking_cache_invalidation_guard::{
        evaluate_cache_invalidation_prompt, CacheInvalidationPromptDecision,
    };

    #[test]
    fn test_routine_release_flag_disabled() {
        let decision = evaluate_cache_invalidation_prompt(false, "4.250.0", "4.250.0", false);
        assert!(!decision.should_prompt_user);
        assert!(decision.reason.contains("Routine release"));
    }

    #[test]
    fn test_schema_refactor_release_prompts_user() {
        let decision = evaluate_cache_invalidation_prompt(true, "4.250.0", "v4.250.0", false);
        assert!(decision.should_prompt_user);
        assert!(decision.is_target_version_matched);
        assert!(decision.reason.contains("prompting one-time"));
    }

    #[test]
    fn test_already_acknowledged_does_not_prompt() {
        let decision = evaluate_cache_invalidation_prompt(true, "4.250.0", "4.250.0", true);
        assert!(!decision.should_prompt_user);
        assert!(decision.reason.contains("already acknowledged"));
    }

    #[test]
    fn test_version_mismatch_does_not_prompt() {
        let decision = evaluate_cache_invalidation_prompt(true, "4.251.0", "4.250.0", false);
        assert!(!decision.should_prompt_user);
        assert!(!decision.is_target_version_matched);
        assert!(decision.reason.contains("Version mismatch"));
    }
}
