//! Claude request transform (split from request.rs).
//! Facade: implementation lives in submodules, each <= 500 lines.

pub mod build_config;
pub mod build_contents;
pub mod build_parts;
pub mod messages;
pub mod safety;
pub mod thinking;
pub mod transform;

#[cfg(test)]
mod tests_a;
#[cfg(test)]
mod tests_b;
#[cfg(test)]
mod tests_c;
#[cfg(test)]
mod tests_d;

#[cfg(test)]
pub(crate) const CLAUDE_AGENT_SDK_IDENTITY: &str =
    "You are a Claude agent, built on Anthropic's Claude Agent SDK.";
#[cfg(test)]
pub(crate) const CLAUDE_CODE_CLI_IDENTITY: &str =
    "You are Claude Code, Anthropic's official CLI for Claude.";

#[cfg(test)]
pub(crate) use super::models::*;
#[cfg(test)]
pub(crate) use build_config::build_tools;
#[cfg(test)]
pub(crate) use thinking::{is_gemini_client_billing_metadata, model_supports_thinking};

// NOTE: build_contents underwent minimal extract-method (empty_tool_result_fallback,
// inject_missing_tool_results) to bring the file under 500 lines; behavior is unchanged.

pub use build_config::clean_thinking_fields_recursive;
pub use messages::{clean_cache_control_from_messages, merge_consecutive_messages};
pub(crate) use safety::{build_safety_settings, SafetyThreshold};
pub use transform::{
    transform_claude_request_in, transform_claude_request_in_timed, TransformTiming,
};
