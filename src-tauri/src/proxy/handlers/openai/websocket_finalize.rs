// Finalize Codex websocket event streams.
use axum::extract::ws::{Message, WebSocket};
use futures::SinkExt;
use serde_json::{json, Value};

use super::websocket_translate::{send_ws_event, TranslationState};

pub(crate) async fn finalize_ws_events(
    state: &mut TranslationState,
    socket: &mut WebSocket,
    session_state: &mut WebsocketSessionState,
    ws_events: &mut Vec<Value>,
) -> Value {
    let mut output_items = Vec::new();
    let mut tool_keys: Vec<u32> = state.tool_calls.keys().cloned().collect();
    tool_keys.sort();

    for tc_idx in tool_keys {
        if let Some((tool_item_id, call_id, name, args)) = state.tool_calls.get(&tc_idx) {
            let tool_output_index = match state.tool_output_indices.get(&tc_idx) {
                Some(idx) => *idx,
                None => {
                    let idx = state.next_output_index;
                    state.next_output_index += 1;
                    state.tool_output_indices.insert(tc_idx, idx);
                    idx
                }
            };
            let args_done = json!({
                "type": "response.function_call_arguments.done",
                "item_id": tool_item_id,
                "output_index": tool_output_index,
                "arguments": args
            });
            send_ws_event(socket, ws_events, &args_done).await;

            let (actual_name, namespace) = split_namespace_tool_name(name);
            let mut item_obj = serde_json::json!({
                "id": tool_item_id,
                "type": "function_call",
                "status": "completed",
                "name": actual_name,
                "call_id": call_id,
                "arguments": args
            });
            if let Some(ns) = namespace {
                item_obj["namespace"] = json!(ns);
            }

            let tool_done = json!({
                "type": "response.output_item.done",
                "output_index": tool_output_index,
                "item": item_obj
            });
            send_ws_event(socket, ws_events, &tool_done).await;

            let tc_val = item_obj.clone();

            session_state
                .tool_call_cache
                .insert(call_id.clone(), tc_val.clone());
            insert_cached_tool_call(call_id.clone(), tc_val.clone());
            output_items.push(tc_val);
        }
    }

    if state.message_item_added {
        let message_output_index = state.message_output_index.unwrap_or(0);
        let text_done = json!({
            "type": "response.output_text.done",
            "item_id": &state.item_id,
            "output_index": message_output_index,
            "content_index": 0,
            "text": &state.accumulated_text
        });
        send_ws_event(socket, ws_events, &text_done).await;

        let part_done = json!({
            "type": "response.content_part.done",
            "item_id": &state.item_id,
            "output_index": message_output_index,
            "content_index": 0,
            "part": {
                "type": "output_text",
                "text": &state.accumulated_text
            }
        });
        send_ws_event(socket, ws_events, &part_done).await;

        let message_done = json!({
            "type": "response.output_item.done",
            "output_index": message_output_index,
            "item": {
                "id": &state.item_id,
                "type": "message",
                "role": "assistant",
                "phase": "final_answer",
                "status": "completed",
                "content": [{
                    "type": "output_text",
                    "text": &state.accumulated_text
                }]
            }
        });
        send_ws_event(socket, ws_events, &message_done).await;

        output_items.push(json!({
            "id": &state.item_id,
            "type": "message",
            "role": "assistant",
            "phase": "final_answer",
            "status": "completed",
            "content": [{
                "type": "output_text",
                "text": &state.accumulated_text
            }]
        }));
    }

    let completed_ev = json!({
        "type": "response.completed",
        "response": {
            "id": &state.response_id,
            "object": "response",
            "status": "completed",
            "output": output_items
        }
    });
    send_ws_event(socket, ws_events, &completed_ev).await;

    json!(output_items)
}

pub(crate) fn split_namespace_tool_name(qualified_name: &str) -> (String, Option<String>) {
    let name = qualified_name.trim();
    if name.starts_with("mcp__") {
        return (name.to_string(), None);
    }
    if let Some(pos) = name.find("__") {
        if pos > 0 {
            let namespace = name[..pos].to_string();
            let actual_name = name[pos + 2..].to_string();
            return (actual_name, Some(namespace));
        }
    }
    (name.to_string(), None)
}
