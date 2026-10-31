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

/// Checks if a role string is a recognized Claude protocol role.
pub fn is_valid_claude_role(role: &str) -> bool {
    let r = role.trim().to_ascii_lowercase();
    matches!(r.as_str(), "user" | "assistant" | "system")
}

/// Checks if a content block type represents raw or structured text.
pub fn is_text_block(block_type: &str) -> bool {
    block_type == "text"
}

/// Checks if a content block type represents tool invocation or execution output.
pub fn is_tool_block(block_type: &str) -> bool {
    block_type == "tool_use" || block_type == "tool_result"
}

/// Checks if a content block type represents internal reasoning or thinking.
pub fn is_thinking_block(block_type: &str) -> bool {
    block_type == "thinking"
}

/// Checks if a content block type represents rich multimodal data (images, documents).
pub fn is_multimodal_block(block_type: &str) -> bool {
    block_type == "image" || block_type == "document"
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
                let Some(block_type) = block.get("type").and_then(|t| t.as_str()) else {
                    continue;
                };

                if is_text_block(block_type) {
                    if let Some(t) = block.get("text").and_then(|s| s.as_str()) {
                        text_parts.push(t);
                    }
                } else if block_type == "image" {
                    flattened.image_block_count += 1;
                } else if block_type == "tool_use" {
                    flattened.tool_use_count += 1;
                } else if block_type == "tool_result" {
                    flattened.tool_result_count += 1;
                    if let Some(res) = block.get("content").and_then(|c| c.as_str()) {
                        text_parts.push(res);
                    }
                } else if is_thinking_block(block_type) {
                    flattened.thinking_block_count += 1;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_claude_role() {
        assert!(is_valid_claude_role("user"));
        assert!(is_valid_claude_role("User"));
        assert!(is_valid_claude_role("ASSISTANT"));
        assert!(is_valid_claude_role("system"));
        assert!(!is_valid_claude_role("bot"));
        assert!(!is_valid_claude_role("admin"));
    }

    #[test]
    fn test_content_block_predicates() {
        assert!(is_text_block("text"));
        assert!(!is_text_block("image"));

        assert!(is_tool_block("tool_use"));
        assert!(is_tool_block("tool_result"));
        assert!(!is_tool_block("text"));

        assert!(is_thinking_block("thinking"));
        assert!(!is_thinking_block("tool_use"));

        assert!(is_multimodal_block("image"));
        assert!(is_multimodal_block("document"));
        assert!(!is_multimodal_block("text"));
    }

    #[test]
    fn test_flatten_claude_content_string() {
        let val = serde_json::json!("Hello Claude!");
        let flattened = flatten_claude_content(&val);
        assert_eq!(flattened.combined_text, "Hello Claude!");
        assert_eq!(flattened.total_blocks, 1);
        assert_eq!(flattened.image_block_count, 0);
    }

    #[test]
    fn test_flatten_claude_content_array() {
        let val = serde_json::json!([
            {"type": "text", "text": "Part 1"},
            {"type": "image"},
            {"type": "tool_use"},
            {"type": "text", "text": "Part 2"}
        ]);
        let flattened = flatten_claude_content(&val);
        assert_eq!(flattened.combined_text, "Part 1\nPart 2");
        assert_eq!(flattened.total_blocks, 4);
        assert_eq!(flattened.image_block_count, 1);
        assert_eq!(flattened.tool_use_count, 1);
    }
}
