//! OpenAI non-streaming response transform (Gemini -> OpenAI).
//! Facade: implementation lives in submodules, each <= 500 lines.

pub mod sanitize;
pub mod transform;

#[cfg(test)]
mod tests;

pub use sanitize::{normalize_and_sanitize_tool_args, resolve_shell_tool_name};
pub use transform::transform_openai_response;
