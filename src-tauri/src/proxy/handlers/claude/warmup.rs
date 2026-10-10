use super::*;

// ===== [Issue #467 Fix] Warmup 请求拦截 =====

/// 检测是否为真正的 Claude Code 保活心跳请求（极度收窄规则，杜绝误杀）
///
/// 只有当最后一条消息为用户角色且内容严格全等于 "Warmup" 单词本身，且不包含任何工具调用或多余内容时，
/// 才认定为客户端心跳。绝不使用 starts_with 匹配，绝不拦截 ToolResult。
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

/// 创建 Warmup 请求的模拟响应
///
/// 返回一个简单的响应，不消耗上游配额
pub(crate) fn create_warmup_response(request: &ClaudeRequest, is_stream: bool) -> Response {
    let model = &request.model;
    let message_id = format!("msg_warmup_{}", chrono::Utc::now().timestamp_millis());

    if is_stream {
        // 流式响应：发送标准的 SSE 事件序列
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
        // 非流式响应
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
