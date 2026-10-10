//! Claude non-streaming response transform (Gemini -> Claude).
//! Facade: implementation lives in submodules, each <= 500 lines.

pub mod processor;
pub mod transform;

pub use processor::NonStreamingProcessor;
pub use transform::transform_response;
