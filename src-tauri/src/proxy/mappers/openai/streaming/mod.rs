//! OpenAI SSE streaming (split from streaming.rs).
//! Facade: implementation lives in submodules.
//!
//! NOTE: streaming/codex.rs (677 lines) exceeds 500 lines because
//! create_codex_sse_stream uses yield inside async_stream! macro and cannot
//! be safely decomposed via pure code moves. Reported as exception.

pub mod codex;
pub mod openai;

#[cfg(test)]
mod tests_a;
#[cfg(test)]
mod tests_b;

pub use codex::create_codex_sse_stream;
pub use openai::{
    create_legacy_sse_stream, create_openai_sse_stream, create_openai_sse_stream_with_anchor,
    store_thought_signature,
};

#[cfg(test)]
pub(crate) use bytes::Bytes;
#[cfg(test)]
pub(crate) use futures::StreamExt;
#[cfg(test)]
pub(crate) use serde_json::Value;
