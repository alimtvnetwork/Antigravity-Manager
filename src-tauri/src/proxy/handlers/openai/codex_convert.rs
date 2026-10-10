// Codex (Responses-style) request -> OpenAI chat request conversion.
use serde_json::{json, Value};

use super::chat_conversion::{
    codex_ledger_from_body, is_codex_transcript_only_assistant_message, prefix_with_step_marker,
};
use super::responses_history::{
    build_responses_tool_output_content, drop_leading_orphan_tool_history,
    history_without_inline_media, rewrite_terminal_assistant_prefill,
};
use super::responses_media::{
    responses_input_item_type, responses_message_parts, responses_tool_output_parts,
};
use super::tool_cache::get_cached_tool_call;

pub(crate) fn convert_codex_to_openai_request(mut body: Value) -> Value {
    let instructions = body
        .get("instructions")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let (interaction_ledger, mut step_markers) = codex_ledger_from_body(&body);
    let input_items = body
        .as_object_mut()
        .and_then(|obj| obj.remove("input"))
        .and_then(|value| match value {
            Value::Array(items) => Some(items),
            _ => None,
        })
        .unwrap_or_default();

    let mut messages = Vec::new();
    if !instructions.is_empty() {
        messages.push(json!({ "role": "system", "content": instructions }));
    }

    let mut call_id_to_name = std::collections::HashMap::new();
    let mut skipped_incomplete_custom_call_ids = std::collections::HashSet::new();

    {
        for item in &input_items {
            let item_type = responses_input_item_type(item);
            if item_type == "custom_tool_call"
                && item.get("status").and_then(|v| v.as_str()) == Some("incomplete")
            {
                if let Some(call_id) = item
                    .get("call_id")
                    .and_then(|v| v.as_str())
                    .or_else(|| item.get("id").and_then(|v| v.as_str()))
                {
                    skipped_incomplete_custom_call_ids.insert(call_id.to_string());
                }
                continue;
            }
            match item_type {
                "function_call" | "custom_tool_call" | "local_shell_call" | "web_search_call" => {
                    let call_id = item
                        .get("call_id")
                        .and_then(|v| v.as_str())
                        .or_else(|| item.get("id").and_then(|v| v.as_str()))
                        .unwrap_or("unknown");
                    let mut name = item
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();
                    if item_type == "local_shell_call" && name == "unknown" {
                        name = "local_shell_call".to_string();
                    } else if item_type == "web_search_call" && name == "unknown" {
                        name = "web_search_call".to_string();
                    }
                    call_id_to_name.insert(call_id.to_string(), name);
                }
                _ => {}
            }
        }
    }

    {
        for mut item in input_items {
            let item_type = responses_input_item_type(&item).to_string();
            let step_marker = step_markers.pop_front();
            if item_type == "custom_tool_call"
                && item.get("status").and_then(|v| v.as_str()) == Some("incomplete")
            {
                continue;
            }
            match item_type.as_str() {
                "message" => {
                    let role = item
                        .get("role")
                        .and_then(Value::as_str)
                        .unwrap_or("user")
                        .to_string();
                    let (text_parts, image_parts) = responses_message_parts(&mut item);
                    let joined_text = text_parts.join("\n");
                    if is_codex_transcript_only_assistant_message(&item, &joined_text) {
                        continue;
                    }

                    if role == "assistant"
                        && joined_text.trim().is_empty()
                        && image_parts.is_empty()
                    {
                        continue;
                    }

                    if image_parts.is_empty() {
                        let content = prefix_with_step_marker(step_marker, joined_text);
                        messages.push(json!({ "role": role, "content": content }));
                    } else {
                        let mut content_blocks = Vec::new();
                        let marker_text = prefix_with_step_marker(step_marker, joined_text);
                        if !marker_text.is_empty() {
                            content_blocks.push(json!({ "type": "text", "text": marker_text }));
                        }
                        content_blocks.extend(image_parts);
                        messages.push(json!({ "role": role, "content": content_blocks }));
                    }
                }
                "function_call" | "custom_tool_call" | "local_shell_call" | "web_search_call" => {
                    let mut name = item
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    let mut args_str = item
                        .get("arguments")
                        .and_then(|v| v.as_str())
                        .unwrap_or("{}")
                        .to_string();
                    let call_id = item
                        .get("call_id")
                        .and_then(|v| v.as_str())
                        .or_else(|| item.get("id").and_then(|v| v.as_str()))
                        .unwrap_or("unknown");

                    if item_type == "custom_tool_call" {
                        if let Some(input) = item.get("input").and_then(|v| v.as_str()) {
                            args_str = serde_json::to_string(&json!({ "input": input }))
                                .unwrap_or_else(|_| "{}".to_string());
                        }
                    } else if item_type == "local_shell_call" || name == "local_shell_call" {
                        name = "shell";
                        if let Some(action) = item.get("action") {
                            if let Some(exec) = action.get("exec") {
                                let mut args_obj = serde_json::Map::new();
                                if let Some(cmd) = exec.get("command") {
                                    let cmd_val = if cmd.is_string() {
                                        json!([cmd])
                                    } else {
                                        cmd.clone()
                                    };
                                    args_obj.insert("command".to_string(), cmd_val);
                                }
                                if let Some(wd) =
                                    exec.get("working_directory").or(exec.get("workdir"))
                                {
                                    args_obj.insert("workdir".to_string(), wd.clone());
                                }
                                args_str = serde_json::to_string(&args_obj)
                                    .unwrap_or_else(|_| "{}".to_string());
                            }
                        }
                    } else if item_type == "web_search_call" || name == "web_search_call" {
                        name = "google_search";
                        if let Some(action) = item.get("action") {
                            let mut args_obj = serde_json::Map::new();
                            if let Some(q) = action.get("query") {
                                args_obj.insert("query".to_string(), q.clone());
                            }
                            args_str = serde_json::to_string(&args_obj)
                                .unwrap_or_else(|_| "{}".to_string());
                        }
                    }

                    messages.push(json!({
                        "role": "assistant",
                        "content": "",
                        "tool_calls": [{
                            "id": call_id,
                            "type": "function",
                            "function": { "name": name, "arguments": args_str }
                        }]
                    }));
                }
                "function_call_output" | "custom_tool_call_output" => {
                    let call_id = item
                        .get("call_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();
                    if item_type == "custom_tool_call_output"
                        && skipped_incomplete_custom_call_ids.contains(&call_id)
                    {
                        tracing::warn!(
                            "Skipping output for incomplete custom tool call {}",
                            call_id
                        );
                        continue;
                    }
                    let (mut output_str, output_media) = responses_tool_output_parts(&mut item);

                    let name = match call_id_to_name.get(&call_id).cloned().or_else(|| {
                        get_cached_tool_call(&call_id).and_then(|v| {
                            v.get("name")
                                .and_then(|n| n.as_str())
                                .map(|s| s.to_string())
                        })
                    }) {
                        Some(name) => name,
                        None if item_type == "custom_tool_call_output" => {
                            tracing::warn!(
                                "Skipping orphan custom_tool_call_output for unknown call_id {}",
                                call_id
                            );
                            continue;
                        }
                        None => "shell".to_string(),
                    };

                    output_str = prefix_with_step_marker(step_marker, output_str);
                    let output_content =
                        build_responses_tool_output_content(output_str, output_media);

                    messages.push(json!({
                        "role": "tool",
                        "tool_call_id": call_id,
                        "name": name,
                        "content": output_content
                    }));
                }
                _ => {}
            }
        }
    }

    let dropped = drop_leading_orphan_tool_history(&mut messages);
    if dropped > 0 {
        tracing::warn!(
            dropped_messages = dropped,
            "[Responses Compat] Dropped leading orphan websocket tool history"
        );
    }
    if rewrite_terminal_assistant_prefill(&mut messages) {
        tracing::debug!(
            "[Responses Compat] Rewrote websocket terminal assistant text prefill as user input"
        );
    }

    if let Some(obj) = body.as_object_mut() {
        obj.insert("messages".to_string(), Value::Array(messages));
        if let Some(ledger) = interaction_ledger {
            obj.insert("_interaction_ledger".to_string(), json!(ledger));
        }
        obj.remove("input");
        obj.remove("instructions");
    }
    body
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proxy::mappers::openai::{transform_openai_request, OpenAIRequest};
    use serde_json::{json, Value};

    fn responses_tool_output_image_is_sent_as_inline_data() {
        let converted = convert_codex_to_openai_request(json!({
            "model": "gemini-3.7-flash-high",
            "input": [
                {
                    "type": "message",
                    "role": "user",
                    "content": [{"type": "input_text", "text": "Generate an image."}]
                },
                {
                    "type": "function_call",
                    "call_id": "call_image",
                    "name": "view_image",
                    "arguments": "{}"
                },
                {
                    "type": "function_call_output",
                    "call_id": "call_image",
                    "output": [
                        {"type": "input_text", "text": "image generated"},
                        {"type": "input_image", "image_url": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=="}
                    ]
                }
            ]
        }));
        assert!(converted.get("input").is_none());
        let request: OpenAIRequest =
            serde_json::from_value(converted).expect("valid OpenAI request");

        let (upstream, _, _, _) =
            transform_openai_request(&request, "project", "gemini-3.7-flash-high", None);
        let parts = upstream["request"]["contents"]
            .as_array()
            .expect("contents")
            .iter()
            .flat_map(|content| content["parts"].as_array().expect("content parts").iter())
            .collect::<Vec<_>>();
        let function_response = parts
            .iter()
            .find(|part| part.get("functionResponse").is_some())
            .expect("function response");
        let inline_data = parts
            .iter()
            .find(|part| part.get("inlineData").is_some())
            .expect("inline image data");

        assert_eq!(
            function_response["functionResponse"]["response"]["result"],
            "image generated"
        );
        assert!(!function_response.to_string().contains("data:image/"));
        assert_eq!(inline_data["inlineData"]["mimeType"], "image/png");
        assert_eq!(inline_data["inlineData"]["data"], "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==");
    }

    fn test_codex_process_commentary_is_retained_in_request_messages() {
        let input = json!({
            "model": "gemini-2.5-pro",
            "input": [
                {
                    "type": "message",
                    "role": "user",
                    "content": [{"type": "input_text", "text": "检查系统故障"}]
                },
                {
                    "type": "message",
                    "id": "msg_thought_123",
                    "role": "assistant",
                    "phase": "commentary",
                    "content": [{"type": "output_text", "text": "private thought"}]
                },
                {
                    "type": "message",
                    "id": "msg_comm_123",
                    "role": "assistant",
                    "phase": "commentary",
                    "content": [{"type": "output_text", "text": "概览显示 HTTP 502，接下来检查网关连接。"}]
                },
                {
                    "type": "function_call",
                    "id": "call_inspect",
                    "name": "inspect_case",
                    "arguments": "{\"case\":\"case_1\"}"
                }
            ]
        });

        let converted = convert_codex_to_openai_request(input);
        let messages = converted["messages"].as_array().expect("messages array");

        // 验证：user 消息存在
        assert_eq!(messages[0]["role"], "user");
        assert_eq!(messages[0]["content"], "检查系统故障");

        // 验证：私有思考 msg_thought_123 被成功过滤，未进入 messages
        assert!(messages
            .iter()
            .all(|m| m.get("content").and_then(Value::as_str) != Some("private thought")));

        // 验证：普通进度 commentary 消息被成功保留
        assert_eq!(messages[1]["role"], "assistant");
        assert_eq!(
            messages[1]["content"],
            "概览显示 HTTP 502，接下来检查网关连接。"
        );

        // 验证：工具调用正常跟随
        assert_eq!(messages[2]["role"], "assistant");
        assert_eq!(
            messages[2]["tool_calls"][0]["function"]["name"],
            "inspect_case"
        );
    }
}
