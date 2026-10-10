// `handle_completions` prelude phase 2: convert payload to messages.
use serde_json::{json, Value};

use super::chat_conversion::{
    codex_ledger_from_body, is_codex_transcript_only_assistant_message, prefix_with_step_marker,
};
use super::completions_prep_session::CompletionsSessionPrep;
use super::responses_history::{
    drop_leading_orphan_tool_history, into_history_without_inline_media,
    rewrite_terminal_assistant_prefill,
};
use super::responses_media::{
    build_responses_tool_output_content, responses_input_item_type, responses_message_parts,
    responses_tool_output_parts,
};

pub(crate) fn completions_convert_payload(
    mut prep: CompletionsSessionPrep,
    is_codex_style: bool,
    store_response: bool,
) -> CompletionsSessionPrep {
    let mut body = std::mem::replace(&mut prep.body, Value::Null);
    let mut session_delta_input = std::mem::take(&mut prep.session_delta_input);
    let mut bounded_session_input = prep.bounded_session_input.take();
    // 1. Convert Payload to Messages (Shared Chat Format)
    if is_codex_style {
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
        bounded_session_input = store_response.then(|| {
            session_delta_input
                .drain(..)
                .filter_map(into_history_without_inline_media)
                .filter(|item| !item.is_null())
                .collect()
        });

        let mut messages = Vec::new();

        // System Instructions
        if !instructions.is_empty() {
            messages.push(json!({ "role": "system", "content": instructions }));
        }

        let mut call_id_to_name = std::collections::HashMap::new();
        let mut skipped_incomplete_custom_call_ids = std::collections::HashSet::new();

        // Pass 1: Build Call ID to Name Map
        {
            for item in &input_items {
                let item_type = responses_input_item_type(&item).to_string();
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
                match item_type.as_str() {
                    "function_call" | "custom_tool_call" | "local_shell_call"
                    | "web_search_call" => {
                        let call_id = item
                            .get("call_id")
                            .and_then(|v| v.as_str())
                            .or_else(|| item.get("id").and_then(|v| v.as_str()))
                            .unwrap_or("unknown");

                        let name = if item_type == "local_shell_call" {
                            "shell"
                        } else if item_type == "web_search_call" {
                            "google_search"
                        } else {
                            item.get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown")
                        };

                        call_id_to_name.insert(call_id.to_string(), name.to_string());
                        tracing::debug!("Mapped call_id {} to name {}", call_id, name);
                    }
                    _ => {}
                }
            }
        }

        // Pass 2: Map durable conversation items to Gemini messages. Visible
        // assistant commentary stays in Codex's local transcript and must not
        // be replayed as model history.
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

                        let reasoning_content = item
                            .get("reasoning_content")
                            .or_else(|| item.get("thought"))
                            .and_then(Value::as_str)
                            .map(str::to_string);
                        let signature = item
                            .get("thoughtSignature")
                            .or_else(|| item.get("thought_signature"))
                            .or_else(|| item.get("signature"))
                            .and_then(Value::as_str)
                            .map(str::to_string);

                        // 若为 assistant 角色且没有任何实际正文、图像或思考元数据，属于纯空占位消息，予以过滤
                        if role == "assistant"
                            && joined_text.trim().is_empty()
                            && image_parts.is_empty()
                            && reasoning_content.is_none()
                            && signature.is_none()
                        {
                            continue;
                        }

                        // 构造消息内容：如果有图像则使用数组格式
                        let mut message = if image_parts.is_empty() {
                            let content = prefix_with_step_marker(step_marker, joined_text);
                            json!({
                                "role": role,
                                "content": content
                            })
                        } else {
                            let mut content_blocks: Vec<Value> = Vec::new();
                            let marker_text = prefix_with_step_marker(step_marker, joined_text);
                            if !marker_text.is_empty() {
                                content_blocks.push(json!({
                                    "type": "text",
                                    "text": marker_text
                                }));
                            }
                            content_blocks.extend(image_parts);
                            json!({
                                "role": role,
                                "content": content_blocks
                            })
                        };

                        if let Some(rc) = reasoning_content {
                            if let Some(obj) = message.as_object_mut() {
                                obj.insert("reasoning_content".to_string(), json!(rc));
                            }
                        }
                        if let Some(sig) = signature {
                            if let Some(obj) = message.as_object_mut() {
                                obj.insert("thoughtSignature".to_string(), json!(sig));
                            }
                        }

                        messages.push(message);
                    }
                    "reasoning" => {
                        let mut thought_text = String::new();
                        if let Some(summary_arr) = item.get("summary").and_then(Value::as_array) {
                            for s in summary_arr {
                                if let Some(t) = s.get("text").and_then(Value::as_str) {
                                    thought_text.push_str(t);
                                }
                            }
                        }
                        if thought_text.is_empty() {
                            if let Some(t) = item
                                .get("text")
                                .or_else(|| item.get("thought"))
                                .and_then(Value::as_str)
                            {
                                thought_text.push_str(t);
                            }
                        }
                        let sig = item
                            .get("thoughtSignature")
                            .or_else(|| item.get("thought_signature"))
                            .or_else(|| item.get("signature"))
                            .and_then(Value::as_str)
                            .map(str::to_string);

                        let mut msg_obj = json!({
                            "role": "assistant",
                            "content": "",
                            "reasoning_content": thought_text,
                        });
                        if let Some(s) = sig {
                            msg_obj["thoughtSignature"] = json!(s);
                        }
                        messages.push(msg_obj);
                    }
                    "function_call" | "custom_tool_call" | "local_shell_call"
                    | "web_search_call" => {
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

                        // Handle native shell calls
                        if item_type == "custom_tool_call" {
                            if let Some(input) = item.get("input").and_then(|v| v.as_str()) {
                                args_str = serde_json::to_string(&json!({ "input": input }))
                                    .unwrap_or_else(|_| "{}".to_string());
                            }
                        } else if item_type == "local_shell_call" {
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
                                        .unwrap_or("{}".to_string());
                                }
                            }
                        } else if item_type == "web_search_call" {
                            name = "google_search";
                            if let Some(action) = item.get("action") {
                                let mut args_obj = serde_json::Map::new();
                                if let Some(q) = action.get("query") {
                                    args_obj.insert("query".to_string(), q.clone());
                                }
                                args_str =
                                    serde_json::to_string(&args_obj).unwrap_or("{}".to_string());
                            }
                        }

                        let tool_sig = item
                            .get("thoughtSignature")
                            .or_else(|| item.get("thought_signature"))
                            .or_else(|| item.get("signature"))
                            .and_then(Value::as_str)
                            .map(str::to_string);

                        let mut tc_obj = json!({
                            "id": call_id,
                            "type": "function",
                            "function": {
                                "name": name,
                                "arguments": args_str
                            }
                        });
                        if let Some(ref s) = tool_sig {
                            tc_obj["thoughtSignature"] = json!(s);
                        }

                        let mut message = json!({
                            "role": "assistant",
                            "content": "",
                            "tool_calls": [ tc_obj ]
                        });
                        if let Some(ref s) = tool_sig {
                            message["thoughtSignature"] = json!(s);
                        }
                        messages.push(message);
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

                        let name = if let Some(name) = call_id_to_name.get(&call_id).cloned() {
                            name
                        } else if item_type == "custom_tool_call_output" {
                            tracing::warn!(
                                "Skipping orphan custom_tool_call_output for unknown call_id {}",
                                call_id
                            );
                            continue;
                        } else {
                            tracing::warn!(
                                "Unknown function_call_output tool name for call_id {}, defaulting to 'shell'",
                                call_id
                            );
                            "shell".to_string()
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
        if let Some(obj) = body.as_object_mut() {
            obj.insert("messages".to_string(), Value::Array(messages));
            if let Some(ledger) = interaction_ledger {
                obj.insert("_interaction_ledger".to_string(), json!(ledger));
            }
        }
    } else if let Some(prompt_val) = body.get("prompt") {
        // Legacy OpenAI Style: prompt -> Chat
        let prompt_str = match prompt_val {
            Value::String(s) => s.clone(),
            Value::Array(arr) => arr
                .iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join("\n"),
            _ => prompt_val.to_string(),
        };
        let messages = json!([ { "role": "user", "content": prompt_str } ]);
        if let Some(obj) = body.as_object_mut() {
            obj.remove("prompt");
            obj.insert("messages".to_string(), messages);
        }
    }

    prep.body = body;
    prep.session_delta_input = session_delta_input;
    prep.bounded_session_input = bounded_session_input;
    prep
}
