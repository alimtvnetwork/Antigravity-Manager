// `GET /v1/responses` WebSocket handler.
use std::collections::HashSet;
use std::sync::Arc;

use axum::{
    body::Body, extract::ws::Message, extract::ws::WebSocket, extract::State,
    extract::WebSocketUpgrade, http::StatusCode, response::IntoResponse, response::Response,
};
use futures::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tracing::{debug, error, info, warn};

use crate::proxy::config::DebugLoggingConfig;
use crate::proxy::debug_logger;
use crate::proxy::handlers::common::FailureStatusTracker;
use crate::proxy::mappers::openai::OpenAIRequest;
use crate::proxy::server::AppState;
use crate::proxy::thinking_store::SessionScope;

use super::responses_history::serialized_json_len;
use super::websocket_codex::convert_codex_to_openai_request;
use super::websocket_finalize::finalize_ws_events;
use super::websocket_translate::{send_ws_event, translate_openai_chunk_to_ws, TranslationState};
use crate::proxy::handlers::openai::responses_history::debug_value_without_inline_data;
use crate::proxy::handlers::openai::responses_history::into_history_without_inline_media;
use crate::proxy::handlers::openai::websocket_normalize::should_handle_prewarm_locally;
use crate::proxy::handlers::openai::websocket_normalize::handle_prewarm_locally;
use axum::http::HeaderMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub(crate) struct WebsocketSessionState {
    pub(crate) last_request: Option<Value>,
    pub(crate) last_response_output: Value,
    pub(crate) last_response_id: String,
    pub(crate) last_response_pending_tool_call_ids: Vec<String>,
    pub(crate) tool_call_cache: std::collections::HashMap<String, Value>,
}

pub async fn handle_responses_websocket(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_websocket_session(socket, headers, state))
}

async fn handle_websocket_session(mut socket: WebSocket, headers: HeaderMap, state: AppState) {
    tracing::info!("Codex responses websocket: client connected");
    let mut session_state = WebsocketSessionState {
        last_request: None,
        last_response_output: json!([]),
        last_response_id: String::new(),
        last_response_pending_tool_call_ids: Vec::new(),
        tool_call_cache: std::collections::HashMap::new(),
    };

    while let Some(msg_result) = socket.recv().await {
        let msg = match msg_result {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("responses websocket: read message failed: {:?}", e);
                break;
            }
        };

        let text = match msg {
            Message::Text(t) => t,
            Message::Binary(b) => match String::from_utf8(b) {
                Ok(s) => s,
                Err(_) => continue,
            },
            Message::Close(_) => {
                tracing::info!("responses websocket: client disconnected");
                break;
            }
            _ => continue,
        };

        let payload: Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(e) => {
                let error_ev = json!({
                    "type": "error",
                    "error": {
                        "message": format!("Invalid JSON: {}", e),
                        "type": "invalid_request_error"
                    }
                });
                // Justification: best-effort send; failure logged without changing control flow
                crate::error::record_ignored(
                    socket.send(Message::Text(error_ev.to_string())).await,
                    "send via send",
                );
                continue;
            }
        };
        drop(text);
        let ws_trace_id = format!("ws_{}", chrono::Utc::now().timestamp_subsec_millis());
        let debug_cfg = state.debug_logging.read().await.clone();
        if debug_logger::is_enabled(&debug_cfg) {
            let payload_log = json!({
                "kind": "codex_websocket_raw_request",
                "protocol": "codex_websocket",
                "trace_id": ws_trace_id,
                "payload": debug_value_without_inline_data(&payload),
            });
            debug_logger::write_exchange_payload(
                &debug_cfg,
                Some(&ws_trace_id),
                "codex_websocket_raw_request",
                &payload_log,
            )
            .await;
        }

        if should_handle_prewarm_locally(&payload, &session_state) {
            let (created, completed) = handle_prewarm_locally(&payload, &mut session_state);
            // Justification: best-effort send; failure logged without changing control flow
            crate::error::record_ignored(
                socket.send(Message::Text(created.to_string())).await,
                "send via send",
            );
            // Justification: best-effort send; failure logged without changing control flow
            crate::error::record_ignored(
                socket.send(Message::Text(completed.to_string())).await,
                "send via send",
            );
            if debug_logger::is_enabled(&debug_cfg) {
                let payload_log = json!({
                    "kind": "codex_websocket_local_response",
                    "protocol": "codex_websocket",
                    "trace_id": ws_trace_id,
                    "events": [created, completed],
                });
                debug_logger::write_exchange_payload(
                    &debug_cfg,
                    Some(&ws_trace_id),
                    "codex_websocket_local_response",
                    &payload_log,
                )
                .await;
            }
            continue;
        }

        let normalized = match normalize_responses_websocket_request(payload, &mut session_state) {
            Ok(n) => n,
            Err(e) => {
                let error_ev = json!({
                    "type": "error",
                    "error": {
                        "message": e,
                        "type": "invalid_request_error"
                    }
                });
                // Justification: best-effort send; failure logged without changing control flow
                crate::error::record_ignored(
                    socket.send(Message::Text(error_ev.to_string())).await,
                    "send via send",
                );
                continue;
            }
        };

        let openai_body = convert_codex_to_openai_request(normalized);
        let response_result = handle_chat_completions(
            State(state.clone()),
            headers.clone(),
            None,
            Json(openai_body),
        )
        .await;

        let response = match response_result {
            Ok(res) => res.into_response(),
            Err((status, err_msg)) => {
                let error_ev = json!({
                    "type": "error",
                    "error": {
                        "message": err_msg,
                        "type": "server_error",
                        "code": status.as_u16().to_string()
                    }
                });
                // Justification: best-effort send; failure logged without changing control flow
                crate::error::record_ignored(
                    socket.send(Message::Text(error_ev.to_string())).await,
                    "send via send",
                );
                continue;
            }
        };

        if !response.status().is_success() {
            let error_ev = json!({
                "type": "error",
                "error": {
                    "message": format!("Upstream returned status {}", response.status()),
                    "type": "server_error"
                }
            });
            // Justification: best-effort send; failure logged without changing control flow
            crate::error::record_ignored(
                socket.send(Message::Text(error_ev.to_string())).await,
                "send via send",
            );
            continue;
        }

        let body = response.into_body();
        let mut stream = body.into_data_stream();

        let mut translation_state = TranslationState {
            response_id: format!("resp-{}", &Uuid::new_v4().to_string()[..24]),
            item_id: format!("item-{}", &Uuid::new_v4().to_string()[..16]),
            message_output_index: None,
            next_output_index: 0,
            tool_output_indices: std::collections::HashMap::new(),
            message_item_added: false,
            content_part_added: false,
            accumulated_text: String::new(),
            tool_calls: std::collections::HashMap::new(),
            tool_calls_added: std::collections::HashSet::new(),
        };

        let created_ev = json!({
            "type": "response.created",
            "response": {
                "id": &translation_state.response_id,
                "object": "response",
                "status": "in_progress",
                "output": []
            }
        });
        let mut outgoing_ws_events = Vec::new();
        send_ws_event(&mut socket, &mut outgoing_ws_events, &created_ev).await;

        let mut buffer = bytes::BytesMut::new();
        while let Some(chunk_res) = stream.next().await {
            let chunk = match chunk_res {
                Ok(c) => c,
                Err(e) => {
                    tracing::warn!("Stream chunk error: {:?}", e);
                    break;
                }
            };
            buffer.extend_from_slice(&chunk);
            while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
                let line_raw = buffer.split_to(pos + 1);
                if let Ok(line_str) = std::str::from_utf8(&line_raw) {
                    let line = line_str.trim();
                    if line.is_empty() || !line.starts_with("data: ") {
                        continue;
                    }
                    let json_part = line.trim_start_matches("data: ").trim();
                    if json_part == "[DONE]" {
                        break;
                    }
                    if let Ok(chunk_json) = serde_json::from_str::<Value>(json_part) {
                        translate_openai_chunk_to_ws(
                            &chunk_json,
                            &mut translation_state,
                            &mut socket,
                            &mut outgoing_ws_events,
                        )
                        .await;
                    }
                }
            }
        }

        if !buffer.is_empty() {
            if let Ok(line_str) = std::str::from_utf8(&buffer) {
                let line = line_str.trim();
                if line.starts_with("data: ") {
                    let json_part = line.trim_start_matches("data: ").trim();
                    if json_part != "[DONE]" {
                        if let Ok(chunk_json) = serde_json::from_str::<Value>(json_part) {
                            translate_openai_chunk_to_ws(
                                &chunk_json,
                                &mut translation_state,
                                &mut socket,
                                &mut outgoing_ws_events,
                            )
                            .await;
                        }
                    }
                }
            }
        }

        let completed_output = finalize_ws_events(
            &mut translation_state,
            &mut socket,
            &mut session_state,
            &mut outgoing_ws_events,
        )
        .await;
        if debug_logger::is_enabled(&debug_cfg) {
            let payload_log = json!({
                "kind": "codex_websocket_converted_response",
                "protocol": "codex_websocket",
                "trace_id": ws_trace_id,
                "events": debug_value_without_inline_data(&Value::Array(outgoing_ws_events)),
                "completed_output": debug_value_without_inline_data(&completed_output),
            });
            debug_logger::write_exchange_payload(
                &debug_cfg,
                Some(&ws_trace_id),
                "codex_websocket_converted_response",
                &payload_log,
            )
            .await;
        }

        session_state.last_response_output =
            into_history_without_inline_media(completed_output).unwrap_or_else(|| json!([]));
        session_state.last_response_id = translation_state.response_id.clone();
        session_state.last_response_pending_tool_call_ids = translation_state
            .tool_calls
            .values()
            .map(|(_, call_id, _, _)| call_id.clone())
            .collect();
    }
}
