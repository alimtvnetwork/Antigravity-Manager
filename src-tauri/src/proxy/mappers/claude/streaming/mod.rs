//! Claude streaming response transform (Gemini SSE -> Claude SSE).
//! Facade: implementation lives in submodules, each <= 500 lines.

pub mod processor;
pub mod processor_text;
pub mod processor_tools;
pub mod state;
pub mod types;

#[cfg(test)]
mod tests;

pub use processor::PartProcessor;
pub use state::StreamingState;
pub use types::{remap_function_call_args, BlockType, SignatureManager};
