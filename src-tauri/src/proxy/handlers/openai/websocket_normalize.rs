// Normalize Codex-style websocket payloads to OpenAI requests.
use super::responses_history::history_without_inline_media;
use super::responses_media::{responses_input_item_type, validate_responses_input_image_limits};
use serde_json::{json, Value};

use super::websocket::WebsocketSessionState;

pub(crate) fn should_handle_prewarm_locally(
    payload: &Value,
    state: &WebsocketSessionState,
) -> bool {
    if state.last_request.is_some() {
        return false;
    }
    let event_type = payload.get("type").and_then(|v| v.as_str()).unwrap_or("");
    if event_type != "response.create" {
        return false;
    }
    if let Some(generate) = payload.get("generate").and_then(|v| v.as_bool()) {
        if !generate {
            return true;
        }
    }
    false
}

pub(crate) fn handle_prewarm_locally(
    payload: &Value,
    state: &mut WebsocketSessionState,
) -> (Value, Value) {
    let response_id = format!("resp_prewarm_{}", Uuid::new_v4());
    let created_at = chrono::Utc::now().timestamp();
    let model = payload
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let created_ev = json!({
        "type": "response.created",
        "sequence_number": 0,
        "response": {
            "id": &response_id,
            "object": "response",
            "created_at": created_at,
            "status": "in_progress",
            "background": false,
            "error": null,
            "output": [],
            "model": model,
        }
    });

    let completed_ev = json!({
        "type": "response.completed",
        "sequence_number": 1,
        "response": {
            "id": &response_id,
            "object": "response",
            "created_at": created_at,
            "status": "completed",
            "background": false,
            "error": null,
            "output": [],
            "usage": {
                "input_tokens": 0,
                "input_tokens_details": {
                    "cached_tokens": 0
                },
                "output_tokens": 0,
                "output_tokens_details": {
                    "reasoning_tokens": 0
                },
                "total_tokens": 0
            },
            "model": model,
        }
    });

    let mut normalized = history_without_inline_media(payload);
    if let Some(obj) = normalized.as_object_mut() {
        obj.remove("type");
        obj.remove("generate");
    }
    state.last_request = Some(normalized);
    state.last_response_output = json!([]);
    state.last_response_id = response_id;
    state.last_response_pending_tool_call_ids = Vec::new();

    (created_ev, completed_ev)
}

pub(crate) fn normalize_responses_websocket_request(
    mut payload: Value,
    state: &mut WebsocketSessionState,
) -> Result<Value, String> {
    let event_type = payload
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    match event_type.as_str() {
        "response.create" => {
            if state.last_request.is_none() {
                if let Some(obj) = payload.as_object_mut() {
                    obj.remove("type");
                    obj.insert("stream".to_string(), Value::Bool(true));
                    if !obj.contains_key("input") {
                        obj.insert("input".to_string(), json!([]));
                    }
                }
                let model_name = payload.get("model").and_then(|v| v.as_str()).unwrap_or("");
                if model_name.is_empty() {
                    return Err("missing model in response.create request".to_string());
                }
                validate_responses_input_image_limits(payload.get("input"))?;
                state.last_request = Some(history_without_inline_media(&payload));
                Ok(payload)
            } else {
                normalize_response_subsequent_request(payload, state)
            }
        }
        "response.append" => normalize_response_subsequent_request(payload, state),
        _ => Err(format!(
            "unsupported websocket request type: {}",
            event_type
        )),
    }
}

fn normalize_response_subsequent_request(
    mut payload: Value,
    state: &mut WebsocketSessionState,
) -> Result<Value, String> {
    if state.last_request.is_none() {
        return Err("websocket request received before response.create".to_string());
    }
    validate_responses_input_image_limits(payload.get("input"))?;

    // [FIX] 拦截 compaction 和完整历史替换事件
    if should_replace_websocket_transcript(&payload) {
        if let Some(obj) = payload.as_object_mut() {
            obj.remove("type");
            obj.remove("previous_response_id");
            obj.insert("stream".to_string(), Value::Bool(true));
        }
        state.last_request = Some(history_without_inline_media(&payload));
        return Ok(payload);
    }

    // [FIX] 始终走完整的 merge 逻辑，废弃 transcript replacement 分支
    // 旧逻辑在检测到 function_call/assistant 时直接替换整个历史，导致多轮对话历史丢失
    // 正确做法：last_request.input + last_response_output + new payload.input 全部合并
    let mut last_request = state.last_request.take().expect("checked above");
    let mut merged_input = last_request
        .as_object_mut()
        .and_then(|obj| obj.remove("input"))
        .and_then(|value| match value {
            Value::Array(items) => Some(items),
            _ => None,
        })
        .unwrap_or_default();

    // 上一轮请求的 input 已按所有权移入 merged_input。
    // 2. 上一轮 response 的 output items（assistant 回复、工具调用等）
    if let Value::Array(items) = std::mem::take(&mut state.last_response_output) {
        merged_input.extend(items);
    }

    // 3. 本轮新的 input items（用户消息、工具调用结果等）
    let current_input = payload
        .as_object_mut()
        .and_then(|obj| obj.remove("input"))
        .and_then(|value| match value {
            Value::Array(items) => Some(items),
            _ => None,
        })
        .unwrap_or_default();
    for item in current_input {
        let t = item.get("type").and_then(|v| v.as_str()).unwrap_or("");
        if t == "compaction" || t == "compaction_summary" {
            continue;
        }
        if t == "function_call_output" || t == "custom_tool_call_output" {
            if let Some(call_id) = item.get("call_id").and_then(|v| v.as_str()) {
                state
                    .last_response_pending_tool_call_ids
                    .retain(|x| x != call_id);
            }
        }
        merged_input.push(item);
    }

    repair_tool_calls(&mut merged_input, &state.tool_call_cache);

    let deduped = dedupe_function_calls_by_call_id(dedupe_input_items_by_id(merged_input));

    if let Some(obj) = payload.as_object_mut() {
        obj.remove("type");
        obj.remove("previous_response_id");
        obj.insert("input".to_string(), Value::Array(deduped));
        if !obj.contains_key("model") {
            if let Some(model) = last_request.get_mut("model").map(Value::take) {
                obj.insert("model".to_string(), model);
            }
        }
        if !obj.contains_key("instructions") {
            if let Some(instructions) = last_request.get_mut("instructions").map(Value::take) {
                obj.insert("instructions".to_string(), instructions);
            }
        }
        if !obj.contains_key("tools") {
            if let Some(tools) = last_request.get_mut("tools").map(Value::take) {
                obj.insert("tools".to_string(), tools);
            }
        }
        if !obj.contains_key("tool_choice") {
            if let Some(tool_choice) = last_request.get_mut("tool_choice").map(Value::take) {
                obj.insert("tool_choice".to_string(), tool_choice);
            }
        }
        obj.insert("stream".to_string(), Value::Bool(true));
    }
    state.last_request = Some(history_without_inline_media(&payload));
    Ok(payload)
}

fn should_replace_websocket_transcript(payload: &Value) -> bool {
    let previous_response_id = payload
        .get("previous_response_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if !previous_response_id.is_empty() {
        return false;
    }
    if let Some(input_array) = payload.get("input").and_then(|v| v.as_array()) {
        for item in input_array {
            let item_type = responses_input_item_type(&item).to_string();
            if item_type == "function_call" || item_type == "custom_tool_call" {
                return true;
            }
            if item_type == "message" {
                let role = item.get("role").and_then(|v| v.as_str()).unwrap_or("");
                if role == "assistant" {
                    return true;
                }
            }
        }
    }
    false
}

fn dedupe_input_items_by_id(items: Vec<Value>) -> Vec<Value> {
    use std::collections::{HashMap, HashSet};
    let mut referenced_call_ids = HashSet::new();
    for item in &items {
        let item_type = item.get("type").and_then(|v| v.as_str()).unwrap_or("");
        if item_type == "function_call_output" || item_type == "custom_tool_call_output" {
            if let Some(call_id) = item.get("call_id").and_then(|v| v.as_str()) {
                if !call_id.is_empty() {
                    referenced_call_ids.insert(call_id.to_string());
                }
            }
        }
    }

    let mut keep_map: HashMap<String, (usize, bool)> = HashMap::new();
    for (idx, item) in items.iter().enumerate() {
        let item_id = item.get("id").and_then(|v| v.as_str()).unwrap_or("");
        if item_id.is_empty() {
            continue;
        }
        let call_id = item.get("call_id").and_then(|v| v.as_str()).unwrap_or("");
        let is_referenced = !call_id.is_empty() && referenced_call_ids.contains(call_id);
        if let Some(&(_existing_idx, existing_referenced)) = keep_map.get(item_id) {
            if is_referenced || !existing_referenced {
                keep_map.insert(item_id.to_string(), (idx, is_referenced));
            }
        } else {
            keep_map.insert(item_id.to_string(), (idx, is_referenced));
        }
    }

    let mut keep_indices = HashSet::new();
    for (_, (idx, _)) in keep_map {
        keep_indices.insert(idx);
    }

    let mut filtered = Vec::new();
    for (idx, item) in items.into_iter().enumerate() {
        let item_id = item.get("id").and_then(|v| v.as_str()).unwrap_or("");
        if !item_id.is_empty() {
            if !keep_indices.contains(&idx) {
                continue;
            }
        }
        filtered.push(item);
    }
    filtered
}

fn dedupe_function_calls_by_call_id(items: Vec<Value>) -> Vec<Value> {
    use std::collections::HashSet;
    use uuid::Uuid;
    let mut seen_call_ids = HashSet::new();
    let mut filtered = Vec::new();
    for item in items {
        let item_type = item.get("type").and_then(|v| v.as_str()).unwrap_or("");
        if item_type == "function_call" || item_type == "custom_tool_call" {
            if let Some(call_id) = item.get("call_id").and_then(|v| v.as_str()) {
                if !call_id.is_empty() {
                    if seen_call_ids.contains(call_id) {
                        continue;
                    }
                    seen_call_ids.insert(call_id.to_string());
                }
            }
        }
        filtered.push(item);
    }
    filtered
}

fn repair_tool_calls(
    input_items: &mut Vec<Value>,
    tool_call_cache: &std::collections::HashMap<String, Value>,
) {
    let mut call_present = std::collections::HashSet::new();
    for item in input_items.iter() {
        let item_type = item.get("type").and_then(|v| v.as_str()).unwrap_or("");
        if item_type == "function_call" || item_type == "custom_tool_call" {
            if let Some(call_id) = item.get("call_id").and_then(|v| v.as_str()) {
                call_present.insert(call_id.to_string());
            }
        }
    }

    let mut new_items = Vec::new();
    let mut inserted = std::collections::HashSet::new();
    for item in input_items.drain(..) {
        let item_type = item.get("type").and_then(|v| v.as_str()).unwrap_or("");
        if item_type == "function_call_output" || item_type == "custom_tool_call_output" {
            if let Some(call_id) = item.get("call_id").and_then(|v| v.as_str()) {
                if !call_id.is_empty()
                    && !call_present.contains(call_id)
                    && !inserted.contains(call_id)
                {
                    if let Some(cached_call) = tool_call_cache.get(call_id) {
                        new_items.push(cached_call.clone());
                        inserted.insert(call_id.to_string());
                    }
                }
            }
        }
        new_items.push(item);
    }
    *input_items = new_items;
}
