// Convert Codex websocket payloads to OpenAI requests.
use serde_json::{json, Value};

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
