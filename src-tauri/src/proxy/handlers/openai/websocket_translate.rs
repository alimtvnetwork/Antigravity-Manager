// Translate OpenAI chunks to Codex websocket events.
use axum::extract::ws::{Message, WebSocket};
use futures::SinkExt;
use serde_json::{json, Value};
use tracing::error;

use super::websocket_finalize::{finalize_ws_events, split_namespace_tool_name};
use crate::proxy::handlers::openai::tool_cache::insert_cached_tool_call;

pub(crate) struct TranslationState {
    pub(crate) response_id: String,
    pub(crate) item_id: String,
    pub(crate) message_output_index: Option<u32>,
    pub(crate) next_output_index: u32,
    pub(crate) tool_output_indices: std::collections::HashMap<u32, u32>,
    pub(crate) message_item_added: bool,
    pub(crate) content_part_added: bool,
    pub(crate) accumulated_text: String,
    pub(crate) tool_calls: std::collections::HashMap<u32, (String, String, String, String)>,
    pub(crate) tool_calls_added: std::collections::HashSet<u32>,
}

pub(crate) async fn send_ws_event(
    socket: &mut WebSocket,
    ws_events: &mut Vec<Value>,
    event: &Value,
) {
    ws_events.push(event.clone());
    // Justification: best-effort send; failure logged without changing control flow
    crate::error::record_ignored(
        socket.send(Message::Text(event.to_string())).await,
        "send via send",
    );
}

pub(crate) async fn translate_openai_chunk_to_ws(
    chunk: &Value,
    state: &mut TranslationState,
    socket: &mut WebSocket,
    ws_events: &mut Vec<Value>,
) {
    if let Some(choices) = chunk.get("choices").and_then(|c| c.as_array()) {
        for choice in choices {
            if let Some(delta) = choice.get("delta") {
                if let Some(reasoning) = delta.get("reasoning_content").and_then(|v| v.as_str()) {
                    if !reasoning.is_empty() {
                        let message_output_index = match state.message_output_index {
                            Some(idx) => idx,
                            None => {
                                let idx = state.next_output_index;
                                state.next_output_index += 1;
                                state.message_output_index = Some(idx);
                                idx
                            }
                        };
                        let reasoning_ev = json!({
                            "type": "response.reasoning_summary_text.delta",
                            "sequence_number": 0,
                            "item_id": &state.item_id,
                            "output_index": message_output_index,
                            "summary_index": 0,
                            "delta": reasoning
                        });
                        send_ws_event(socket, ws_events, &reasoning_ev).await;

                        if !state.message_item_added {
                            let item_added = json!({
                                "type": "response.output_item.added",
                                "output_index": message_output_index,
                                "item": {
                                    "id": &state.item_id,
                                    "type": "message",
                                    "role": "assistant",
                                    "phase": "commentary",
                                    "status": "in_progress",
                                    "content": []
                                }
                            });
                            send_ws_event(socket, ws_events, &item_added).await;

                            let part_added = json!({
                                "type": "response.content_part.added",
                                "item_id": &state.item_id,
                                "output_index": message_output_index,
                                "content_index": 0,
                                "part": {
                                    "type": "output_text",
                                    "text": ""
                                }
                            });
                            send_ws_event(socket, ws_events, &part_added).await;
                            state.message_item_added = true;
                            state.content_part_added = true;
                        }

                        let delta_ev = json!({
                            "type": "response.output_text.delta",
                            "item_id": &state.item_id,
                            "output_index": message_output_index,
                            "content_index": 0,
                            "delta": reasoning
                        });
                        send_ws_event(socket, ws_events, &delta_ev).await;
                        state.accumulated_text.push_str(reasoning);
                    }
                }

                if let Some(content) = delta.get("content").and_then(|v| v.as_str()) {
                    if !content.is_empty() {
                        let message_output_index = match state.message_output_index {
                            Some(idx) => idx,
                            None => {
                                let idx = state.next_output_index;
                                state.next_output_index += 1;
                                state.message_output_index = Some(idx);
                                idx
                            }
                        };
                        if !state.message_item_added {
                            let item_added = json!({
                                "type": "response.output_item.added",
                                "output_index": message_output_index,
                                "item": {
                                    "id": &state.item_id,
                                    "type": "message",
                                    "role": "assistant",
                                    "phase": "commentary",
                                    "status": "in_progress",
                                    "content": []
                                }
                            });
                            send_ws_event(socket, ws_events, &item_added).await;

                            let part_added = json!({
                                "type": "response.content_part.added",
                                "item_id": &state.item_id,
                                "output_index": message_output_index,
                                "content_index": 0,
                                "part": {
                                    "type": "output_text",
                                    "text": ""
                                }
                            });
                            send_ws_event(socket, ws_events, &part_added).await;
                            state.message_item_added = true;
                            state.content_part_added = true;
                        }

                        let delta_ev = json!({
                            "type": "response.output_text.delta",
                            "item_id": &state.item_id,
                            "output_index": message_output_index,
                            "content_index": 0,
                            "delta": content
                        });
                        send_ws_event(socket, ws_events, &delta_ev).await;
                        state.accumulated_text.push_str(content);
                    }
                }

                if let Some(tool_calls) = delta.get("tool_calls").and_then(|v| v.as_array()) {
                    for tc in tool_calls {
                        let tc_idx = tc.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                        let tc_id = tc.get("id").and_then(|v| v.as_str()).unwrap_or("");
                        let tc_name = tc
                            .get("function")
                            .and_then(|f| f.get("name"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let tc_args = tc
                            .get("function")
                            .and_then(|f| f.get("arguments"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("");

                        if !tc_id.is_empty() || !tc_name.is_empty() {
                            let tool_item_id =
                                format!("item-{}", &Uuid::new_v4().to_string()[..16]);
                            let call_id = if tc_id.is_empty() {
                                format!("call_{}", &Uuid::new_v4().to_string()[..16])
                            } else {
                                tc_id.to_string()
                            };
                            state.tool_calls.insert(
                                tc_idx,
                                (
                                    tool_item_id,
                                    call_id.clone(),
                                    tc_name.to_string(),
                                    String::new(),
                                ),
                            );
                            if !tc_name.is_empty() {
                                // 临时插入一个包含 name 的 Value，最终会被 finalize_ws_events 里的完整 Value 覆盖
                                insert_cached_tool_call(call_id, json!({ "name": tc_name }));
                            }
                        }

                        if let Some((tool_item_id, call_id, name, args)) =
                            state.tool_calls.get_mut(&tc_idx)
                        {
                            args.push_str(tc_args);
                            let tool_output_index = match state.tool_output_indices.get(&tc_idx) {
                                Some(idx) => *idx,
                                None => {
                                    let idx = state.next_output_index;
                                    state.next_output_index += 1;
                                    state.tool_output_indices.insert(tc_idx, idx);
                                    idx
                                }
                            };

                            if !state.tool_calls_added.contains(&tc_idx) {
                                let (actual_name, namespace) = split_namespace_tool_name(name);
                                let mut item_obj = serde_json::json!({
                                    "id": tool_item_id,
                                    "type": "function_call",
                                    "status": "in_progress",
                                    "name": actual_name,
                                    "call_id": call_id,
                                    "arguments": ""
                                });
                                if let Some(ns) = namespace {
                                    item_obj["namespace"] = json!(ns);
                                }
                                let tool_added = json!({
                                    "type": "response.output_item.added",
                                    "output_index": tool_output_index,
                                    "item": item_obj
                                });
                                send_ws_event(socket, ws_events, &tool_added).await;
                                state.tool_calls_added.insert(tc_idx);
                            }

                            if !tc_args.is_empty() {
                                let args_delta = json!({
                                    "type": "response.function_call_arguments.delta",
                                    "item_id": tool_item_id,
                                    "output_index": tool_output_index,
                                    "delta": tc_args
                                });
                                send_ws_event(socket, ws_events, &args_delta).await;
                            }
                        }
                    }
                }
            }
        }
    }
}
