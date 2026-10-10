// `handle_completions` prelude phase 3: Codex normalization + request parse.
use axum::http::StatusCode;
use axum::response::Response;
use serde_json::{json, Value};

use crate::proxy::mappers::openai::OpenAIRequest;

use super::completions_prep_session::CompletionsSessionPrep;
use super::responses_history::{
    drop_leading_orphan_tool_history, rewrite_terminal_assistant_prefill,
};
use crate::proxy::handlers::openai::responses_media::responses_input_item_type;

pub(crate) struct CompletionsCodexPrep {
    pub previous_response_id: Option<String>,
    pub explicit_session_id: Option<String>,
    pub response_id_for_save: String,
    pub session_parent: Option<crate::proxy::http_session_store::SessionParent>,
    pub routing_session_id: String,
    pub signature_read_key: Option<String>,
    pub normalized_interaction_ledger:
        Option<crate::proxy::mappers::openai::interaction_ledger::InteractionLedger>,
    pub session_save_input: Vec<Value>,
    pub session_save_instructions: String,
    pub openai_req: OpenAIRequest,
}

pub(crate) fn completions_prep_codex(
    prep: CompletionsSessionPrep,
    is_codex_style: bool,
    store_response: bool,
) -> Result<CompletionsCodexPrep, Response> {
    let CompletionsSessionPrep {
        body,
        previous_response_id,
        explicit_session_id,
        response_id_for_save,
        session_parent,
        stored_routing_session_id: _,
        session_delta_input: _,
        routing_session_id,
        signature_read_key,
        bounded_session_input,
    } = prep;
    let mut body = body;
    let mut bounded_session_input = bounded_session_input;
    // 2. Reuse handle_chat_completions logic (wrapping with custom handler or direct call)
    // Actually, due to SSE handling differences (Codex uses different event format), we replicate the loop here or abstract it.
    // For now, let's replicate the core loop but with Codex specific SSE mapping.

    // [Fix Phase 2] Backport normalization logic from handle_chat_completions
    // Handle "instructions" + "input" (Codex style) -> system + user messages
    // This is critical because `transform_openai_request` expects `messages` to be populated.

    // [FIX] 检查是否已经有 messages (被第一次标准化处理过)
    let has_codex_fields = body.get("instructions").is_some() || body.get("input").is_some();
    let already_normalized = body
        .get("messages")
        .and_then(|m| m.as_array())
        .map(|arr| !arr.is_empty())
        .unwrap_or(false);

    // 只有在未标准化时才进行简单转换
    if has_codex_fields && !already_normalized {
        tracing::debug!("[Codex] Performing simple normalization (messages not yet populated)");

        let mut messages = Vec::new();

        // instructions -> system message
        if let Some(inst) = body.get("instructions").and_then(|v| v.as_str()) {
            if !inst.is_empty() {
                messages.push(json!({
                    "role": "system",
                    "content": inst
                }));
            }
        }

        // input -> user message (支持对象数组形式的对话历史)
        if let Some(input) = body.get("input") {
            if let Some(s) = input.as_str() {
                messages.push(json!({
                    "role": "user",
                    "content": s
                }));
            } else if let Some(arr) = input.as_array() {
                // 判断是消息对象数组还是简单的内容块/字符串数组
                let is_message_array = arr
                    .first()
                    .and_then(|v| v.as_object())
                    .map(|obj| obj.contains_key("role") || obj.contains_key("type"))
                    .unwrap_or(false);

                if is_message_array {
                    // 深度识别：像处理 messages 一样处理 input 数组，并自动映射 Responses API 的工具流
                    for item in arr {
                        if let Some(obj) = item.as_object() {
                            let item_type = responses_input_item_type(item);
                            if !item_type.is_empty() {
                                match item_type {
                                    "message" => {
                                        let role = obj
                                            .get("role")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("user");
                                        let content =
                                            obj.get("content").cloned().unwrap_or(json!(""));
                                        messages.push(json!({ "role": role, "content": content }));
                                    }
                                    "function_call" | "custom_tool_call" => {
                                        let call_id = obj
                                            .get("call_id")
                                            .or_else(|| obj.get("id"))
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("");
                                        let name =
                                            obj.get("name").and_then(|v| v.as_str()).unwrap_or("");
                                        let mut arguments = obj
                                            .get("arguments")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("")
                                            .to_string();
                                        if item_type == "custom_tool_call" {
                                            if let Some(input) =
                                                obj.get("input").and_then(|v| v.as_str())
                                            {
                                                arguments = serde_json::to_string(
                                                    &json!({ "input": input }),
                                                )
                                                .unwrap_or_else(|_| "{}".to_string());
                                            }
                                        }
                                        messages.push(json!({
                                            "role": "assistant",
                                            "content": "",
                                            "tool_calls": [{
                                                "id": if call_id.is_empty() { "call_unknown" } else { call_id },
                                                "type": "function",
                                                "function": { "name": name, "arguments": arguments },
                                            }],
                                        }));
                                    }
                                    "function_call_output" | "custom_tool_call_output" => {
                                        let call_id = obj
                                            .get("call_id")
                                            .or_else(|| obj.get("tool_call_id"))
                                            .or_else(|| obj.get("id"))
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("");
                                        let output_value =
                                            obj.get("output").cloned().unwrap_or(json!(""));
                                        let output_str = if let Some(s) = output_value.as_str() {
                                            s.to_string()
                                        } else {
                                            output_value.to_string()
                                        };
                                        messages.push(json!({
                                            "role": "tool",
                                            "tool_call_id": call_id,
                                            "content": output_str,
                                        }));
                                    }
                                    _ => {
                                        messages.push(item.clone());
                                    }
                                }
                                continue;
                            }
                        }
                        messages.push(item.clone());
                    }
                } else {
                    // 降级处理：传统的字符串或混合内容拼接
                    let content = arr
                        .iter()
                        .map(|v| {
                            if let Some(s) = v.as_str() {
                                s.to_string()
                            } else if v.is_object() {
                                v.to_string()
                            } else {
                                "".to_string()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n");

                    if !content.is_empty() {
                        messages.push(json!({
                            "role": "user",
                            "content": content
                        }));
                    }
                }
            } else {
                let content = input.to_string();
                if !content.is_empty() {
                    messages.push(json!({
                        "role": "user",
                        "content": content
                    }));
                }
            };
        }

        if let Some(obj) = body.as_object_mut() {
            tracing::debug!(
                "[Codex] Injecting normalized messages: {} messages",
                messages.len()
            );
            obj.insert("messages".to_string(), Value::Array(messages));
        }
    } else if already_normalized {
        tracing::debug!(
            "[Codex] Skipping normalization (messages already populated by first pass)"
        );
    }

    if is_codex_style {
        if let Some(messages) = body.get_mut("messages").and_then(Value::as_array_mut) {
            let dropped = drop_leading_orphan_tool_history(messages);
            if dropped > 0 {
                tracing::warn!(
                    dropped_messages = dropped,
                    "[Responses Compat] Dropped leading orphan tool history"
                );
            }
            if rewrite_terminal_assistant_prefill(messages) {
                tracing::debug!(
                    "[Responses Compat] Rewrote terminal assistant text prefill as user input"
                );
            }
        }
    }
    // [FIX] 在 openai_req 反序列化之前，从 body 中捕获原始 input 和 instructions
    // 用于后续 session 保存时，保留完整的工具调用历史（而非从 openai_req.messages 重建丢失信息）
    let normalized_interaction_ledger = body.get("_interaction_ledger").cloned();
    let (session_save_input, session_save_instructions) = if let Some(obj) = body.as_object_mut() {
        let input = bounded_session_input.take().unwrap_or_default();
        obj.remove("input");
        let instructions = obj
            .remove("instructions")
            .and_then(|value| match value {
                Value::String(text) => Some(text),
                _ => None,
            })
            .unwrap_or_default();
        (
            input,
            if store_response {
                instructions
            } else {
                String::new()
            },
        )
    } else {
        (Vec::new(), String::new())
    };

    let mut openai_req: OpenAIRequest = match serde_json::from_value(body) {
        Ok(req) => req,
        Err(e) => {
            return Err(
                (StatusCode::BAD_REQUEST, format!("Invalid request: {}", e)).into_response()
            );
        }
    };

    // Safety: Inject empty message if needed
    if openai_req.messages.is_empty() {
        openai_req
            .messages
            .push(crate::proxy::mappers::openai::OpenAIMessage {
                role: "user".to_string(),
                content: Some(crate::proxy::mappers::openai::OpenAIContent::String(
                    " ".to_string(),
                )),
                reasoning_content: None,
                signature: None,
                tool_calls: None,
                tool_call_id: None,
                name: None,
                refusal: None,
            });
    }

    Ok(CompletionsCodexPrep {
        previous_response_id,
        explicit_session_id,
        response_id_for_save,
        session_parent,
        routing_session_id,
        signature_read_key,
        normalized_interaction_ledger,
        session_save_input,
        session_save_instructions,
        openai_req,
    })
}
