#[cfg(test)]
mod tests {
    use super::super::thinking_cache_invalidation_controller::{
        evaluate_cache_invalidation_prompt, record_user_cache_action, CachePromptDecision,
        ThinkingCacheConfig, UserCacheActionState,
    };

    #[test]
    fn test_routine_release_no_prompt_configured() {
        let config = ThinkingCacheConfig {
            suggestion_delete_thinking_store: false,
            suggestion_target_version: "4.8.2".to_string(),
        };
        let state = UserCacheActionState::NotPrompted;
        let decision = evaluate_cache_invalidation_prompt(&config, &state, true);
        assert_eq!(decision, CachePromptDecision::SkipNoPromptConfigured);
    }

    #[test]
    fn test_no_existing_cache_skips_prompt() {
        let config = ThinkingCacheConfig {
            suggestion_delete_thinking_store: true,
            suggestion_target_version: "4.8.2".to_string(),
        };
        let state = UserCacheActionState::NotPrompted;
        let decision = evaluate_cache_invalidation_prompt(&config, &state, false);
        assert_eq!(decision, CachePromptDecision::SkipNoPromptConfigured);
    }

    #[test]
    fn test_schema_refactor_prompts_user_when_not_prompted() {
        let config = ThinkingCacheConfig {
            suggestion_delete_thinking_store: true,
            suggestion_target_version: "4.8.2".to_string(),
        };
        let state = UserCacheActionState::NotPrompted;
        let decision = evaluate_cache_invalidation_prompt(&config, &state, true);
        assert_eq!(
            decision,
            CachePromptDecision::ShouldPromptUser {
                target_version: "4.8.2".to_string()
            }
        );
    }

    #[test]
    fn test_schema_refactor_skips_when_already_cleaned() {
        let config = ThinkingCacheConfig {
            suggestion_delete_thinking_store: true,
            suggestion_target_version: "4.8.2".to_string(),
        };
        let state = UserCacheActionState::PromptedAndCleaned {
            version: "4.8.2".to_string(),
            timestamp_s: 1700000000,
        };
        let decision = evaluate_cache_invalidation_prompt(&config, &state, true);
        assert_eq!(
            decision,
            CachePromptDecision::SkipAlreadyAcknowledged {
                recorded_version: "4.8.2".to_string()
            }
        );
    }

    #[test]
    fn test_schema_refactor_skips_when_already_dismissed() {
        let config = ThinkingCacheConfig {
            suggestion_delete_thinking_store: true,
            suggestion_target_version: "4.8.2".to_string(),
        };
        let state = UserCacheActionState::PromptedAndDismissed {
            version: "4.8.2".to_string(),
            timestamp_s: 1700000000,
        };
        let decision = evaluate_cache_invalidation_prompt(&config, &state, true);
        assert_eq!(
            decision,
            CachePromptDecision::SkipAlreadyAcknowledged {
                recorded_version: "4.8.2".to_string()
            }
        );
    }

    #[test]
    fn test_schema_refactor_prompts_when_version_differs() {
        let config = ThinkingCacheConfig {
            suggestion_delete_thinking_store: true,
            suggestion_target_version: "4.9.0".to_string(),
        };
        let state = UserCacheActionState::PromptedAndCleaned {
            version: "4.8.2".to_string(),
            timestamp_s: 1700000000,
        };
        let decision = evaluate_cache_invalidation_prompt(&config, &state, true);
        assert_eq!(
            decision,
            CachePromptDecision::ShouldPromptUser {
                target_version: "4.9.0".to_string()
            }
        );
    }

    #[test]
    fn test_version_normalization_with_v_prefix() {
        let config = ThinkingCacheConfig {
            suggestion_delete_thinking_store: true,
            suggestion_target_version: "4.8.2".to_string(),
        };
        let state = UserCacheActionState::PromptedAndDismissed {
            version: "v4.8.2".to_string(),
            timestamp_s: 1700000000,
        };
        let decision = evaluate_cache_invalidation_prompt(&config, &state, true);
        assert_eq!(
            decision,
            CachePromptDecision::SkipAlreadyAcknowledged {
                recorded_version: "v4.8.2".to_string()
            }
        );
    }

    #[test]
    fn test_record_user_cache_action_clean() {
        let state = record_user_cache_action("clean", "4.8.2", 1700001234);
        assert_eq!(
            state,
            UserCacheActionState::PromptedAndCleaned {
                version: "4.8.2".to_string(),
                timestamp_s: 1700001234,
            }
        );
    }

    #[test]
    fn test_record_user_cache_action_dismiss() {
        let state = record_user_cache_action("dismiss", "4.8.2", 1700005678);
        assert_eq!(
            state,
            UserCacheActionState::PromptedAndDismissed {
                version: "4.8.2".to_string(),
                timestamp_s: 1700005678,
            }
        );
    }

    #[test]
    fn test_record_user_cache_action_case_insensitivity() {
        let state = record_user_cache_action("CLEAN", "4.8.2", 1700009999);
        assert_eq!(
            state,
            UserCacheActionState::PromptedAndCleaned {
                version: "4.8.2".to_string(),
                timestamp_s: 1700009999,
            }
        );
    }

    #[test]
    fn test_default_trait_implementations() {
        assert_eq!(
            UserCacheActionState::default(),
            UserCacheActionState::NotPrompted
        );
        let config = ThinkingCacheConfig::default();
        assert!(!config.suggestion_delete_thinking_store);
        assert!(config.suggestion_target_version.is_empty());
    }
}
