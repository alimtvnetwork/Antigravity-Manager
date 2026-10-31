use super::*;

// ===== [Issue #467 Fix] Warmup request interception =====

/// Detect whether it is a real Claude Code keep-alive heartbeat request (extremely narrow the rules to prevent accidental killing)
///
/// Only if the last message is a user role and the content is strictly equal to the word "Warmup" itself and does not contain any tool calls or redundant content,
/// It is recognized as the client heartbeat. Never use starts_with matching and never intercept ToolResults.
pub(crate) fn is_warmup_request(request: &ClaudeRequest) -> bool {
    if let Some(msg) = request.messages.last() {
        if msg.role != "user" {
            return false;
        }
        match &msg.content {
            crate::proxy::mappers::claude::models::MessageContent::String(s) => {
                s.trim().eq_ignore_ascii_case("warmup")
            }
            crate::proxy::mappers::claude::models::MessageContent::Array(arr) => {
                if arr.len() == 1 {
                    if let crate::proxy::mappers::claude::models::ContentBlock::Text { text } =
                        &arr[0]
                    {
                        text.trim().eq_ignore_ascii_case("warmup")
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
        }
    } else {
        false
    }
}

/// Create a mock response to a Warmup request
///
/// Return a simple response that consumes no upstream quota
pub(crate) fn create_warmup_response(request: &ClaudeRequest, is_stream: bool) -> Response {
    let model = &request.model;
    let message_id = format!("msg_warmup_{}", chrono::Utc::now().timestamp_millis());

    if is_stream {
        // Streaming response: Sends a standard sequence of SSE events
        let events = vec![
            // message_start
            format!(
                "event: message_start\ndata: {{\"type\":\"message_start\",\"message\":{{\"id\":\"{}\",\"type\":\"message\",\"role\":\"assistant\",\"content\":[],\"model\":\"{}\",\"stop_reason\":null,\"stop_sequence\":null,\"usage\":{{\"input_tokens\":1,\"output_tokens\":0}}}}}}\n\n",
                message_id, model
            ),
            // content_block_start
            "event: content_block_start\ndata: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n\n".to_string(),
            // content_block_delta
            "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"OK\"}}\n\n".to_string(),
            // content_block_stop
            "event: content_block_stop\ndata: {\"type\":\"content_block_stop\",\"index\":0}\n\n".to_string(),
            // message_delta
            "event: message_delta\ndata: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\",\"stop_sequence\":null},\"usage\":{\"output_tokens\":1}}\n\n".to_string(),
            // message_stop
            "event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n".to_string(),
        ];

        let body = events.join("");

        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/event-stream")
            .header(header::CACHE_CONTROL, "no-cache")
            .header(header::CONNECTION, "keep-alive")
            .header("X-Warmup-Intercepted", "true")
            .body(Body::from(body))
            .unwrap()
    } else {
        // non-streaming response
        let response = json!({
            "id": message_id,
            "type": "message",
            "role": "assistant",
            "content": [{
                "type": "text",
                "text": "OK"
            }],
            "model": model,
            "stop_reason": "end_turn",
            "stop_sequence": null,
            "usage": {
                "input_tokens": 1,
                "output_tokens": 1
            }
        });

        (
            StatusCode::OK,
            [("X-Warmup-Intercepted", "true")],
            Json(response),
        )
            .into_response()
    }
}
