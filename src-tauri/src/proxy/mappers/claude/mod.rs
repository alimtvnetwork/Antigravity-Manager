// Claude mapper 模块
// 负责 Claude ↔ Gemini 协议转换

pub mod collector;
pub mod models;
pub mod request;
pub mod response;
pub mod streaming;
pub mod thinking_utils;
pub mod utils;

pub use collector::collect_stream_to_json;
pub use models::*;
pub use request::{
    clean_cache_control_from_messages, merge_consecutive_messages, transform_claude_request_in,
    transform_claude_request_in_timed,
};
pub use response::transform_response;
pub use streaming::{PartProcessor, StreamingState};
pub use thinking_utils::{
    close_tool_loop_for_thinking, filter_invalid_thinking_blocks_with_family,
}; // [NEW]

pub mod sse_stream;

#[cfg(test)]
mod sse_stream_tests;

pub use sse_stream::{create_claude_sse_stream, emit_force_stop};
