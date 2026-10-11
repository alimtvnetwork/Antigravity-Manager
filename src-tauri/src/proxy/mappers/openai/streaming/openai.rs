// OpenAI SSE streams (split from streaming.rs).
// OpenAI 流式转换
use bytes::{Bytes, BytesMut};
use chrono::Utc;
use futures::{Stream, StreamExt};
use rand::Rng;
use serde_json::{json, Value};
use std::pin::Pin;
use uuid::Uuid;

/// 保存 thoughtSignature 到会话缓存

pub fn store_thought_signature(sig: &str, session_id: &str, message_count: usize) {
    if sig.is_empty() {
        return;
    }

    // 2. [CRITICAL] 存储到 Session 隔离缓存 (对齐 Claude 协议)
    crate::proxy::SignatureCache::global().cache_session_signature(
        session_id,
        sig.to_string(),
        message_count,
    );

    tracing::debug!(
        "[ThoughtSig] 存储 Session 签名 (sid: {}, len: {}, msg_count: {})",
        session_id,
        sig.len(),
        message_count
    );
}

/// Extract and convert Gemini usageMetadata to OpenAI usage format
/// Supports both legacy v1internal format and new Interactions API format.
///
/// Key semantic difference:
/// - Old format: candidatesTokenCount = all output tokens (text + thinking + tool)
/// - New format: total_output_tokens = text + tool output only; thought tokens are separate (total_thought_tokens)
/// For Codex, we must sum them back together as `completion_tokens`.
pub(crate) fn extract_usage_metadata(u: &Value) -> Option<super::super::models::OpenAIUsage> {
    let canonical = crate::proxy::pipeline::CanonicalUsage::from_gemini(u);
    let mut usage = super::super::models::OpenAIUsage::from(&canonical);
    usage.input_tokens_by_modality = u.get("input_tokens_by_modality").cloned();
    usage.total_tool_use_tokens = u
        .get("total_tool_use_tokens")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);
    Some(usage)
}

pub fn create_openai_sse_stream<S, E>(
    gemini_stream: Pin<Box<S>>,
    model: String,
    session_id: String,
    message_count: usize,
    client_tool_names: Option<std::collections::HashSet<String>>,
    include_usage: bool,
) -> Pin<Box<dyn Stream<Item = Result<Bytes, String>> + Send>>
where
    S: Stream<Item = Result<Bytes, E>> + Send + ?Sized + 'static,
    E: std::fmt::Display + Send + 'static,
{
    create_openai_sse_stream_with_anchor(
        gemini_stream,
        model,
        session_id,
        message_count,
        client_tool_names,
        include_usage,
        None,
    )
}

pub fn create_openai_sse_stream_with_anchor<S, E>(
    mut gemini_stream: Pin<Box<S>>,
    model: String,
    session_id: String,
    message_count: usize,
    client_tool_names: Option<std::collections::HashSet<String>>,
    include_usage: bool,
    causal_anchor: Option<String>,
) -> Pin<Box<dyn Stream<Item = Result<Bytes, String>> + Send>>
where
    S: Stream<Item = Result<Bytes, E>> + Send + ?Sized + 'static,
    E: std::fmt::Display + Send + 'static,
{
    let mut buffer = BytesMut::new();
    let stream_id = format!("chatcmpl-{}", Uuid::new_v4());
    let created_ts = Utc::now().timestamp();

    let empty_set = std::collections::HashSet::new();
    let client_tool_names = client_tool_names.unwrap_or(empty_set);

    let stream = async_stream::stream! {
        let mut emitted_tool_calls = std::collections::HashSet::new();
        let mut final_usage: Option<super::super::models::OpenAIUsage> = None;
        let mut error_occurred = false;
        let mut tool_call_index = 0;
        let mut thinking_acc = if let Some(ref a) = causal_anchor {
            crate::proxy::thinking_store::TurnAccumulator::with_anchor(a)
        } else {
            crate::proxy::thinking_store::TurnAccumulator::new()
        };

        let mut heartbeat_interval = tokio::time::interval(std::time::Duration::from_secs(15));
        heartbeat_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                item = gemini_stream.next() => {
                    match item {
                        Some(Ok(bytes)) => {
                            buffer.extend_from_slice(&bytes);
                            while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
                                let line_raw = buffer.split_to(pos + 1);
                                if let Ok(line_str) = std::str::from_utf8(&line_raw) {
                                    let line = line_str.trim();
                                    if line.is_empty() { continue; }
                                    if line.starts_with("data: ") {
                                        let json_part = line.trim_start_matches("data: ").trim();
                                        if json_part == "[DONE]" { continue; }
                                        if let Ok(mut json) = serde_json::from_str::<Value>(json_part) {
                                            let actual_data = if let Some(inner) = json.get_mut("response").map(|v| v.take()) { inner } else { json };
                                            if let Some(u) = actual_data.get("usageMetadata") {
                                                final_usage = extract_usage_metadata(u);
                                            }

                                            if let Some(candidates) = actual_data.get("candidates").and_then(|c| c.as_array()) {
                                                // [DEBUG] 打印原始 candidate 以排查空回复问题
                                                if candidates.len() > 0 {
                                                     tracing::debug!("[Stream-Debug] Raw Candidate: {:?}", candidates[0]);
                                                }
                                                for (idx, candidate) in candidates.iter().enumerate() {
                                                    let parts = candidate.get("content").and_then(|c| c.get("parts")).and_then(|p| p.as_array());
                                                    let mut content_out = String::new();
                                                    let mut thought_out = String::new();

                                                    if let Some(parts_list) = parts {
                                                        for part in parts_list {
                                                            thinking_acc.ingest_part(part);
                                                            let is_thought_part = part.get("thought").and_then(|v| v.as_bool()).unwrap_or(false);
                                                            if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                                                                if is_thought_part {
                                                                    // thought 内容只写入 thought_out（给支持 reasoning_content 的客户端），防止客户端重复显示思维过程
                                                                    let clean_text = text.replace("<think>\n", "").replace("<think>", "").replace("\n</think>", "").replace("</think>", "");
                                                                    thought_out.push_str(&clean_text);
                                                                } else {
                                                                    // 真实正文内容（非思考块）保留原始文本，避免技术讨论或代码反引号中的 `<think>` 标签被粗暴抹除为空
                                                                    content_out.push_str(text);
                                                                }
                                                            }
                                                            if let Some(sig) = part.get("thoughtSignature").or(part.get("thought_signature")).and_then(|s| s.as_str()) {
                                                                store_thought_signature(sig, &session_id, message_count);
                                                            }
                                                            if let Some(img) = part.get("inlineData") {
                                                                let mime_type = img.get("mimeType").and_then(|v| v.as_str()).unwrap_or("image/png");
                                                                let data = img.get("data").and_then(|v| v.as_str()).unwrap_or("");
                                                                if !data.is_empty() {
                                                                    content_out.push_str(&format!("![image](data:{};base64,{})", mime_type, data));
                                                                }
                                                            }
                                                            if let Some(func_call) = part.get("functionCall") {
                                                                let call_key = serde_json::to_string(func_call).unwrap_or_default();
                                                                if !emitted_tool_calls.contains(&call_key) {
                                                                    emitted_tool_calls.insert(call_key);
                                                                    let name = func_call.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");
                                                                    let mut args = func_call.get("args").unwrap_or(&json!({})).clone();

                                                                    // [FIX #1575 & #3430] 标准化并清洗 shell / PowerShell 等工具参数名称与必填字段
                                                                    super::super::response::normalize_and_sanitize_tool_args(name, &mut args);

                                                                    let final_name = super::super::response::resolve_shell_tool_name(name, &client_tool_names);

                                                                    let call_id = func_call
                                                                        .get("id")
                                                                        .and_then(|v| v.as_str())
                                                                        .map(|s| s.to_string())
                                                                        .unwrap_or_else(|| {
                                                                            let mut hasher = std::collections::hash_map::DefaultHasher::new();
                                                                            use std::hash::{Hash, Hasher};
                                                                            serde_json::to_string(func_call).unwrap_or_default().hash(&mut hasher);
                                                                            tool_call_index.hash(&mut hasher);
                                                                            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos().hash(&mut hasher);
                                                                            format!("call_{:x}", hasher.finish())
                                                                        });

                                                                    if let Some(sig) = part.get("thoughtSignature").or(part.get("thought_signature")).and_then(|s| s.as_str()) {
                                                                        crate::proxy::SignatureCache::global().cache_tool_signature(&call_id, sig.to_string());
                                                                    }
                                                                    thinking_acc.record_tool_id(name, &call_id);

                                                                    let args_str = serde_json::to_string(&args).unwrap_or_default();
                                                                    let tool_call_chunk = json!({
                                                                        "id": &stream_id,
                                                                        "object": "chat.completion.chunk",
                                                                        "created": created_ts,
                                                                        "model": &model,
                                                                        "choices": [{
                                                                            "index": idx as u32,
                                                                            "delta": {
                                                                                "role": "assistant",
                                                                                "tool_calls": [{
                                                                                    "index": tool_call_index,
                                                                                    "id": call_id,
                                                                                    "type": "function",
                                                                                    "function": { "name": final_name, "arguments": args_str }
                                                                                }]
                                                                            },
                                                                            "finish_reason": serde_json::Value::Null
                                                                        }]
                                                                    });

                                                                    tool_call_index += 1;
                                                                    let sse_out = format!("data: {}\n\n", serde_json::to_string(&tool_call_chunk).unwrap_or_default());
                                                                    yield Ok::<Bytes, String>(Bytes::from(sse_out));
                                                                }
                                                            }
                                                        }
                                                    }

                                                    if let Some(grounding) = candidate.get("groundingMetadata") {
                                                        let mut grounding_text = String::new();
                                                        if let Some(queries) = grounding.get("webSearchQueries").and_then(|q| q.as_array()) {
                                                            let query_list: Vec<&str> = queries.iter().filter_map(|v| v.as_str()).collect();
                                                            if !query_list.is_empty() {
                                                                grounding_text.push_str("\n\n---\n**🔍 已为您搜索：** ");
                                                                grounding_text.push_str(&query_list.join(", "));
                                                            }
                                                        }
                                                        if let Some(chunks) = grounding.get("groundingChunks").and_then(|c| c.as_array()) {
                                                            let mut links = Vec::new();
                                                            for (i, chunk) in chunks.iter().enumerate() {
                                                                if let Some(web) = chunk.get("web") {
                                                                    let title = web.get("title").and_then(|v| v.as_str()).unwrap_or("网页来源");
                                                                    let uri = web.get("uri").and_then(|v| v.as_str()).unwrap_or("#");
                                                                    links.push(format!("[{}] [{}]({})", i + 1, title, uri));
                                                                }
                                                            }
                                                            if !links.is_empty() {
                                                                grounding_text.push_str("\n\n**🌐 来源引文：**\n");
                                                                grounding_text.push_str(&links.join("\n"));
                                                            }
                                                        }
                                                        if !grounding_text.is_empty() { content_out.push_str(&grounding_text); }
                                                    }

                                                    let raw_finish_reason = candidate.get("finishReason").and_then(|f| f.as_str());

                                                    let gemini_finish_reason = raw_finish_reason.map(|f| match f {
                                                        "STOP" => "stop",
                                                        "MAX_TOKENS" => "length",
                                                        "SAFETY" => "content_filter",
                                                        "RECITATION" => "content_filter",
                                                        "MALFORMED_FUNCTION_CALL" => "stop",
                                                        _ => "stop",
                                                    });

                                                    // [FIX #1575] 如果发射了工具调用，强制设置为 tool_calls
                                                    // 解决 Gemini 返回 STOP 但有工具调用时，OpenAI 客户端认为对话已结束的问题
                                                    let finish_reason = if !emitted_tool_calls.is_empty() && gemini_finish_reason.is_some() {
                                                        Some("tool_calls")
                                                    } else {
                                                        gemini_finish_reason
                                                    };

                                                    if !thought_out.is_empty() {
                                                        let reasoning_chunk = json!({
                                                            "id": &stream_id,
                                                            "object": "chat.completion.chunk",
                                                            "created": created_ts,
                                                            "model": &model,
                                                            "choices": [{
                                                                "index": idx as u32,
                                                                "delta": { "role": "assistant", "content": serde_json::Value::Null, "reasoning_content": thought_out },
                                                                "finish_reason": serde_json::Value::Null
                                                            }]
                                                        });
                                                        let sse_out = format!("data: {}\n\n", serde_json::to_string(&reasoning_chunk).unwrap_or_default());
                                                        yield Ok::<Bytes, String>(Bytes::from(sse_out));
                                                    }

                                                    if !content_out.is_empty() || finish_reason.is_some() {
                                                        let delta = if !content_out.is_empty() {
                                                            json!({ "content": content_out })
                                                        } else {
                                                            json!({})
                                                        };
                                                        let mut openai_chunk = json!({
                                                            "id": &stream_id,
                                                            "object": "chat.completion.chunk",
                                                            "created": created_ts,
                                                            "model": &model,
                                                            "choices": [{
                                                                "index": idx as u32,
                                                                "delta": delta,
                                                                "finish_reason": finish_reason
                                                            }]
                                                        });
                                                        if finish_reason.is_some() {
                                                            if !include_usage {
                                                                if let Some(ref usage) = final_usage {
                                                                    openai_chunk["usage"] = serde_json::to_value(usage).unwrap();
                                                                }
                                                                final_usage = None;
                                                            }
                                                        }
                                                        let sse_out = format!("data: {}\n\n", serde_json::to_string(&openai_chunk).unwrap_or_default());
                                                        yield Ok::<Bytes, String>(Bytes::from(sse_out));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Some(Err(e)) => {
                            use crate::proxy::mappers::error_classifier::classify_stream_error;
                            let (error_type, user_msg, i18n_key) = classify_stream_error(&e);
                            tracing::error!("OpenAI Stream Error: {}", e);
                            let error_chunk = json!({
                                "id": &stream_id, "object": "chat.completion.chunk", "created": created_ts, "model": &model, "choices": [],
                                "error": { "type": error_type, "message": user_msg, "code": "stream_error", "i18n_key": i18n_key }
                            });
                            yield Ok(Bytes::from(format!("data: {}\n\n", serde_json::to_string(&error_chunk).unwrap_or_default())));
                            yield Ok(Bytes::from("data: [DONE]\n\n"));
                            error_occurred = true;
                            break;
                        }
                        None => break,
                    }
                }
                _ = heartbeat_interval.tick() => {
                    yield Ok::<Bytes, String>(Bytes::from(": ping\n\n"));
                }
            }
        }

        // [FIX #1732] Flush remaining buffer to prevent hang on network fragmentation
        if !buffer.is_empty() {
            if let Ok(line_str) = std::str::from_utf8(&buffer) {
                let line = line_str.trim();
                if !line.is_empty() && line.starts_with("data: ") {
                    let json_part = line.trim_start_matches("data: ").trim();
                    if json_part != "[DONE]" {
                        // Re-use logic for processing the last line
                        // (Note: In a more complex refactor we'd extract this to a function,
                        // but for a targeted fix, processing the terminal data chunk is safer)
                        tracing::debug!("[OpenAI-SSE] Flushing remaining {} bytes in buffer", buffer.len());
                    }
                }
            }
        }

        thinking_acc.commit(&session_id);
        if !error_occurred {
            // [CRITICAL FIX #3455] Only emit standalone usage chunk with empty choices if client explicitly
            // requested stream_options.include_usage: true. Emitting choices: [] unconditionally causes Python
            // OpenAI SDK and autonomous agents (Hermes, etc.) to crash with `IndexError: list index out of range`!
            if include_usage {
                if let Some(usage) = final_usage.take() {
                    let usage_chunk = json!({
                        "id": &stream_id,
                        "object": "chat.completion.chunk",
                        "created": created_ts,
                        "model": &model,
                        "choices": [],
                        "usage": usage
                    });
                    yield Ok::<Bytes, String>(Bytes::from(format!("data: {}\n\n", serde_json::to_string(&usage_chunk).unwrap_or_default())));
                }
            }
            yield Ok::<Bytes, String>(Bytes::from("data: [DONE]\n\n"));
        }
    };
    Box::pin(stream)
}

pub fn create_legacy_sse_stream<S, E>(
    mut gemini_stream: Pin<Box<S>>,
    model: String,
    session_id: String,
    message_count: usize,
) -> Pin<Box<dyn Stream<Item = Result<Bytes, String>> + Send>>
where
    S: Stream<Item = Result<Bytes, E>> + Send + ?Sized + 'static,
    E: std::fmt::Display + Send + 'static,
{
    let mut buffer = BytesMut::new();
    let charset = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();
    let random_str: String = (0..28)
        .map(|_| {
            let idx = rng.gen_range(0..charset.len());
            charset.chars().nth(idx).unwrap()
        })
        .collect();
    let stream_id = format!("cmpl-{}", random_str);
    let created_ts = Utc::now().timestamp();

    let stream = async_stream::stream! {
        let mut final_usage: Option<super::super::models::OpenAIUsage> = None;
        let mut error_occurred = false;
        let mut thinking_acc = crate::proxy::thinking_store::TurnAccumulator::new();
        let mut heartbeat_interval = tokio::time::interval(std::time::Duration::from_secs(15));
        heartbeat_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                item = gemini_stream.next() => {
                    match item {
                        Some(Ok(bytes)) => {
                            buffer.extend_from_slice(&bytes);
                            while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
                                let line_raw = buffer.split_to(pos + 1);
                                if let Ok(line_str) = std::str::from_utf8(&line_raw) {
                                    let line = line_str.trim();
                                    if line.is_empty() { continue; }
                                    if line.starts_with("data: ") {
                                        let json_part = line.trim_start_matches("data: ").trim();
                                        if json_part == "[DONE]" { continue; }
                                        if let Ok(mut json) = serde_json::from_str::<Value>(json_part) {
                                            let actual_data = if let Some(inner) = json.get_mut("response").map(|v| v.take()) { inner } else { json };
                                            if let Some(u) = actual_data.get("usageMetadata") { final_usage = extract_usage_metadata(u); }

                                            let mut content_out = String::new();
                                            if let Some(candidates) = actual_data.get("candidates").and_then(|c| c.as_array()) {
                                                if let Some(candidate) = candidates.get(0) {
                                                    if let Some(parts) = candidate.get("content").and_then(|c| c.get("parts")).and_then(|p| p.as_array()) {
                                                        for part in parts {
                                                            thinking_acc.ingest_part(part);
                                                            let is_thought = part.get("thought").and_then(|v| v.as_bool()).unwrap_or(false);
                                                            if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                                                                if is_thought {
                                                                    let clean_text = text.replace("<think>\n", "").replace("<think>", "").replace("\n</think>", "").replace("</think>", "");
                                                                    content_out.push_str(&clean_text);
                                                                } else {
                                                                    content_out.push_str(text);
                                                                }
                                                            }
                                                            if let Some(sig) = part.get("thoughtSignature").or(part.get("thought_signature")).and_then(|s| s.as_str()) {
                                                                store_thought_signature(sig, &session_id, message_count);
                                                            }
                                                        }
                                                    }
                                                }
                                            }

                                            let finish_reason = actual_data.get("candidates").and_then(|c| c.as_array()).and_then(|c| c.get(0)).and_then(|c| c.get("finishReason")).and_then(|f| f.as_str()).map(|f| match f {
                                                "STOP" => "stop", "MAX_TOKENS" => "length", "SAFETY" => "content_filter", "RECITATION" => "content_filter", _ => "stop",
                                            });

                                            let mut legacy_chunk = json!({
                                                "id": &stream_id, "object": "text_completion", "created": created_ts, "model": &model,
                                                "choices": [{ "text": content_out, "index": 0, "logprobs": null, "finish_reason": finish_reason }]
                                            });
                                            if let Some(ref usage) = final_usage { legacy_chunk["usage"] = serde_json::to_value(usage).unwrap(); }
                                            if finish_reason.is_some() { final_usage = None; }
                                            yield Ok::<Bytes, String>(Bytes::from(format!("data: {}\n\n", serde_json::to_string(&legacy_chunk).unwrap_or_default())));
                                        }
                                    }
                                }
                            }
                        }
                        Some(Err(e)) => {
                            use crate::proxy::mappers::error_classifier::classify_stream_error;
                            let (error_type, user_msg, i18n_key) = classify_stream_error(&e);
                            tracing::error!("Legacy Stream Error: {}", e);
                            let error_chunk = json!({
                                "id": &stream_id, "object": "text_completion", "created": created_ts, "model": &model, "choices": [],
                                "error": { "type": error_type, "message": user_msg, "code": "stream_error", "i18n_key": i18n_key }
                            });
                            yield Ok::<Bytes, String>(Bytes::from(format!("data: {}\n\n", serde_json::to_string(&error_chunk).unwrap_or_default())));
                            yield Ok::<Bytes, String>(Bytes::from("data: [DONE]\n\n"));
                            error_occurred = true;
                            break;
                        }
                        None => break,
                    }
                }
                _ = heartbeat_interval.tick() => { yield Ok::<Bytes, String>(Bytes::from(": ping\n\n")); }
            }
        }
        thinking_acc.commit(&session_id);
        if !error_occurred {
            yield Ok::<Bytes, String>(Bytes::from("data: [DONE]\n\n"));
        }
    };
    Box::pin(stream)
}
