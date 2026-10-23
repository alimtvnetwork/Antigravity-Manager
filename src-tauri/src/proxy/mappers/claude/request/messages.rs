// Message preprocessing (split from request.rs).
// Claude requested conversion (Claude → Gemini v1internal)
// Corresponds to transformClaudeRequestIn

use super::super::models::*;
use crate::proxy::mappers::signature_store::get_thought_signature; // Deprecated, kept for fallback
use crate::proxy::session_manager::SessionManager;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Normalize the standalone identity block injected by Claude Agent SDK clients.
///
/// Antigravity's upstream currently classifies this SDK identity differently from
/// Claude Code's CLI identity and can reject an otherwise identical request with
/// RESOURCE_EXHAUSTED. Keep the match exact so user-authored text that merely
/// mentions the SDK identity is not rewritten.
pub(crate) fn normalize_claude_client_identity(text: &str) -> &str {
    crate::proxy::mappers::prompt_sanitizer::PromptSanitizer::normalize_client_identity(text)
}

// ===== Safety Settings Configuration =====

/// Safety threshold levels for Gemini API
/// Can be configured via GEMINI_SAFETY_THRESHOLD environment variable

/// Clean cache_control field in message
///
/// This function will deeply traverse all message content blocks and remove the cache_control field.
/// This is necessary because:
/// 1. Clients such as VS Code will send back historical messages (including cache_control) intact.
/// 2. Anthropic API does not accept requests containing the cache_control field
/// 3. Even if it is forwarded to Gemini, it should be cleaned to maintain protocol purity.
///
/// [FIX #593] Enhanced version: Add detailed logs for debugging MCP tool compatibility issues
pub fn clean_cache_control_from_messages(messages: &mut [Message]) {
    tracing::info!(
        "[DEBUG-593] Starting cache_control cleanup for {} messages",
        messages.len()
    );

    let mut total_cleaned = 0;

    for (idx, msg) in messages.iter_mut().enumerate() {
        if let MessageContent::Array(blocks) = &mut msg.content {
            for (block_idx, block) in blocks.iter_mut().enumerate() {
                match block {
                    ContentBlock::Thinking { cache_control, .. } => {
                        if cache_control.is_some() {
                            tracing::info!(
                                "[ISSUE-744] Found cache_control in Thinking block at message[{}].content[{}]: {:?}",
                                idx,
                                block_idx,
                                cache_control
                            );
                            *cache_control = None;
                            total_cleaned += 1;
                        }
                    }
                    ContentBlock::Image { cache_control, .. } => {
                        if cache_control.is_some() {
                            tracing::debug!(
                                "[Cache-Control-Cleaner] Removed cache_control from Image block at message[{}].content[{}]",
                                idx,
                                block_idx
                            );
                            *cache_control = None;
                            total_cleaned += 1;
                        }
                    }
                    ContentBlock::Document { cache_control, .. } => {
                        if cache_control.is_some() {
                            tracing::debug!(
                                "[Cache-Control-Cleaner] Removed cache_control from Document block at message[{}].content[{}]",
                                idx,
                                block_idx
                            );
                            *cache_control = None;
                            total_cleaned += 1;
                        }
                    }
                    ContentBlock::ToolUse { cache_control, .. } => {
                        if cache_control.is_some() {
                            tracing::debug!(
                                "[Cache-Control-Cleaner] Removed cache_control from ToolUse block at message[{}].content[{}]",
                                idx,
                                block_idx
                            );
                            *cache_control = None;
                            total_cleaned += 1;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    if total_cleaned > 0 {
        tracing::info!(
            "[DEBUG-593] Cache control cleanup complete: removed {} cache_control fields",
            total_cleaned
        );
    } else {
        tracing::debug!("[DEBUG-593] No cache_control fields found");
    }
}

/// [FIX #593] Recursive deep cleaning of cache_control field in JSON
///
/// cache_control for handling nested structures and non-standard locations.
/// This is a last line of defense, ensuring that requests sent to Antigravity do not contain any cache_control.

/// [FIX #593] Recursive deep cleaning of cache_control field in JSON
///
/// cache_control for handling nested structures and non-standard locations.
/// This is a last line of defense, ensuring that requests sent to Antigravity do not contain any cache_control.
pub(crate) fn deep_clean_cache_control(value: &mut Value) {
    match value {
        Value::Object(map) => {
            if map.remove("cache_control").is_some() {
                tracing::debug!("[DEBUG-593] Removed cache_control from nested JSON object");
            }
            for (_, v) in map.iter_mut() {
                deep_clean_cache_control(v);
            }
        }
        Value::Array(arr) => {
            for item in arr.iter_mut() {
                deep_clean_cache_control(item);
            }
        }
        _ => {}
    }
}

/// [FIX #564] Sort blocks in assistant messages to ensure thinking blocks are first
///
/// When context compression (kilo) reorders message blocks, thinking blocks may appear
/// after text blocks. Claude/Anthropic API requires thinking blocks to be first if
/// any thinking blocks exist in the message. This function pre-sorts blocks to ensure
/// thinking/redacted_thinking blocks always come before other block types.

/// [FIX #564] Sort blocks in assistant messages to ensure thinking blocks are first
///
/// When context compression (kilo) reorders message blocks, thinking blocks may appear
/// after text blocks. Claude/Anthropic API requires thinking blocks to be first if
/// any thinking blocks exist in the message. This function pre-sorts blocks to ensure
/// thinking/redacted_thinking blocks always come before other block types.
pub(crate) fn sort_thinking_blocks_first(messages: &mut [Message]) {
    for msg in messages.iter_mut() {
        if msg.role == "assistant" {
            if let MessageContent::Array(blocks) = &mut msg.content {
                // [FIX #709] Triple-stage partition: [Thinking, Text, ToolUse]
                // This ensures protocol compliance while maintaining logical order.

                let mut thinking_blocks: Vec<ContentBlock> = Vec::new();
                let mut text_blocks: Vec<ContentBlock> = Vec::new();
                let mut tool_blocks: Vec<ContentBlock> = Vec::new();
                let mut other_blocks: Vec<ContentBlock> = Vec::new();

                let original_len = blocks.len();
                let mut needs_reorder = false;
                let mut saw_non_thinking = false;

                for (_i, block) in blocks.iter().enumerate() {
                    match block {
                        ContentBlock::Thinking { .. } | ContentBlock::RedactedThinking { .. } => {
                            if saw_non_thinking {
                                needs_reorder = true;
                            }
                        }
                        ContentBlock::Text { .. } => {
                            saw_non_thinking = true;
                        }
                        ContentBlock::ToolUse { .. } => {
                            saw_non_thinking = true;
                            // Check if tool is after text (this is normal, but we want a strict group order)
                        }
                        _ => saw_non_thinking = true,
                    }
                }

                if needs_reorder || original_len > 1 {
                    // For safety, we always perform the triple partition if there's more than one block.
                    // This also handles empty text block filtering.
                    for block in blocks.drain(..) {
                        match &block {
                            ContentBlock::Thinking { .. }
                            | ContentBlock::RedactedThinking { .. } => {
                                thinking_blocks.push(block);
                            }
                            ContentBlock::Text { text } => {
                                // Filter out purely empty or structural text like "(no content)"
                                if !text.trim().is_empty() && text != "(no content)" {
                                    text_blocks.push(block);
                                }
                            }
                            ContentBlock::ToolUse { .. } => {
                                tool_blocks.push(block);
                            }
                            _ => {
                                other_blocks.push(block);
                            }
                        }
                    }

                    // Reconstruct in strict order: Thinking -> Text/Other -> Tool
                    blocks.extend(thinking_blocks);
                    blocks.extend(text_blocks);
                    blocks.extend(other_blocks);
                    blocks.extend(tool_blocks);

                    if needs_reorder {
                        tracing::warn!(
                            "[FIX #709] Reordered assistant messages to [Thinking, Text, Tool] structure."
                        );
                    }
                }
            }
        }
    }
}

/// Merge consecutive messages of the same role in ClaudeRequest
///
/// Scenario: When switching back to encoding mode from Spec/Plan mode, two consecutive "user" messages may appear
/// (One is ToolResult and one is <system-reminder>).
/// This would violate the role alternation rules and result in a 400 error.

/// Merge consecutive messages of the same role in ClaudeRequest
///
/// Scenario: When switching back to encoding mode from Spec/Plan mode, two consecutive "user" messages may appear
/// (One is ToolResult and one is <system-reminder>).
/// This would violate the role alternation rules and result in a 400 error.
pub fn merge_consecutive_messages(messages: &mut Vec<Message>) {
    if messages.len() <= 1 {
        return;
    }

    let mut merged: Vec<Message> = Vec::with_capacity(messages.len());
    let old_messages = std::mem::take(messages);
    let mut messages_iter = old_messages.into_iter();

    if let Some(mut current) = messages_iter.next() {
        for next in messages_iter {
            if current.role == next.role {
                // Merge content
                match (&mut current.content, next.content) {
                    (MessageContent::Array(current_blocks), MessageContent::Array(next_blocks)) => {
                        current_blocks.extend(next_blocks);
                    }
                    (MessageContent::Array(current_blocks), MessageContent::String(next_text)) => {
                        current_blocks.push(ContentBlock::Text { text: next_text });
                    }
                    (MessageContent::String(current_text), MessageContent::String(next_text)) => {
                        *current_text = format!("{}\n\n{}", current_text, next_text);
                    }
                    (MessageContent::String(current_text), MessageContent::Array(next_blocks)) => {
                        let mut new_blocks = vec![ContentBlock::Text {
                            text: current_text.clone(),
                        }];
                        new_blocks.extend(next_blocks);
                        current.content = MessageContent::Array(new_blocks);
                    }
                }
            } else {
                merged.push(current);
                current = next;
            }
        }
        merged.push(current);
    }

    *messages = merged;
}

// Convert Claude request to Gemini v1internal format
/// [FIX #709] Reorder serialized Gemini parts to ensure thinking blocks are first,
/// deduplicate multiple thinking blocks to at most ONE per model turn, and clean out raw dot placeholders.

/// [FIX #709] Reorder serialized Gemini parts to ensure thinking blocks are first,
/// deduplicate multiple thinking blocks to at most ONE per model turn, and clean out raw dot placeholders.
pub(crate) fn reorder_gemini_parts(parts: &mut Vec<Value>) {
    if parts.len() <= 1 {
        return;
    }

    let mut thinking_parts = Vec::new();
    // Non-thinking components **maintain their original relative order**.
    //
    // The historical implementation is rearranged according to `[text] → [other] → [tool]`, squeezing `functionCall` to the end:
    // The same conversation will produce different `contents` through Claude and OpenAI/Gemini.
    // (The prefix cache hash is bound to miss), and the signature anchor is moved on the text / tool shuffling wheel.
    // The order of parts is the anchor semantics and cannot be rewritten except "thinking blocks are placed first".
    let mut rest_parts = Vec::new();

    for part in parts.drain(..) {
        if part.get("thought").and_then(|t| t.as_bool()) == Some(true) {
            thinking_parts.push(part);
        } else if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
            let t = text.trim();
            // Filter empty / dummy text parts that might have been created during merging or from client dot echo
            if !t.is_empty() && t != "(no content)" && t != "·" {
                rest_parts.push(part);
            }
        } else {
            rest_parts.push(part);
        }
    }

    // Keep at most ONE thinking block per model turn (the one with longest text or real signature)
    if thinking_parts.len() > 1 {
        let best_thought = thinking_parts.into_iter().max_by_key(|p| {
            let sig_len = p
                .get("thoughtSignature")
                .or_else(|| p.get("thought_signature"))
                .and_then(|s| s.as_str())
                .map(|s| s.len())
                .unwrap_or(0);
            let text_len = p
                .get("text")
                .and_then(|t| t.as_str())
                .map(|t| t.len())
                .unwrap_or(0);
            (sig_len, text_len)
        });
        if let Some(t) = best_thought {
            parts.push(t);
        }
    } else if let Some(t) = thinking_parts.pop() {
        parts.push(t);
    }

    parts.extend(rest_parts);
}
