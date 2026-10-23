// Content building (split from request.rs).
// Claude requested conversion (Claude → Gemini v1internal)
// Corresponds to transformClaudeRequestIn

use super::super::models::*;
use crate::proxy::mappers::common_utils::request::is_model_compatible;
use crate::proxy::mappers::signature_store::get_thought_signature; // Deprecated, kept for fallback
use crate::proxy::session_manager::SessionManager;
use serde_json::{json, Value};
use std::collections::HashMap;

use super::build_parts::{
    build_google_content, empty_tool_result_fallback, inject_missing_tool_results,
};
use crate::modules::proxy_db::signatures::SENTINEL_SIGNATURE;
use crate::proxy::mappers::claude::thinking_utils::MIN_SIGNATURE_LENGTH;

/// Build Contents (Messages)
pub(crate) fn build_contents(
    content: &MessageContent,
    is_assistant: bool,
    _claude_req: &ClaudeRequest,
    is_thinking_enabled: bool,
    _session_id: &str,
    _msg_index: usize,
    _allow_dummy_thought: bool,
    is_retry: bool,
    tool_id_to_name: &mut HashMap<String, String>,
    tool_name_to_schema: &HashMap<String, Value>,
    mapped_model: &str,
    last_thought_signature: &mut Option<String>,
    pending_tool_use_ids: &mut Vec<String>,
    last_user_task_text_normalized: &mut Option<String>,
    previous_was_tool_result: &mut bool,
    _existing_tool_result_ids: &std::collections::HashSet<String>,
) -> Result<Vec<Value>, String> {
    let mut parts = Vec::new();
    // Track tool results in the current turn to identify missing ones
    let mut current_turn_tool_result_ids = std::collections::HashSet::new();

    // Pre-scan for this assistant turn's signature to keep each turn strictly isolated
    let mut turn_signature: Option<String> = None;
    if is_assistant {
        if let MessageContent::Array(blocks) = content {
            for b in blocks {
                match b {
                    ContentBlock::Thinking {
                        signature: Some(s), ..
                    } => {
                        if (s == SENTINEL_SIGNATURE || s.len() >= MIN_SIGNATURE_LENGTH)
                            && (!mapped_model.to_lowercase().contains("gemini")
                                || crate::proxy::thinking_store::is_likely_gemini_signature(s))
                        {
                            turn_signature = Some(s.clone());
                            break;
                        }
                    }
                    ContentBlock::ToolUse { signature, .. } => {
                        if let Some(s) = signature {
                            if (s == SENTINEL_SIGNATURE || s.len() >= MIN_SIGNATURE_LENGTH)
                                && (!mapped_model.to_lowercase().contains("gemini")
                                    || crate::proxy::thinking_store::is_likely_gemini_signature(s))
                            {
                                turn_signature = Some(s.clone());
                                break;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Track if we have already seen non-thinking content in this message.
    // Anthropic/Gemini protocol: Thinking blocks MUST come first.
    let mut saw_non_thinking = false;

    match content {
        MessageContent::String(text) => {
            if text != "(no content)" {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    parts.extend(
                        crate::proxy::mappers::common_utils::parse_markdown_images_to_parts(
                            trimmed,
                        ),
                    );
                }
            }
        }
        MessageContent::Array(blocks) => {
            for item in blocks {
                match item {
                    ContentBlock::Text { text } => {
                        if text != "(no content)" && !text.trim().is_empty() {
                            // [NEW] Task deduplication logic: If the current message is a User message and it immediately follows the ToolResult,
                            // Check that the text is exactly the same as the previous round's task description.
                            if !is_assistant && *previous_was_tool_result {
                                if let Some(last_task) = last_user_task_text_normalized {
                                    let current_normalized =
                                        text.replace(|c: char| c.is_whitespace(), "");
                                    if !current_normalized.is_empty()
                                        && current_normalized == *last_task
                                    {
                                        tracing::info!("[Claude-Request] Dropping duplicated task text echo (len: {})", text.len());
                                        continue;
                                    }
                                }
                            }

                            parts.extend(
                                crate::proxy::mappers::common_utils::parse_markdown_images_to_parts(
                                    text,
                                ),
                            );
                            saw_non_thinking = true;

                            // Record the latest User task text for subsequent comparison
                            if !is_assistant {
                                *last_user_task_text_normalized =
                                    Some(text.replace(|c: char| c.is_whitespace(), ""));
                            }
                            *previous_was_tool_result = false;
                        }
                    }
                    ContentBlock::Thinking {
                        thinking,
                        signature,
                        ..
                    } => {
                        tracing::debug!(
                            "[DEBUG-TRANSFORM] Processing thinking block. Sig: {:?}",
                            signature
                        );

                        // [2026-09-27] Placeholder/empty thinking blocks are discarded directly (thinking blocks are not written, and are not downgraded to text).
                        // The official sample (baogao.txt) on round 9/24 is "no thinking block + anchor with signature",
                        // Empty / "." / "..." / Space / "·" and other placeholder thinking have no information; return the signature
                        // The final place_turn_signature of the pipeline is processed according to anchor rules (inbound has
                        // Transfer the signature of the placeholder block to the first non-thinking part).
                        let is_placeholder =
                            crate::proxy::thinking_store::is_placeholder_thought(thinking);
                        if is_placeholder || thinking.trim().is_empty() {
                            tracing::debug!(
                                "[Claude-Request] Placeholder thinking block dropped (text={:?}).",
                                &thinking[..thinking.len().min(20)],
                            );
                            continue;
                        }
                        let final_thought_text = thinking.as_str();

                        // [HOTFIX] Gemini Protocol Enforcement: Thinking block MUST be the first block.
                        // If we already have content (like Text), we must downgrade this thinking block to Text.
                        if saw_non_thinking || !parts.is_empty() {
                            tracing::warn!("[Claude-Request] Thinking block found at non-zero index (prev parts: {}). Downgrading to Text.", parts.len());
                            if !final_thought_text.is_empty() {
                                parts.push(json!({
                                    "text": final_thought_text
                                }));
                                saw_non_thinking = true;
                            }
                            continue;
                        }

                        // [FIX] If thinking is disabled (smart downgrade), convert ALL thinking blocks to text
                        // to avoid "thinking is disabled but message contains thinking" error
                        if !is_thinking_enabled {
                            tracing::warn!("[Claude-Request] Thinking disabled. Downgrading thinking block to text.");
                            if !final_thought_text.is_empty() {
                                parts.push(json!({
                                    "text": final_thought_text
                                }));
                                saw_non_thinking = true;
                            }
                            continue;
                        }

                        let is_claude_model = mapped_model.to_lowercase().contains("claude");

                        let mut effective_sig = None;

                        // 1. Check incoming signature if long enough or sentinel
                        if let Some(sig) = signature {
                            if sig == SENTINEL_SIGNATURE || sig.len() >= MIN_SIGNATURE_LENGTH {
                                let cached_family = crate::proxy::SignatureCache::global()
                                    .get_signature_family(sig);

                                match cached_family {
                                    Some(family) => {
                                        let compatible =
                                            !is_retry && is_model_compatible(&family, mapped_model);
                                        if compatible
                                            || (!is_retry
                                                && is_claude_model
                                                && family.to_lowercase().contains("claude"))
                                        {
                                            effective_sig = Some(sig.clone());
                                        } else {
                                            tracing::warn!(
                                                "[Thinking-Signature] {} signature (Family: {}, Target: {}).",
                                                if is_retry { "Stripping historical" } else { "Incompatible" },
                                                family, mapped_model
                                            );
                                        }
                                    }
                                    None => {
                                        if !is_retry {
                                            if mapped_model.to_lowercase().contains("gemini") {
                                                if crate::proxy::thinking_store::is_likely_gemini_signature(sig) {
                                                    effective_sig = Some(sig.clone());
                                                } else {
                                                    tracing::warn!(
                                                        "[Thinking-Signature] Dropping unknown/foreign signature for Gemini target (len: {}), fallback to sentinel",
                                                        sig.len()
                                                    );
                                                }
                                            } else {
                                                effective_sig = Some(sig.clone());
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // 2. Try turn_signature (strictly local to this assistant turn)
                        if effective_sig.is_none() {
                            effective_sig = turn_signature.clone();
                        }

                        // 3. **Never create a sentinel** without a legal signature.
                        //    Basis (3 official messages / 23 signatures): Sentinel appears 0/23 times in official traffic,
                        //    It does not belong to the Antigravity protocol; and "skipping verification" does not mean restoring reasoning continuity.
                        //    Lack of signatures is tolerated by upstream (official "on flywheel" means absent),
                        //    The return is uniformly handed over to the assembly line for final review `place_turn_signature`.
                        if let Some(sig) = effective_sig {
                            *last_thought_signature = Some(sig.clone());
                            let part = json!({
                                "text": final_thought_text,
                                "thought": true,
                                "thoughtSignature": sig
                            });
                            parts.push(part);
                        } else {
                            // No signature: Keep the thought block structure (`thought: true`), but **do not carry the signature field**.
                            // Think blocks for Gemini targets should not be signed (iron rule I4:
                            // There is no thoughtSignature in the part of `thought:true` in the official message).
                            tracing::debug!(
                                "[Thinking-Signature] No signature for thought block (model: {}). Keeping thought block without signature.",
                                mapped_model
                            );
                            parts.push(json!({
                                "text": final_thought_text,
                                "thought": true,
                            }));
                        }
                    }
                    ContentBlock::RedactedThinking { data } => {
                        // [FIX] Handle RedactedThinking as normal text, preserving context
                        tracing::debug!("[Claude-Request] Degrade RedactedThinking to text");
                        parts.push(json!({
                            "text": format!("[Redacted Thinking: {}]", data)
                        }));
                        saw_non_thinking = true;
                        continue;
                    }
                    ContentBlock::Image { source, .. } => {
                        if source.source_type == "base64" {
                            let part =
                                crate::proxy::mappers::common_utils::create_gemini_inline_part(
                                    source.media_type.as_deref(),
                                    source.data.as_deref().unwrap_or_default(),
                                    "Image",
                                );
                            parts.push(part);
                            saw_non_thinking = true;
                        }
                    }
                    ContentBlock::Document { source, .. } => {
                        if source.source_type == "base64" {
                            let part =
                                crate::proxy::mappers::common_utils::create_gemini_inline_part(
                                    source.media_type.as_deref(),
                                    source.data.as_deref().unwrap_or_default(),
                                    "Document",
                                );
                            parts.push(part);
                            saw_non_thinking = true;
                        }
                    }
                    ContentBlock::ToolUse {
                        id,
                        name,
                        input,
                        signature,
                        ..
                    } => {
                        let mut final_input = input.clone();

                        // [New] Utilize universal engine to correct parameter types (replacing the previous hard-coded shell tool repair logic)
                        if let Some(original_schema) = tool_name_to_schema.get(name) {
                            crate::proxy::common::json_schema::fix_tool_call_args(
                                &mut final_input,
                                original_schema,
                            );
                        }

                        let mut part = json!({
                            "functionCall": {
                                "name": name,
                                "args": final_input,
                                "id": id
                            }
                        });
                        saw_non_thinking = true;

                        // Track pending tool use
                        if is_assistant {
                            pending_tool_use_ids.push(id.clone());
                        }

                        // Record id -> name mapping and signing context
                        tool_id_to_name.insert(id.clone(), name.clone());
                        let final_sig = signature
                            .as_ref()
                            .filter(|s| {
                                (s.as_str() == SENTINEL_SIGNATURE
                                    || s.len() >= MIN_SIGNATURE_LENGTH)
                                    && (!mapped_model.to_lowercase().contains("gemini")
                                        || crate::proxy::thinking_store::is_likely_gemini_signature(
                                            s,
                                        ))
                            })
                            .cloned();

                        if let Some(ref s) = final_sig {
                            *last_thought_signature = Some(s.clone());
                        }

                        let is_claude_model = mapped_model.to_lowercase().contains("claude");
                        if is_claude_model {
                            // Claude model: Anthropic's official signature verification engine requires that the signature must be on the thinking block, and tool calls will never carry signatures, let alone fake sentinels.
                            if let Some(obj) = part.as_object_mut() {
                                obj.remove("thoughtSignature");
                                obj.remove("thought_signature");
                            }
                        } else {
                            // Pure cable transparent transmission: if the client has its own signature, keep it, if not, leave it blank. Fully authorized inbound pipeline unified alignment and backfilling
                            if let Some(ref sig) = final_sig {
                                part["thoughtSignature"] = json!(sig);
                            }
                        }
                        parts.push(part);
                    }
                    ContentBlock::ToolResult {
                        tool_use_id,
                        content,
                        is_error,
                        ..
                    } => {
                        // Mark this tool ID as resolved in this turn
                        current_turn_tool_result_ids.insert(tool_use_id.clone());
                        // Priority is given to using the previously recorded name, otherwise tool_use_id is used.
                        let func_name = tool_id_to_name
                            .get(tool_use_id)
                            .cloned()
                            .unwrap_or_else(|| tool_use_id.clone());

                        // Tool results should pass transparency. If images are present, map them to inlineData.
                        let mut extra_parts = Vec::new();

                        let mut merged_content = match content {
                            serde_json::Value::String(s) => {
                                crate::proxy::mappers::common_utils::extract_multimodal_from_tool_text(
                                    s,
                                    &mut extra_parts,
                                )
                            }
                            serde_json::Value::Array(arr) => {
                                let mut texts = Vec::new();
                                for block in arr {
                                    if let Some(text) = block.get("text").and_then(|v| v.as_str()) {
                                        texts.push(text.to_string());
                                    } else if block.get("source").is_some() {
                                        if block.get("type").and_then(|v| v.as_str())
                                            == Some("image")
                                        {
                                            let source = block.get("source").unwrap();
                                            let media_type =
                                                source.get("media_type").and_then(|v| v.as_str());
                                            let data = source
                                                .get("data")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or_default();
                                            extra_parts.push(crate::proxy::mappers::common_utils::create_gemini_inline_part(
                                                media_type,
                                                data,
                                                "Tool Result Image",
                                            ));
                                        }
                                    }
                                }
                                texts.join("\n")
                            }
                            _ => content.to_string(),
                        };

                        // Smart Truncation: max chars limit
                        const MAX_TOOL_RESULT_CHARS: usize = 200_000;
                        if merged_content.len() > MAX_TOOL_RESULT_CHARS {
                            tracing::warn!(
                                "Truncating tool result from {} chars to {}",
                                merged_content.len(),
                                MAX_TOOL_RESULT_CHARS
                            );
                            let mut truncated = merged_content
                                .chars()
                                .take(MAX_TOOL_RESULT_CHARS)
                                .collect::<String>();
                            truncated.push_str("\n...[truncated output]");
                            merged_content = truncated;
                        }

                        // [Optimization] If the result is empty, inject an explicit confirmation signal to prevent model hallucinations
                        merged_content = empty_tool_result_fallback(&merged_content, *is_error);

                        let part = json!({
                            "functionResponse": {
                                "name": func_name,
                                "response": {"result": merged_content},
                                "id": tool_use_id
                            }
                        });

                        // Dangerous test branch rule: ToolResult (functionResponse) never carries a signature
                        parts.push(part);

                        // Add image parts
                        for extra in extra_parts {
                            parts.push(extra);
                        }

                        // Mark status, used for deduplication judgment of the next User message
                        *previous_was_tool_result = true;
                    }
                    // ContentBlock::RedactedThinking handled above at line 583
                    ContentBlock::ServerToolUse { .. }
                    | ContentBlock::WebSearchToolResult { .. } => {
                        // Search result blocks should not be sent back to upstream by the client (replaced by tool_result)
                        continue;
                    }
                }
            }
        }
    }

    inject_missing_tool_results(
        &mut parts,
        is_assistant,
        pending_tool_use_ids,
        &current_turn_tool_result_ids,
        tool_id_to_name,
    );

    // Fix for "Thinking enabled, assistant message must start with thinking block" 400 error
    // [Optimization] Apply this to ALL assistant messages in history, not just the last one.
    // Vertex AI requires every assistant message to start with a thinking block when thinking is enabled.
    if is_assistant && is_thinking_enabled {
        let _is_google_cloud = mapped_model.starts_with("projects/");
        let thought_idx = parts
            .iter()
            .position(|p| p.get("thought").and_then(|v| v.as_bool()) == Some(true));

        match thought_idx {
            Some(0) => {
                // Already at index 0! Nothing to do.
            }
            Some(idx) => {
                // Existing thought block is not at index 0, move it to index 0 (DO NOT insert a duplicate!)
                let thought_part = parts.remove(idx);
                parts.insert(0, thought_part);
            }
            None => {
                // Pure cable principle: If the client does not originally have a thinking block, the adapter is strictly prohibited from forging "..." placeholder blocks out of thin air!
                // The determination of missing thinking blocks and state machine resurrection are collectively entrusted to the InboundThinkingPipeline.
            }
        }
    }

    Ok(parts)
}
