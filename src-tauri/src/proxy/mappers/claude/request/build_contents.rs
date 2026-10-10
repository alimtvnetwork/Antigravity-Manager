// Content building (split from request.rs).
// Claude 请求转换 (Claude → Gemini v1internal)
// 对应 transformClaudeRequestIn

use super::super::models::*;
use crate::proxy::mappers::signature_store::get_thought_signature; // Deprecated, kept for fallback
use crate::proxy::session_manager::SessionManager;
use serde_json::{json, Value};
use std::collections::HashMap;

use super::build_parts::{
    build_google_content, empty_tool_result_fallback, inject_missing_tool_results,
};

/// 构建 Contents (Messages)
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
                            // [NEW] 任务去重逻辑: 如果当前是 User 消息，且紧跟在 ToolResult 之后，
                            // 检查该文本是否与上一轮任务描述完全一致。
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

                            // 记录最近一次 User 任务文本用于后续比对
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

                        // [2026-09-27] 占位/空思考块直接丢弃（不写思考块、不降级为文本）。
                        // 官方样本（baogao.txt）9/24 轮是「无思考块 + 锚点带签名」，
                        // 空 / "." / "..." / 空格 / "·" 等占位思考无信息量；签名归位
                        // 由流水线终审 place_turn_signature 按锚点规则处理（ inbound 已
                        // 把占位块的签名转移给首个非思考 part）。
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

                        // 3. 无合法签名时**绝不发明哨兵**。
                        //    依据（3 份官方报文 / 23 处签名）：哨兵在官方流量里出现 0/23 次，
                        //    它不属于 Antigravity 协议；且「跳过校验」不等于恢复推理连续性。
                        //    签名缺失是被上游容忍的（官方"在飞轮"即缺席），
                        //    归位统一交给流水线终审 `place_turn_signature`。
                        if let Some(sig) = effective_sig {
                            *last_thought_signature = Some(sig.clone());
                            let part = json!({
                                "text": final_thought_text,
                                "thought": true,
                                "thoughtSignature": sig
                            });
                            parts.push(part);
                        } else {
                            // 无签名：保留思考块结构（`thought: true`），但**不携带签名字段**。
                            // Gemini 目标的思考块本就不应带签名（铁律 I4：
                            // 官方报文里 `thought:true` 的 part 没有 thoughtSignature）。
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
                        // [FIX] 将 RedactedThinking 作为普通文本处理，保留上下文
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

                        // [New] 利用通用引擎修正参数类型 (替代以前硬编码的 shell 工具修复逻辑)
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

                        // 记录 id -> name 映射与签名上下文
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
                            // Claude 模型：Anthropic 官方验签引擎要求签名必须在思考块上，工具调用绝不携带签名，更不塞假哨兵
                            if let Some(obj) = part.as_object_mut() {
                                obj.remove("thoughtSignature");
                                obj.remove("thought_signature");
                            }
                        } else {
                            // 纯净线缆透传：客户端若自带签名则保持，未带则留空，全权委托进站流水线统一对齐与回填
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
                        // 优先使用之前记录的 name，否则用 tool_use_id
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

                        // [优化] 如果结果为空，注入显式确认信号，防止模型幻觉
                        merged_content = empty_tool_result_fallback(&merged_content, is_error);

                        let part = json!({
                            "functionResponse": {
                                "name": func_name,
                                "response": {"result": merged_content},
                                "id": tool_use_id
                            }
                        });

                        // 危险测试分支法则：ToolResult (functionResponse) 绝不携带签名
                        parts.push(part);

                        // 追加图片 parts
                        for extra in extra_parts {
                            parts.push(extra);
                        }

                        // 标记状态，用于下一条 User 消息的去重判断
                        *previous_was_tool_result = true;
                    }
                    // ContentBlock::RedactedThinking handled above at line 583
                    ContentBlock::ServerToolUse { .. }
                    | ContentBlock::WebSearchToolResult { .. } => {
                        // 搜索结果 block 不应由客户端发回给上游 (已由 tool_result 替代)
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
                // 纯净线缆原则：客户端若原本无思考块，适配器严禁凭空伪造 "..." 占位块！
                // 缺失思考块的判定与状态机复活统一委托给进站流水线（InboundThinkingPipeline）
            }
        }
    }

    Ok(parts)
}
