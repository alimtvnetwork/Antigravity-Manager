// Chat->image redirection: `handle_chat_redirection` + `intercept_chat_to_image`.
use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    response::Response,
    Json,
};
use serde_json::{json, Value};
use tracing::{debug, error};

use super::chat_completions::handle_chat_completions;
use super::images_generations::handle_images_generations_internal;
use crate::proxy::server::AppState;

pub async fn handle_chat_redirection(
    State(state): State<AppState>,
    headers: HeaderMap,
    upstream_recorder: Option<
        axum::extract::Extension<crate::proxy::monitor::UpstreamRequestBodyHolder>,
    >,
    Json(body): Json<Value>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    handle_chat_completions(State(state), headers, upstream_recorder, Json(body)).await
}

pub(crate) async fn intercept_chat_to_image(
    state: AppState,
    body: Value,
    model_name: &str,
) -> Result<Response, (StatusCode, String)> {
    // 1. Extract prompt from messages
    let mut prompt = String::new();
    if let Some(messages) = body.get("messages").and_then(|v| v.as_array()) {
        for msg in messages {
            if msg.get("role").and_then(|v| v.as_str()) == Some("user") {
                if let Some(content) = msg.get("content") {
                    if let Some(s) = content.as_str() {
                        prompt = s.to_string();
                    } else if let Some(arr) = content.as_array() {
                        for part in arr {
                            if part.get("type").and_then(|v| v.as_str()) == Some("text") {
                                prompt.push_str(
                                    part.get("text").and_then(|v| v.as_str()).unwrap_or(""),
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    if prompt.is_empty() {
        prompt = "A beautiful painting".to_string(); // fallback
    }

    let is_stream = body
        .get("stream")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // 2. Call internal image generator
    let img_req = json!({
        "prompt": prompt,
        "model": model_name,
        "n": 1,
        "response_format": "url"
    });

    match handle_images_generations_internal(state, img_req).await {
        Ok((email, img_res)) => {
            // Extract URL
            let mut img_markdown = String::new();
            if let Some(data) = img_res.get("data").and_then(|v| v.as_array()) {
                for item in data {
                    if let Some(url) = item.get("url").and_then(|v| v.as_str()) {
                        img_markdown.push_str(&format!("![Generated Image]({})\n\n", url));
                    }
                }
            }

            if img_markdown.is_empty() {
                img_markdown = "Failed to extract image URL from generation result.".to_string();
            }

            // 3. Construct Chat Completion Response
            if is_stream {
                use axum::body::Body;
                use axum::http::HeaderMap;

                let chunk = json!({
                    "id": format!("chatcmpl-img-{}", uuid::Uuid::new_v4()),
                    "object": "chat.completion.chunk",
                    "created": chrono::Utc::now().timestamp(),
                    "model": model_name,
                    "choices": [{
                        "index": 0,
                        "delta": {
                            "role": "assistant",
                            "content": img_markdown
                        },
                        "finish_reason": null
                    }]
                });

                let done_chunk = json!({
                    "id": format!("chatcmpl-img-{}", uuid::Uuid::new_v4()),
                    "object": "chat.completion.chunk",
                    "created": chrono::Utc::now().timestamp(),
                    "model": model_name,
                    "choices": [{
                        "index": 0,
                        "delta": {},
                        "finish_reason": "stop"
                    }]
                });

                let sse_data = format!(
                    "data: {}\n\ndata: {}\n\ndata: [DONE]\n\n",
                    chunk.to_string(),
                    done_chunk.to_string()
                );

                let body = Body::from(sse_data);
                Ok(Response::builder()
                    .header("Content-Type", "text/event-stream")
                    .header("Cache-Control", "no-cache")
                    .header("X-Account-Email", email)
                    .body(body)
                    .unwrap())
            } else {
                let resp = json!({
                    "id": format!("chatcmpl-img-{}", uuid::Uuid::new_v4()),
                    "object": "chat.completion",
                    "created": chrono::Utc::now().timestamp(),
                    "model": model_name,
                    "choices": [{
                        "index": 0,
                        "message": {
                            "role": "assistant",
                            "content": img_markdown
                        },
                        "finish_reason": "stop"
                    }],
                    "usage": { "prompt_tokens": 0, "completion_tokens": 0, "total_tokens": 0 }
                });

                Ok((
                    StatusCode::OK,
                    [("X-Account-Email", email.as_str())],
                    Json(resp),
                )
                    .into_response())
            }
        }
        Err((status, msg, _email)) => Err((status, msg)),
    }
}
