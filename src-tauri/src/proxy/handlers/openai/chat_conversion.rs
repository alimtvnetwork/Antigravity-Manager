// Chat <-> Responses conversion helpers and Codex ledger helpers.
use serde_json::{json, Value};

use crate::proxy::mappers::openai::{OpenAIContent, OpenAIResponse};

fn openai_content_text(content: &OpenAIContent) -> String {
    match content {
        OpenAIContent::String(text) => text.clone(),
        OpenAIContent::Array(parts) => parts
            .iter()
            .filter_map(|part| match part {
                OpenAIContentBlock::Text { text } => Some(text.as_str()),
                _ => None,
            })
            .collect(),
    }
}

fn responses_usage_value(chat_response: &OpenAIResponse) -> Value {
    chat_response
        .usage
        .as_ref()
        .map(|usage| usage.to_responses_usage_value())
        .unwrap_or_else(|| {
            json!({
                "input_tokens": 0,
                "input_tokens_details": { "cached_tokens": 0 },
                "output_tokens": 0,
                "output_tokens_details": { "reasoning_tokens": 0 },
                "total_tokens": 0
            })
        })
}
pub(crate) fn convert_chat_response_to_responses(chat_response: &OpenAIResponse) -> Value {
    let mut output = Vec::new();

    for choice in &chat_response.choices {
        if let Some(reasoning) = choice
            .message
            .reasoning_content
            .as_deref()
            .filter(|reasoning| !reasoning.trim().is_empty())
        {
            output.push(json!({
                "id": format!("rs_{}", uuid::Uuid::new_v4().simple()),
                "type": "reasoning",
                "status": "completed",
                "summary": [{ "type": "summary_text", "text": reasoning }]
            }));
        }

        let text = choice
            .message
            .content
            .as_ref()
            .map(openai_content_text)
            .unwrap_or_default();
        let refusal = choice
            .message
            .refusal
            .as_deref()
            .filter(|refusal| !refusal.is_empty());
        if !text.is_empty() || refusal.is_some() {
            let mut content = Vec::new();
            if !text.is_empty() {
                content.push(json!({
                    "type": "output_text",
                    "text": text,
                    "annotations": []
                }));
            }
            if let Some(refusal) = refusal {
                content.push(json!({ "type": "refusal", "refusal": refusal }));
            }
            output.push(json!({
                "id": format!("msg_{}", uuid::Uuid::new_v4().simple()),
                "type": "message",
                "role": "assistant",
                "status": "completed",
                "content": content
            }));
        }

        for tool_call in choice.message.tool_calls.iter().flatten() {
            let Some(function) = tool_call.function.as_ref() else {
                tracing::warn!(
                    tool_call_id = %tool_call.id,
                    "[Responses Compat] Skipping tool call without function payload"
                );
                continue;
            };
            let call_id = tool_call.call_id.as_deref().unwrap_or(&tool_call.id);
            output.push(json!({
                "id": format!("fc_{}", uuid::Uuid::new_v4().simple()),
                "type": "function_call",
                "status": "completed",
                "call_id": call_id,
                "name": function.name,
                "arguments": function.arguments
            }));
        }
    }

    json!({
        "id": format!("resp_{}", uuid::Uuid::new_v4().simple()),
        "object": "response",
        "type": "response",
        "created_at": chrono::Utc::now().timestamp(),
        "status": "completed",
        "error": null,
        "output": output,
        "model": chat_response.model,
        "usage": responses_usage_value(chat_response)
    })
}

/// 仅识别客户端本地私有展示项（如本地渲染的思考块），绝不误杀伴随工具调用的真实过程进度说明。
/// 只有明确以 `msg_thought_` 为 ID 前缀或历史遗留以 `**Thinking**` 开头的消息才被视为 transcript-only。
/// 真实的 `phase="commentary"` 消息包含正文说明，必须作为上下文历史保留以保证模型少样本进度输出范式。
pub(crate) fn is_codex_transcript_only_assistant_message(item: &Value, text: &str) -> bool {
    if responses_input_item_type(item) != "message"
        || item.get("role").and_then(Value::as_str) != Some("assistant")
    {
        return false;
    }

    item.get("id")
        .and_then(Value::as_str)
        .is_some_and(|id| id.starts_with(CODEX_VISIBLE_THOUGHT_MESSAGE_PREFIX))
        || text.trim_start().starts_with("**Thinking**")
}

pub(crate) fn codex_ledger_from_body(
    body: &Value,
) -> (
    Option<crate::proxy::mappers::openai::interaction_ledger::InteractionLedger>,
    VecDeque<String>,
) {
    let ledger = crate::proxy::mappers::openai::interaction_ledger::build_codex_interaction_ledger(
        body.get("input"),
        body.get("instructions").and_then(|v| v.as_str()),
        body.get("session_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        body.get("previous_response_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
    );

    let mut markers = VecDeque::new();
    if let Some(ledger) = &ledger {
        for step in ledger.turns.iter().flat_map(|turn| turn.steps.iter()) {
            if step.raw_item.get("type").and_then(|v| v.as_str()) != Some("instructions") {
                markers.push_back(
                    crate::proxy::mappers::openai::interaction_ledger::step_marker(step),
                );
            }
        }
    }

    (ledger, markers)
}

pub(crate) fn responses_routing_session_id(
    explicit_session_id: Option<&str>,
    previous_response_id: Option<&str>,
    stored_routing_session_id: Option<&str>,
    response_id: &str,
) -> String {
    explicit_session_id
        .filter(|id| !id.is_empty())
        .or(stored_routing_session_id.filter(|id| !id.is_empty()))
        .or(previous_response_id.filter(|id| !id.is_empty()))
        .unwrap_or(response_id)
        .to_string()
}

fn strip_codex_step_markers(content: &str) -> String {
    if !content.contains("[codex-turn:") {
        return content.to_string();
    }
    let had_trailing_newline = content.ends_with('\n');
    let mut cleaned = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("[codex-turn:")
            && trimmed.contains(" step:")
            && trimmed.contains(" type:")
            && trimmed.ends_with(']')
        {
            continue;
        }
        cleaned.push(line);
    }
    let mut res = cleaned.join("\n");
    if had_trailing_newline && !res.ends_with('\n') {
        res.push('\n');
    }
    res
}

pub(crate) fn prefix_with_step_marker(_marker: Option<String>, content: String) -> String {
    // Step markers are useful in debug ledgers, but must not be visible to
    // Gemini. If they enter the prompt, the model learns to emit them as text.
    strip_codex_step_markers(&content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
use crate::proxy::handlers::openai::responses_media::responses_input_item_type;

    fn responses_routing_identity_follows_the_response_chain() {
        let first = responses_routing_session_id(None, None, None, "resp-root-a");
        let second = responses_routing_session_id(None, None, None, "resp-root-b");
        assert_eq!(first, "resp-root-a");
        assert_eq!(second, "resp-root-b");
        assert_ne!(first, second);

        let continued =
            responses_routing_session_id(None, Some("resp-parent"), Some(&first), "resp-child");
        let branch =
            responses_routing_session_id(None, Some("resp-parent"), Some(&first), "resp-branch");
        assert_eq!(continued, first);
        assert_eq!(branch, first);
        assert_eq!(
            responses_routing_session_id(
                Some("client-session"),
                Some("resp-parent"),
                Some(&first),
                "resp-child"
            ),
            "client-session"
        );
        assert_eq!(
            responses_routing_session_id(None, Some("resp-missing"), None, "resp-child"),
            "resp-missing"
        );
    }

    fn responses_compat_emits_standard_text_reasoning_and_function_calls() {
        let chat_response = serde_json::from_value(json!({
            "id": "chatcmpl_test",
            "object": "chat.completion",
            "created": 1,
            "model": "gemini-3.6-flash-high",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Visible answer",
                    "reasoning_content": "Internal analysis",
                    "tool_calls": [{
                        "id": "call_reply",
                        "type": "function",
                        "function": {
                            "name": "reply",
                            "arguments": "{\"msg_id\":\"1\"}"
                        }
                    }]
                },
                "finish_reason": "tool_calls"
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 5,
                "total_tokens": 15
            }
        }))
        .expect("valid OpenAI response fixture");

        let response = convert_chat_response_to_responses(&chat_response);
        assert_eq!(response["object"], "response");
        assert_eq!(response["output"][0]["type"], "reasoning");
        assert_eq!(
            response["output"][0]["summary"][0]["text"],
            "Internal analysis"
        );
        assert_eq!(response["output"][1]["type"], "message");
        assert_eq!(response["output"][1]["content"][0]["type"], "output_text");
        assert_eq!(
            response["output"][1]["content"][0]["text"],
            "Visible answer"
        );
        assert_eq!(response["output"][2]["type"], "function_call");
        assert_eq!(response["output"][2]["call_id"], "call_reply");
        assert_eq!(response["output"][2]["name"], "reply");
        assert!(response["output"][1]["content"][0]["text"]
            .as_str()
            .is_some_and(|text| !text.contains("Internal analysis")));
    }

    fn identifies_codex_transcript_only_assistant_messages() {
        let thought = json!({
            "type": "message",
            "id": "msg_thought_abc_0",
            "role": "assistant",
            "phase": "commentary",
            "content": [{"type": "output_text", "text": "thinking"}],
        });
        let normal_commentary = json!({
            "type": "message",
            "role": "assistant",
            "phase": "commentary",
            "content": [{"type": "output_text", "text": "progress"}],
        });
        let contaminated_final = json!({
            "type": "message",
            "role": "assistant",
            "phase": "final_answer",
            "content": [{"type": "output_text", "text": "**Thinking**\n\nlegacy thought"}],
        });
        let clean_final = json!({
            "type": "message",
            "role": "assistant",
            "phase": "final_answer",
            "content": [{"type": "output_text", "text": "done"}],
        });

        assert!(is_codex_transcript_only_assistant_message(
            &thought, "thinking"
        ));
        assert!(!is_codex_transcript_only_assistant_message(
            &normal_commentary,
            "progress"
        ));
        assert!(is_codex_transcript_only_assistant_message(
            &contaminated_final,
            "**Thinking**\n\nlegacy thought"
        ));
        assert!(!is_codex_transcript_only_assistant_message(
            &clean_final,
            "done"
        ));
    }
}
