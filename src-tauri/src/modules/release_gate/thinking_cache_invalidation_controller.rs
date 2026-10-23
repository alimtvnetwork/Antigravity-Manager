//! Thinking Cache Invalidation Controller.
//!
//! Enforces thinking cache invalidation prompting logic per AGENTS.md:
//! - Routine releases (no prompt): Keep `SUGGESTION_DELETE_THINKING_STORE = false`.
//! - Architecture / schema refactors (prompt users to clean once):
//!   Set `SUGGESTION_DELETE_THINKING_STORE = true`, Set `SUGGESTION_TARGET_VERSION = '<version>'`.
//! - On upgrade, users with existing cache get a one-time prompt; action state persists in `gui_config.json`.

use serde::{Deserialize, Serialize};

/// Represents the persistent user action state regarding the cache invalidation suggestion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserCacheActionState {
    NotPrompted,
    PromptedAndCleaned { version: String, timestamp_s: u64 },
    PromptedAndDismissed { version: String, timestamp_s: u64 },
}

impl Default for UserCacheActionState {
    fn default() -> Self {
        Self::NotPrompted
    }
}

/// Thinking cache invalidation configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThinkingCacheConfig {
    pub suggestion_delete_thinking_store: bool,
    pub suggestion_target_version: String,
}

impl Default for ThinkingCacheConfig {
    fn default() -> Self {
        Self {
            suggestion_delete_thinking_store: false,
            suggestion_target_version: String::new(),
        }
    }
}

/// Decision on whether to prompt the user to invalidate thinking cache.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CachePromptDecision {
    ShouldPromptUser { target_version: String },
    SkipNoPromptConfigured,
    SkipAlreadyAcknowledged { recorded_version: String },
}

/// Helper function to check if the recorded version matches the target version.
fn is_version_match(recorded_version: &str, target_version: &str) -> bool {
    let clean_recorded = recorded_version.trim().trim_start_matches('v');
    let clean_target = target_version.trim().trim_start_matches('v');
    recorded_version == target_version || clean_recorded == clean_target
}

/// Evaluates whether the user should be prompted to invalidate the thinking cache.
pub fn evaluate_cache_invalidation_prompt(
    config: &ThinkingCacheConfig,
    current_state: &UserCacheActionState,
    has_existing_cache: bool,
) -> CachePromptDecision {
    if !config.suggestion_delete_thinking_store || !has_existing_cache {
        return CachePromptDecision::SkipNoPromptConfigured;
    }

    if let UserCacheActionState::PromptedAndCleaned { version, .. }
    | UserCacheActionState::PromptedAndDismissed { version, .. } = current_state
    {
        if is_version_match(version, &config.suggestion_target_version) {
            return CachePromptDecision::SkipAlreadyAcknowledged {
                recorded_version: version.clone(),
            };
        }
    }

    CachePromptDecision::ShouldPromptUser {
        target_version: config.suggestion_target_version.clone(),
    }
}

/// Records user cache action ("clean" or any dismissal) into a new persistent state.
pub fn record_user_cache_action(
    action: &str,
    target_version: &str,
    timestamp_s: u64,
) -> UserCacheActionState {
    if action.eq_ignore_ascii_case("clean") {
        UserCacheActionState::PromptedAndCleaned {
            version: target_version.to_string(),
            timestamp_s,
        }
    } else {
        UserCacheActionState::PromptedAndDismissed {
            version: target_version.to_string(),
            timestamp_s,
        }
    }
}
