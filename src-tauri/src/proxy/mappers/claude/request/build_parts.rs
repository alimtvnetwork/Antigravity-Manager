// Content part builders (split from request.rs).
// Claude 请求转换 (Claude → Gemini v1internal)
// 对应 transformClaudeRequestIn

use super::super::models::*;
use crate::proxy::mappers::signature_store::get_thought_signature; // Deprecated, kept for fallback
use crate::proxy::session_manager::SessionManager;
use serde_json::{json, Value};
use std::collections::HashMap;

use super::build_contents::build_contents;
use super::messages::normalize_claude_client_identity;
use super::messages::reorder_gemini_parts;
use super::thinking::{clean_system_prompt_text, is_gemini_client_billing_metadata};
use super::transform::TransformTiming;

/// 构建 System Instruction (支持动态身份映射与 Prompt 隔离)
pub(crate) fn build_system_instruction(
    system: &Option<SystemPrompt>,
    model_name: &str,
    extra_system_messages: &[String],
) -> Option<Value> {
    let mut parts = Vec::new();

    // 不注入官方 Antigravity 身份：客户端自带 system prompt 时原样透传。
    // 全局提示词仍做清洗 + 换行隔离，避免 Markdown 粘连；身份文本若混入则由 clean_system_prompt_text 剥掉。
    let global_prompt_config = crate::proxy::config::get_global_system_prompt();
    if global_prompt_config.enabled && !global_prompt_config.content.trim().is_empty() {
        let cleaned = clean_system_prompt_text(&global_prompt_config.content);
        if !cleaned.is_empty() {
            parts.push(json!({"text": format!("{}\n\n", cleaned)}));
        }
    }

    // 添加用户的系统提示词
    if let Some(sys) = system {
        match sys {
            SystemPrompt::String(text) => {
                // [Issue #3452] 过滤 Claude Desktop 注入的单行计费元数据，防止与大量工具组合时触发 Google 上游 429 RESOURCE_EXHAUSTED
                if !is_gemini_client_billing_metadata(model_name, text) {
                    let norm = normalize_claude_client_identity(text);
                    let cleaned = clean_system_prompt_text(norm);
                    if !cleaned.is_empty() {
                        parts.push(json!({"text": cleaned}));
                    }
                }
            }
            SystemPrompt::Array(blocks) => {
                for block in blocks {
                    if block.block_type == "text" {
                        // [Issue #3452] 过滤 Claude Desktop 注入的单行计费元数据
                        if is_gemini_client_billing_metadata(model_name, &block.text) {
                            continue;
                        }
                        let norm = normalize_claude_client_identity(&block.text);
                        let cleaned = clean_system_prompt_text(norm);
                        if !cleaned.is_empty() {
                            parts.push(json!({
                                "text": cleaned
                            }));
                        }
                    }
                }
            }
        }
    }

    // 3. 添加提取出来的 role == "system" 消息
    for extra_text in extra_system_messages {
        if is_gemini_client_billing_metadata(model_name, extra_text) {
            continue;
        }
        let cleaned = clean_system_prompt_text(extra_text);
        if !cleaned.is_empty() {
            parts.push(json!({"text": format!("\n{}", cleaned)}));
        }
    }

    if parts.is_empty() {
        return None;
    }

    Some(json!({
        "role": "user",
        "parts": parts
    }))
}

pub(crate) fn build_google_content(
    msg: &Message,
    claude_req: &ClaudeRequest,
    is_thinking_enabled: bool,
    session_id: &str,
    msg_index: usize,
    allow_dummy_thought: bool,
    is_retry: bool,
    tool_id_to_name: &mut HashMap<String, String>,
    tool_name_to_schema: &HashMap<String, Value>,
    mapped_model: &str,
    last_thought_signature: &mut Option<String>,
    pending_tool_use_ids: &mut Vec<String>,
    last_user_task_text_normalized: &mut Option<String>,
    previous_was_tool_result: &mut bool,
    existing_tool_result_ids: &std::collections::HashSet<String>,
) -> Result<Value, String> {
    let role = if msg.role == "assistant" {
        "model"
    } else {
        &msg.role
    };

    // Proactive Tool Chain Repair:
    // If we are about to process an Assistant message, but we still have pending tool_use_ids,
    // it means the previous turn was interrupted or the user ignored the tool.
    // We MUST inject a synthetic User message with error results to close the loop.
    if role == "model" && !pending_tool_use_ids.is_empty() {
        tracing::warn!("[Elastic-Recovery] Detected interrupted tool chain (Assistant -> Assistant). Injecting synthetic User message for IDs: {:?}", pending_tool_use_ids);

        let synthetic_parts: Vec<serde_json::Value> = pending_tool_use_ids
            .iter()
            .filter(|id| !existing_tool_result_ids.contains(*id)) // [FIX #632] Only inject if ID is truly missing
            .map(|id| {
                let name = tool_id_to_name.get(id).cloned().unwrap_or(id.clone());
                json!({
                    "functionResponse": {
                        "name": name,
                        "response": {
                            "result": "Tool execution interrupted. No result provided."
                        },
                        "id": id
                    }
                })
            })
            .collect();

        if !synthetic_parts.is_empty() {
            return Ok(json!({
                "role": "user",
                "parts": synthetic_parts
            }));
        }
        // Clear pending IDs as we have handled them
        pending_tool_use_ids.clear();
    }

    let mut parts = build_contents(
        &msg.content,
        msg.role == "assistant",
        claude_req,
        is_thinking_enabled,
        session_id,
        msg_index,
        allow_dummy_thought,
        is_retry,
        tool_id_to_name,
        tool_name_to_schema,
        mapped_model,
        last_thought_signature,
        pending_tool_use_ids,
        last_user_task_text_normalized,
        previous_was_tool_result,
        existing_tool_result_ids,
    )?;

    if parts.is_empty() {
        if role == "user" {
            parts.push(json!({ "text": crate::proxy::mappers::common_utils::TRANSIT_DEFENSE_FALLBACK_TEXT }));
        } else {
            return Ok(json!(null)); // Indicate no content to add
        }
    }

    if role == "model" {
        reorder_gemini_parts(&mut parts);
    }

    Ok(json!({
        "role": role,
        "parts": parts
    }))
}

/// 构建 Contents (Messages)
pub(crate) fn build_google_contents(
    messages: &[Message],
    claude_req: &ClaudeRequest,
    tool_id_to_name: &mut HashMap<String, String>,
    tool_name_to_schema: &HashMap<String, Value>,
    is_thinking_enabled: bool,
    allow_dummy_thought: bool,
    mapped_model: &str,
    session_id: &str, // [NEW v3.3.17] Session ID for signature caching
    is_retry: bool,
    timing: &mut TransformTiming,
) -> Result<Value, String> {
    let mut contents = Vec::new();
    let mut last_thought_signature: Option<String> = None;
    let mut _accumulated_usage: Option<Value> = None;
    // Track pending tool_use IDs for recovery
    let mut pending_tool_use_ids: Vec<String> = Vec::new();

    // [NEW] 用于识别并过滤 Claude Code 重复回显的任务指令
    let mut last_user_task_text_normalized: Option<String> = None;
    let mut previous_was_tool_result = false;

    let _msg_count = messages.len();

    // [FIX #632] Pre-scan all messages to identify all tool_result IDs that ALREADY exist in the conversation.
    // This prevents Elastic-Recovery from injecting duplicate results if they are present later in the chain.
    let mut existing_tool_result_ids = std::collections::HashSet::new();
    for msg in messages {
        if let MessageContent::Array(blocks) = &msg.content {
            for block in blocks {
                if let ContentBlock::ToolResult { tool_use_id, .. } = block {
                    existing_tool_result_ids.insert(tool_use_id.clone());
                }
            }
        }
    }

    for (i, msg) in messages.iter().enumerate() {
        if msg.role == "assistant" {
            // CRITICAL: Reset last_thought_signature at the start of each assistant message
            // so signatures never bleed across turns!
            last_thought_signature = None;
        }

        let google_content = build_google_content(
            msg,
            claude_req,
            is_thinking_enabled,
            session_id,
            i,
            allow_dummy_thought,
            is_retry,
            tool_id_to_name,
            tool_name_to_schema,
            mapped_model,
            &mut last_thought_signature,
            &mut pending_tool_use_ids,
            &mut last_user_task_text_normalized,
            &mut previous_was_tool_result,
            &existing_tool_result_ids,
        )?;

        if !google_content.is_null() {
            contents.push(google_content);
        }
    }

    // [Removed] ensure_last_assistant_has_thinking
    // Corrupted signature issues proved we cannot fake thinking blocks.
    // Instead we rely on should_disable_thinking_due_to_history to prevent this state.

    // 思考回填：仅在开启思考且非 Gemini < 3 模型时恢复思维块与签名
    let should_finalize_thinking =
        is_thinking_enabled && !crate::proxy::model_specs::is_gemini_under_v3(mapped_model);

    let think_start = std::time::Instant::now();
    crate::proxy::pipeline::InboundThinkingPipeline::process_contents(
        &mut contents,
        crate::proxy::pipeline::ProxyProtocol::AnthropicClaude,
        mapped_model,
        should_finalize_thinking,
        Some(session_id),
        false,
    );
    timing.think_fill_micros = think_start.elapsed().as_micros() as u64;

    Ok(json!(contents))
}

/// Merge adjacent messages with the same role

/// Merge adjacent messages with the same role
pub(crate) fn merge_adjacent_roles(mut contents: Vec<Value>) -> Vec<Value> {
    if contents.is_empty() {
        return contents;
    }

    let mut merged = Vec::new();
    let mut current_msg = contents.remove(0);

    for msg in contents {
        let current_role = current_msg["role"].as_str().unwrap_or_default();
        let next_role = msg["role"].as_str().unwrap_or_default();

        if current_role == next_role {
            // Merge parts
            if let Some(current_parts) = current_msg.get_mut("parts").and_then(|p| p.as_array_mut())
            {
                if let Some(next_parts) = msg.get("parts").and_then(|p| p.as_array()) {
                    current_parts.extend(next_parts.clone());

                    // [FIX #709] Core Fix: After merging parts from adjacent messages,
                    // we must RE-SORT them to ensure any thinking blocks from the
                    // second message are moved to the very front of the combined array.
                    reorder_gemini_parts(current_parts);
                }
            }
        } else {
            merged.push(current_msg);
            current_msg = msg;
        }
    }
    merged.push(current_msg);
    merged
}

/// 构建 Tools

/// Fallback message when a tool result is empty (prevents model hallucination).
pub(crate) fn empty_tool_result_fallback(merged_content: &str, is_error: Option<bool>) -> String {
    if merged_content.trim().is_empty() {
        if is_error.unwrap_or(false) {
            "Tool execution failed with no output.".to_string()
        } else {
            "Command executed successfully.".to_string()
        }
    } else {
        merged_content.to_string()
    }
}

/// Inject synthetic tool results for pending tool uses missing from a User message.
pub(crate) fn inject_missing_tool_results(
    parts: &mut Vec<serde_json::Value>,
    is_assistant: bool,
    pending_tool_use_ids: &mut Vec<String>,
    current_turn_tool_result_ids: &std::collections::HashSet<String>,
    tool_id_to_name: &std::collections::HashMap<String, String>,
) {
    // If this is a User message, check if we need to inject missing tool results
    if !is_assistant && !pending_tool_use_ids.is_empty() {
        let missing_ids: Vec<_> = pending_tool_use_ids
            .iter()
            .filter(|id| !current_turn_tool_result_ids.contains(*id))
            .cloned()
            .collect();

        if !missing_ids.is_empty() {
            tracing::warn!("[Elastic-Recovery] Injecting {} missing tool results into User message (IDs: {:?})", missing_ids.len(), missing_ids);
            for id in missing_ids.iter().rev() {
                // Insert in reverse order to maintain order at index 0? No, just insert at 0.
                let name = tool_id_to_name.get(id).cloned().unwrap_or(id.clone());
                let synthetic_part = json!({
                    "functionResponse": {
                        "name": name,
                        "response": {
                            "result": "Tool execution interrupted. No result provided."
                        },
                        "id": id
                    }
                });
                // Prepend to ensure they are present before any text
                parts.insert(0, synthetic_part);
            }
        }
        // All pending IDs are now handled (either present or injected)
        pending_tool_use_ids.clear();
    }
}
