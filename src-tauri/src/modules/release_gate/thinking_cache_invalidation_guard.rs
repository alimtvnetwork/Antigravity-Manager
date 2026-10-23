//! Thinking Cache Invalidation Control Guard.
//!
//! Enforces release clean prompts per AGENTS.md:
//! - Routine releases: Keep `SUGGESTION_DELETE_THINKING_STORE = false`.
//! - Architecture/schema refactors: Set `SUGGESTION_DELETE_THINKING_STORE = true`
//!   and `SUGGESTION_TARGET_VERSION = '<version>'`.
//! - On upgrade, users with existing cache get a one-time prompt; state persists in `gui_config.json`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheInvalidationPromptDecision {
    pub should_prompt_user: bool,
    pub is_target_version_matched: bool,
    pub reason: String,
}

/// Evaluates whether the user should be prompted to invalidate thinking cache.
pub fn evaluate_cache_invalidation_prompt(
    is_flag_enabled: bool,
    target_version: &str,
    current_app_version: &str,
    has_user_acknowledged: bool,
) -> CacheInvalidationPromptDecision {
    if !is_flag_enabled {
        return CacheInvalidationPromptDecision {
            should_prompt_user: false,
            is_target_version_matched: false,
            reason: "Routine release: SUGGESTION_DELETE_THINKING_STORE is disabled".to_string(),
        };
    }

    if has_user_acknowledged {
        return CacheInvalidationPromptDecision {
            should_prompt_user: false,
            is_target_version_matched: true,
            reason: "User has already acknowledged cache invalidation for target version"
                .to_string(),
        };
    }

    let clean_target = target_version.trim().trim_start_matches('v');
    let clean_current = current_app_version.trim().trim_start_matches('v');

    let is_matched = clean_target == clean_current;

    if is_matched {
        CacheInvalidationPromptDecision {
            should_prompt_user: true,
            is_target_version_matched: true,
            reason: format!(
                "Architecture/schema refactor release v{}: prompting one-time thinking cache invalidation",
                clean_target
            ),
        }
    } else {
        CacheInvalidationPromptDecision {
            should_prompt_user: false,
            is_target_version_matched: false,
            reason: format!(
                "Version mismatch: current v{} != target v{}",
                clean_current, clean_target
            ),
        }
    }
}
