//! OpenAI request transform (split from request.rs).
//! Facade: implementation lives in submodules, each <= 500 lines.
//!
//! NOTE: transform_openai_request_with_session underwent behavior-preserving
//! phase extraction (setup/system/contents/body/tools/finalize) to bring files under 500 lines.

pub mod helpers;
pub mod session;
pub mod session_body;
pub mod session_contents;
pub mod session_finalize;
pub mod session_setup;
pub mod session_system;
pub mod session_tools;
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
mod tests_e;

pub(crate) use helpers::{extract_client_tool_names, is_tiered_flash_model};
pub use session::transform_openai_request_with_session;
pub use transform::{enforce_uppercase_types, transform_openai_request};
