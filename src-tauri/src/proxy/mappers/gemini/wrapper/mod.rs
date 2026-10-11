//! Gemini request/response wrapper (split from wrapper.rs).
//! Facade: implementation lives in submodules, each <= 500 lines.
//!
//! NOTE: wrap_request_v2 underwent behavior-preserving phase extraction
//! (setup/compression/prep/contents/thinking/finalize) to bring files under 500 lines.

pub mod prompts;
pub mod request;
pub mod request_v2;
pub mod response;
pub mod v2_compression;
pub mod v2_contents;
pub mod v2_finalize;
pub mod v2_prep;
pub mod v2_thinking;

#[cfg(test)]
mod tests_a;
#[cfg(test)]
mod tests_b;

pub use prompts::{CONTEXT_SUMMARY_PROMPT, INTERNAL_BACKGROUND_TASK, SUMMARY_REQUEST_TIMEOUT_SECS};
pub use request::wrap_request;
pub use request_v2::wrap_request_v2;
pub use response::{inject_ids_to_response, unwrap_response};

#[cfg(test)]
pub(crate) use request::TEST_MUTEX;
