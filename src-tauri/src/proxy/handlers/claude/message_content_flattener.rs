//! Claude Message Content & Multimodal Block Flattener
//!
//! Normalizes heterogeneous Claude message contents (single text string, arrays of
//! text blocks, image blocks, tool_use blocks, or thinking blocks) into clean,
//! canonical flat representations for pipeline processing and token accounting.

use serde_json::Value;
use tracing::trace;

/// Extracted and flattened message content details.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FlattenedContent {
    /// Combined text extracted from all text blocks.
    pub combined_text: String,
    /// Count of image blocks present.
    pub image_block_count: usize,
    /// Count of tool use blocks present.
    pub tool_use_count: usize,
    /// Count of tool result blocks present.
    pub tool_result_count: usize,
    /// Count of thinking blocks present.
    pub thinking_block_count: usize,
    /// Total number of distinct content blocks.
    pub total_blocks: usize,
}

/// Flattens a Claude message `content` JSON value into a canonical summary and combined string.
pub fn flatten_claude_content(content: &Value) -> FlattenedContent {
    let mut flattened = FlattenedContent::default();

    match content {
        Value::String(s) => {
            flattened.combined_text = s.clone();
            flattened.total_blocks = 1;
        }
        Value::Array(blocks) => {
            flattened.total_blocks = blocks.len();
            let mut text_parts = Vec::new();

            for block in blocks {
                if let Some(block_type) = block.get("type").and_then(|t| t.as_str()) {
                    match block_type {
                        "text" => {
                            if let Some(t) = block.get("text").and_then(|s| s.as_str()) {
                                text_parts.push(t);
                            }
                        }
                        "image" => {
                            flattened.image_block_count += 1;
                        }
                        "tool_use" => {
                            flattened.tool_use_count += 1;
                        }
                        "tool_result" => {
                            flattened.tool_result_count += 1;
                            if let Some(res) = block.get("content").and_then(|c| c.as_str()) {
                                text_parts.push(res);
                            }
                        }
                        "thinking" => {
                            flattened.thinking_block_count += 1;
                        }
                        _ => {}
                    }
                }
            }

            flattened.combined_text = text_parts.join("\n");
        }
        _ => {}
    }

    trace!(
        text_len = flattened.combined_text.len(),
        images = flattened.image_block_count,
        tools = flattened.tool_use_count,
        "Flattened Claude message content"
    );

    flattened
}

/// Normalizes content to an array of objects if it is currently a raw string.
pub fn ensure_content_block_array(content: &Value) -> Value {
    match content {
        Value::String(s) => {
            serde_json::json!([
                {
                    "type": "text",
                    "text": s
                }
            ])
        }
        Value::Array(_) => content.clone(),
        _ => serde_json::json!([]),
    }
}
