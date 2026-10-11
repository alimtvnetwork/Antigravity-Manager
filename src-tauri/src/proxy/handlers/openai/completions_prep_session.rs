// `handle_completions` prelude phase 1: previous_response_id session restore.
use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use serde_json::{json, Value};
use tracing::debug;

use super::chat_conversion::responses_routing_session_id;
use super::responses_history::omit_media_before_latest_user_turn;
use super::responses_media::validate_responses_input_image_limits;
use crate::proxy::http_session_store::SessionParent;
use axum::response::IntoResponse;

pub(crate) struct CompletionsSessionPrep {
    pub body: Value,
    pub previous_response_id: Option<String>,
    pub explicit_session_id: Option<String>,
    pub response_id_for_save: String,
    pub session_parent: Option<SessionParent>,
    pub stored_routing_session_id: Option<String>,
    pub session_delta_input: Vec<Value>,
    pub routing_session_id: String,
    pub signature_read_key: Option<String>,
    pub bounded_session_input: Option<Vec<Value>>,
}

pub(crate) async fn completions_prep_session(
    headers: &HeaderMap,
    body: Value,
    is_codex_style: bool,
    store_response: bool,
) -> Result<CompletionsSessionPrep, Response> {
    let mut body = body;
    // [MULTI-TURN] 支持 previous_response_id 链式历史恢复
    // 当客户端通过 HTTP POST /v1/responses 传入 previous_response_id 时，
    // 从服务器端 session store 取出上一轮的历史，合并到本轮的 input 中
    let previous_response_id = body
        .get("previous_response_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let explicit_session_id = headers
        .get("x-session-id")
        .or_else(|| headers.get("session-id"))
        .or_else(|| headers.get("x-claude-code-session-id"))
        .and_then(|h| h.to_str().ok())
        .map(|s| s.trim().to_string())
        .filter(|id| !id.is_empty())
        .or_else(|| {
            body.get("session_id")
                .and_then(Value::as_str)
                .filter(|id| !id.is_empty())
                .map(str::to_string)
        });
    let response_id_for_save = format!("resp-{}", uuid::Uuid::new_v4());
    let http_tool_call_cache: std::collections::HashMap<String, serde_json::Value> =
        std::collections::HashMap::new();
    let mut session_parent = None;
    let mut stored_routing_session_id = None;
    let mut session_delta_input = Vec::new();
    if is_codex_style {
        let mut existing_input = body
            .as_object_mut()
            .and_then(|obj| obj.remove("input"))
            .and_then(|value| match value {
                Value::Array(items) => Some(items),
                _ => None,
            })
            .unwrap_or_default();
        // 完整回放先裁掉最新用户轮次之前的内联媒体，再执行硬限制校验。
        omit_media_before_latest_user_turn(&mut existing_input);

        let merged = if let Some(ref prev_id) = previous_response_id {
            if let Some((session, parent)) =
                crate::proxy::http_session_store::get_session_with_parent(prev_id).await
            {
                stored_routing_session_id = Some(parent.routing_session_id().to_string());
                let prepared = crate::proxy::http_session_store::prepare_session_input_with_storage(
                    session.input_items,
                    existing_input,
                    &http_tool_call_cache,
                    store_response,
                );
                session_delta_input = prepared.delta;
                if store_response && !prepared.reset_parent {
                    session_parent = Some(parent);
                }
                if let Some(obj) = body.as_object_mut() {
                    if !obj.contains_key("instructions") && !session.instructions.is_empty() {
                        obj.insert("instructions".to_string(), json!(session.instructions));
                    }
                    if !obj.contains_key("model") && !session.model.is_empty() {
                        obj.insert("model".to_string(), json!(session.model));
                    }
                }
                tracing::debug!(
                    "[MultiTurn] Restored session from prev_id={}, {} items in history",
                    prev_id,
                    prepared.merged.len()
                );
                prepared.merged
            } else {
                if store_response {
                    session_delta_input = existing_input.clone();
                }
                existing_input
            }
        } else {
            if store_response {
                session_delta_input = existing_input.clone();
            }
            existing_input
        };

        if let Some(obj) = body.as_object_mut() {
            obj.insert("input".to_string(), Value::Array(merged));
        }
        if let Err(message) = validate_responses_input_image_limits(body.get("input")) {
            return Err((StatusCode::BAD_REQUEST, message).into_response());
        }
    }
    let routing_session_id = responses_routing_session_id(
        explicit_session_id.as_deref(),
        previous_response_id.as_deref(),
        stored_routing_session_id.as_deref(),
        &response_id_for_save,
    );
    let signature_read_key = previous_response_id.clone();

    let mut bounded_session_input = None;

    Ok(CompletionsSessionPrep {
        body,
        previous_response_id,
        explicit_session_id,
        response_id_for_save,
        session_parent,
        stored_routing_session_id,
        session_delta_input,
        routing_session_id,
        signature_read_key,
        bounded_session_input,
    })
}
