// Codex SSE stream (split from streaming.rs).
// NOTE: create_codex_sse_stream (627 lines) uses yield inside async_stream! macro;
// it cannot be safely decomposed via pure code moves. Reported as exception.
// OpenAI 流式转换
use bytes::{Bytes, BytesMut};
use chrono::Utc;
use futures::{Stream, StreamExt};
use rand::Rng;
use serde_json::{json, Value};
use std::pin::Pin;
use uuid::Uuid;

/// 保存 thoughtSignature 到会话缓存

fn split_namespace_tool_name(qualified_name: &str) -> (String, Option<String>) {
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

fn inject_seq(mut event: Value, seq: &mut u64) -> Value {
    if let Some(obj) = event.as_object_mut() {
        obj.insert("sequence_number".to_string(), json!(*seq));
    }
    *seq += 1;
    event
}

/// Serialize one named Responses API SSE frame.
///
/// Codex Desktop consumes the SSE `event` field as well as the JSON payload's
/// `type`. Keeping both in sync matches the native Responses stream and avoids
/// reasoning/tool lifecycle events being treated as anonymous data messages.
fn codex_sse_frame(event: &Value) -> Bytes {
    let event_name = event
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("message");
    let payload = serde_json::to_string(event).unwrap_or_else(|_| "{}".to_string());
    Bytes::from(format!("event: {event_name}\ndata: {payload}\n\n"))
}

pub fn create_codex_sse_stream<S, E>(
    mut gemini_stream: Pin<Box<S>>,
    model: String,
    session_id: String,
    message_count: usize,
    _assistant_turn_index: usize,
    response_id: String,
    completion_tx: Option<
        tokio::sync::oneshot::Sender<(Vec<Value>, tokio::sync::oneshot::Sender<()>)>,
    >,
    cache_tool_calls: bool,
) -> Pin<Box<dyn Stream<Item = Result<Bytes, String>> + Send>>
where
    S: Stream<Item = Result<Bytes, E>> + Send + ?Sized + 'static,
    E: std::fmt::Display + Send + 'static,
{
    let mut buffer = BytesMut::new();
    let item_id_prefix = uuid::Uuid::new_v4().simple().to_string();
    let message_item_id = format!("msg_{}_0", &item_id_prefix[..16]);
    let created_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let mut completion_tx = completion_tx;
    let stream = async_stream::stream! {
            let mut sequence_number: u64 = 0;

            // Native Responses lifecycle: created must be followed by in_progress.
            let lifecycle_response = json!({
                "id": &response_id,
                "object": "response",
                "created_at": created_at,
                "status": "in_progress",
                "model": &model,
                "output": [],
                "error": null,
                "incomplete_details": null,
                "usage": null
            });
            let created_ev = json!({ "type": "response.created", "response": lifecycle_response.clone() });
            let created_ev = inject_seq(created_ev, &mut sequence_number);
            yield Ok::<Bytes, String>(codex_sse_frame(&created_ev));
            let in_progress_ev = json!({ "type": "response.in_progress", "response": lifecycle_response });
            let in_progress_ev = inject_seq(in_progress_ev, &mut sequence_number);
            yield Ok::<Bytes, String>(codex_sse_frame(&in_progress_ev));

            let mut message_item_emitted = false;
            let mut reasoning_open = false;
            let mut reasoning_item_seq: u32 = 0;
            let mut active_reasoning_item_id = String::new();

            let mut emitted_tool_calls = std::collections::HashSet::new();
            let mut accumulated_text = String::new();
            let mut accumulated_thinking = String::new();
            let mut thinking_acc = crate::proxy::thinking_store::TurnAccumulator::new();
            let mut has_seen_tool_calls = false;
            let mut final_finish_reason: Option<String> = None;

            let mut final_outputs_map: std::collections::BTreeMap<u32, serde_json::Value> = std::collections::BTreeMap::new();
            let mut next_output_index: u32 = 0;
            let mut message_output_index: u32 = 0;
            let mut reasoning_output_index: u32 = 0;
            let mut final_usage: Option<super::models::OpenAIUsage> = None;
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
                                        if line.is_empty() || !line.starts_with("data: ") { continue; }
                                        let json_part = line.trim_start_matches("data: ").trim();
                                        if json_part == "[DONE]" { continue; }

                                        if let Ok(mut json) = serde_json::from_str::<Value>(json_part) {
                                            let actual_data = if let Some(inner) = json.get_mut("response").map(|v| v.take()) { inner } else { json };

                                            if let Some(u) = actual_data.get("usageMetadata") {
                                                final_usage = extract_usage_metadata(u);
                                            }

                                            if let Some(candidates) = actual_data.get("candidates").and_then(|c| c.as_array()) {
                                                if candidates.len() > 0 {
                                                    tracing::debug!("[Codex-Stream-Debug] Raw Candidate: {:?}", candidates[0]);
                                                }
                                                if let Some(candidate) = candidates.get(0) {
                                                    if let Some(reason) = candidate.get("finishReason").and_then(Value::as_str) {
                                                        final_finish_reason = Some(reason.to_string());
                                                    }
                                                    if let Some(parts) = candidate.get("content").and_then(|c| c.get("parts")).and_then(|p| p.as_array()) {
                                                        for part in parts {
                                                            thinking_acc.ingest_part(part);
                                                            let is_thought = part.get("thought").and_then(|v| v.as_bool()).unwrap_or(false);

                                                            // Close the reasoning summary before opening normal text
                                                            // or a tool item so output item lifecycles never overlap.
                                                            let is_text_or_tool = part.get("text").is_some() || part.get("functionCall").is_some() || part.get("inlineData").is_some();
                                                            if is_text_or_tool && !is_thought && reasoning_open {
                                                                let text_done = json!({
                                                                    "type": "response.reasoning_summary_text.done",
                                                                    "item_id": &active_reasoning_item_id,
                                                                    "output_index": reasoning_output_index,
                                                                    "summary_index": 0,
                                                                    "text": &accumulated_thinking
                                                                });
                                                                let text_done = inject_seq(text_done, &mut sequence_number);
                                                                yield Ok::<Bytes, String>(codex_sse_frame(&text_done));

                                                                let summary_part_done = json!({
                                                                    "type": "response.reasoning_summary_part.done",
                                                                    "item_id": &active_reasoning_item_id,
                                                                    "output_index": reasoning_output_index,
                                                                    "summary_index": 0,
                                                                    "part": {
                                                                        "type": "summary_text",
                                                                        "text": &accumulated_thinking
                                                                    }
                                                                });
                                                                let summary_part_done = inject_seq(summary_part_done, &mut sequence_number);
                                                                yield Ok::<Bytes, String>(codex_sse_frame(&summary_part_done));

                                                                let reasoning_item = json!({
                                                                    "id": &active_reasoning_item_id,
                                                                    "type": "reasoning",
                                                                    "status": "completed",
                                                                    "summary": [{
                                                                        "type": "summary_text",
                                                                        "text": &accumulated_thinking
                                                                    }]
                                                                });

                                                                let done_ev = json!({
                                                                    "type": "response.output_item.done",
                                                                    "output_index": reasoning_output_index,
                                                                    "item": &reasoning_item
                                                                });
                                                                let done_ev = inject_seq(done_ev, &mut sequence_number);
                                                                yield Ok::<Bytes, String>(codex_sse_frame(&done_ev));

                                                                final_outputs_map.insert(reasoning_output_index, reasoning_item);
                                                                reasoning_open = false;
                                                            }

                                                            if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                                                                let clean_text = if is_thought {
                                                                    text.replace("<think>\n", "").replace("<think>", "").replace("\n</think>", "").replace("</think>", "")
                                                                } else {
                                                                    text.to_string()
                                                                };
                                                                if !clean_text.is_empty() {
                                                                    if is_thought && message_item_emitted {
                                                                        // Once ordinary assistant text has started, it is the
                                                                        // authoritative result for this response. A late thought
                                                                        // delta must not be appended to it or open an overlapping
                                                                        // reasoning item.
                                                                        tracing::warn!("[Codex-Stream] Dropping late thought delta after assistant text started");
                                                                    } else if is_thought {
                                                                        if !reasoning_open {
                                                                            reasoning_output_index = next_output_index;
                                                                            next_output_index += 1;
                                                                            active_reasoning_item_id = format!(
                                                                                "rs_{}_{}",
                                                                                &item_id_prefix[..16],
                                                                                reasoning_item_seq
                                                                            );
                                                                            reasoning_item_seq += 1;
                                                                            accumulated_thinking.clear();

                                                                            let output_item_added = json!({"type": "response.output_item.added", "output_index": reasoning_output_index, "item": {"id": &active_reasoning_item_id, "type": "reasoning", "status": "in_progress", "summary": []}});
                                                                            let output_item_added = inject_seq(output_item_added, &mut sequence_number);
                                                                            yield Ok::<Bytes, String>(codex_sse_frame(&output_item_added));

                                                                            let part_added = json!({"type": "response.reasoning_summary_part.added", "item_id": &active_reasoning_item_id, "output_index": reasoning_output_index, "summary_index": 0, "part": {"type": "summary_text", "text": ""}});
                                                                            let part_added = inject_seq(part_added, &mut sequence_number);
                                                                            yield Ok::<Bytes, String>(codex_sse_frame(&part_added));

                                                                            reasoning_open = true;
                                                                        }

                                                                        accumulated_thinking.push_str(&clean_text);
                                                                        let delta_ev = json!({
                                                                            "type": "response.reasoning_summary_text.delta",
                                                                            "item_id": &active_reasoning_item_id,
                                                                            "output_index": reasoning_output_index,
                                                                            "summary_index": 0,
                                                                            "delta": clean_text
                                                                        });
                                                                        let delta_ev = inject_seq(delta_ev, &mut sequence_number);
                                                                        yield Ok::<Bytes, String>(codex_sse_frame(&delta_ev));
                                                                    } else {
                                                                        if !message_item_emitted {
                                                                            message_item_emitted = true;
                                                                            message_output_index = next_output_index;
                                                                            next_output_index += 1;
                                                                            let output_item_added = json!({"type": "response.output_item.added", "output_index": message_output_index, "item": {"id": &message_item_id, "type": "message", "role": "assistant", "phase": "commentary", "status": "in_progress", "content": []}});
                                                                            let output_item_added = inject_seq(output_item_added, &mut sequence_number);
                                                                            yield Ok::<Bytes, String>(codex_sse_frame(&output_item_added));
                                                                            let content_part_added = json!({"type": "response.content_part.added", "item_id": &message_item_id, "output_index": message_output_index, "content_index": 0, "part": {"type": "output_text", "text": "", "annotations": []}});
                                                                            let content_part_added = inject_seq(content_part_added, &mut sequence_number);
                                                                            yield Ok::<Bytes, String>(codex_sse_frame(&content_part_added));
                                                                        }

                                                                        accumulated_text.push_str(&clean_text);
                                                                        let delta_ev = json!({
                                                                            "type": "response.output_text.delta",
                                                                            "item_id": &message_item_id,
                                                                            "output_index": message_output_index,
                                                                            "content_index": 0,
                                                                            "delta": clean_text
                                                                        });
                                                                        let delta_ev = inject_seq(delta_ev, &mut sequence_number);
                                                                        yield Ok::<Bytes, String>(codex_sse_frame(&delta_ev));
                                                                    }
                                                                }
                                                            }
                                                            if let Some(sig) = part.get("thoughtSignature").or(part.get("thought_signature")).and_then(|s| s.as_str()) {
                                                                store_thought_signature(sig, &session_id, message_count);
                                                            }
                                                            if let Some(func_call) = part.get("functionCall") {
                                                                let call_key = serde_json::to_string(func_call).unwrap_or_default();
                                                                if !emitted_tool_calls.contains(&call_key) {
                                                                    emitted_tool_calls.insert(call_key.clone());

                                                                    let name = func_call.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");
                                                                    let mut args = func_call.get("args").unwrap_or(&json!({})).clone();

                                                                    // [FIX #1575 & #3430] 标准化并清洗 shell / PowerShell 等工具参数名称与必填字段
                                                                    super::response::normalize_and_sanitize_tool_args(name, &mut args);

                                                                    let args_str = serde_json::to_string(&args).unwrap_or_default();

                                                                    let call_id = func_call
                                                                        .get("id")
                                                                        .and_then(|v| v.as_str())
                                                                        .map(|s| s.to_string())
                                                                        .unwrap_or_else(|| {
                                                                            let mut hasher = std::collections::hash_map::DefaultHasher::new();
                                                                            use std::hash::{Hash, Hasher};
    use crate::proxy::mappers::openai::streaming::openai::store_thought_signature;
                                                                            call_key.hash(&mut hasher);
                                                                            sequence_number.hash(&mut hasher);
                                                                            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos().hash(&mut hasher);
                                                                            format!("call_{:x}", hasher.finish())
                                                                        });

                                                                    if let Some(sig) = part.get("thoughtSignature").or(part.get("thought_signature")).and_then(|s| s.as_str()) {
                                                                        crate::proxy::SignatureCache::global().cache_tool_signature(&call_id, sig.to_string());
                                                                    }
                                                                    thinking_acc.record_tool_id(name, &call_id);

                                                                    let (actual_name, namespace) = split_namespace_tool_name(name);
                                                                    let tool_item_id = format!("item-{}", &Uuid::new_v4().to_string()[..16]);
                                                                    let is_custom_tool = actual_name == "apply_patch" || actual_name == "apply_patch_v2" || actual_name == "shell";

                                                                    let final_args_str = args_str.clone();

                                                                    let mut item_obj = json!({
                                                                        "id": &tool_item_id,
                                                                        "type": if is_custom_tool { "custom_tool_call" } else { "function_call" },
                                                                        "status": "completed",
                                                                        "name": actual_name,
                                                                        "call_id": &call_id,
                                                                    });
                                                                    if is_custom_tool {
                                                                        item_obj["input"] = json!(&final_args_str);
                                                                    } else {
                                                                        item_obj["arguments"] = json!(&final_args_str);
                                                                    }
                                                                    if let Some(ns) = namespace {
                                                                        item_obj["namespace"] = json!(ns);
                                                                    }

                                                                    let tool_output_index = next_output_index;
                                                                    next_output_index += 1;

                                                                    has_seen_tool_calls = true;

                                                                    let mut added_item = item_obj.clone();
                                                                    added_item["status"] = json!("in_progress");
                                                                    if is_custom_tool {
                                                                        added_item["input"] = json!("");
                                                                    } else {
                                                                        added_item["arguments"] = json!("");
                                                                    }
                                                                    let added_ev = json!({
                                                                        "type": "response.output_item.added",
                                                                        "output_index": tool_output_index,
                                                                        "item": added_item
                                                                    });
                                                                    let added_ev = inject_seq(added_ev, &mut sequence_number);
                                                                    yield Ok::<Bytes, String>(codex_sse_frame(&added_ev));

                                                                    let mut delta_ev = json!({
                                                                        "type": if is_custom_tool { "response.custom_tool_call_input.delta" } else { "response.function_call_arguments.delta" },
                                                                        "item_id": &tool_item_id,
                                                                        "output_index": tool_output_index,
                                                                        "delta": &final_args_str
                                                                    });
                                                                    if is_custom_tool {
                                                                        delta_ev["call_id"] = json!(&call_id);
                                                                    }
                                                                    let delta_ev = inject_seq(delta_ev, &mut sequence_number);
                                                                    yield Ok::<Bytes, String>(codex_sse_frame(&delta_ev));

                                                                    let mut args_done_ev = json!({
                                                                        "type": if is_custom_tool { "response.custom_tool_call_input.done" } else { "response.function_call_arguments.done" },
                                                                        "item_id": &tool_item_id,
                                                                        "output_index": tool_output_index,
                                                                    });
                                                                    if is_custom_tool {
                                                                        args_done_ev["call_id"] = json!(&call_id);
                                                                        args_done_ev["input"] = json!(&final_args_str);
                                                                    } else {
                                                                        args_done_ev["arguments"] = json!(&final_args_str);
                                                                    }
                                                                    let args_done_ev = inject_seq(args_done_ev, &mut sequence_number);
                                                                    yield Ok::<Bytes, String>(codex_sse_frame(&args_done_ev));

                                                                    let done_ev = json!({
                                                                        "type": "response.output_item.done",
                                                                        "output_index": tool_output_index,
                                                                        "item": item_obj
                                                                    });
                                                                    let done_ev = inject_seq(done_ev, &mut sequence_number);
                                                                    yield Ok::<Bytes, String>(codex_sse_frame(&done_ev));

                                                                    let tc_val = item_obj.clone();
                                                                    if cache_tool_calls {
                                                                        crate::proxy::handlers::openai::insert_cached_tool_call(call_id.clone(), tc_val.clone());
                                                                    }
                                                                    final_outputs_map.insert(tool_output_index, tc_val);
                                                                }
                                                            }
                                                        }
                                                    }

                                                    // 处理 groundingMetadata (搜索引文)
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
                                                        if !grounding_text.is_empty() {
                                                            if !message_item_emitted {
                                                                message_item_emitted = true;
                                                                message_output_index = next_output_index;
                                                                next_output_index += 1;
                                                                let output_item_added = json!({"type": "response.output_item.added", "output_index": message_output_index, "item": {"id": &message_item_id, "type": "message", "role": "assistant", "phase": "commentary", "status": "in_progress", "content": []}});
                                                                let output_item_added = inject_seq(output_item_added, &mut sequence_number);
                                                                yield Ok::<Bytes, String>(codex_sse_frame(&output_item_added));
                                                                let content_part_added = json!({"type": "response.content_part.added", "item_id": &message_item_id, "output_index": message_output_index, "content_index": 0, "part": {"type": "output_text", "text": "", "annotations": []}});
                                                                let content_part_added = inject_seq(content_part_added, &mut sequence_number);
                                                                yield Ok::<Bytes, String>(codex_sse_frame(&content_part_added));
                                                            }
                                                            accumulated_text.push_str(&grounding_text);
                                                            let delta_ev = json!({
                                                                "type": "response.output_text.delta",
                                                                "item_id": &message_item_id,
                                                                "output_index": message_output_index,
                                                                "content_index": 0,
                                                                "delta": grounding_text
                                                            });
                                                            let delta_ev = inject_seq(delta_ev, &mut sequence_number);
                                                            yield Ok::<Bytes, String>(codex_sse_frame(&delta_ev));
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Some(Err(_)) => break,
                            None => break,
                        }
                    }
                    _ = heartbeat_interval.tick() => {
                        yield Ok::<Bytes, String>(Bytes::from(": ping\n\n"));
                    }
                }
            }

            // Finalize any reasoning summary still open when the upstream stream ends.
            if reasoning_open {
                let text_done = json!({
                    "type": "response.reasoning_summary_text.done",
                    "item_id": &active_reasoning_item_id,
                    "output_index": reasoning_output_index,
                    "summary_index": 0,
                    "text": &accumulated_thinking
                });
                let text_done = inject_seq(text_done, &mut sequence_number);
                yield Ok::<Bytes, String>(codex_sse_frame(&text_done));

                let summary_part_done = json!({
                    "type": "response.reasoning_summary_part.done",
                    "item_id": &active_reasoning_item_id,
                    "output_index": reasoning_output_index,
                    "summary_index": 0,
                    "part": {
                        "type": "summary_text",
                        "text": &accumulated_thinking
                    }
                });
                let summary_part_done = inject_seq(summary_part_done, &mut sequence_number);
                yield Ok::<Bytes, String>(codex_sse_frame(&summary_part_done));

                let reasoning_item = json!({
                    "id": &active_reasoning_item_id,
                    "type": "reasoning",
                    "status": "completed",
                    "summary": [{
                        "type": "summary_text",
                        "text": &accumulated_thinking
                    }]
                });

                let done_ev = json!({
                    "type": "response.output_item.done",
                    "output_index": reasoning_output_index,
                    "item": &reasoning_item
                });
                let done_ev = inject_seq(done_ev, &mut sequence_number);
                yield Ok::<Bytes, String>(codex_sse_frame(&done_ev));

                final_outputs_map.insert(reasoning_output_index, reasoning_item);
            }

            // A proxy-generated diagnostic (for example an invalid apply_patch) may
            // only become available after the upstream stream has ended. Open the
            // message lazily here so it is not silently dropped behind reasoning.
            if !message_item_emitted && !accumulated_text.is_empty() {
                message_item_emitted = true;
                message_output_index = next_output_index;
                let output_item_added = json!({"type": "response.output_item.added", "output_index": message_output_index, "item": {"id": &message_item_id, "type": "message", "role": "assistant", "phase": "commentary", "status": "in_progress", "content": []}});
                let output_item_added = inject_seq(output_item_added, &mut sequence_number);
                yield Ok::<Bytes, String>(codex_sse_frame(&output_item_added));
                let content_part_added = json!({"type": "response.content_part.added", "item_id": &message_item_id, "output_index": message_output_index, "content_index": 0, "part": {"type": "output_text", "text": "", "annotations": []}});
                let content_part_added = inject_seq(content_part_added, &mut sequence_number);
                yield Ok::<Bytes, String>(codex_sse_frame(&content_part_added));
            }

            if message_item_emitted {
                let text_done = json!({
                    "type": "response.output_text.done",
                    "item_id": &message_item_id,
                    "output_index": message_output_index,
                    "content_index": 0,
                    "text": &accumulated_text
                });
                let text_done = inject_seq(text_done, &mut sequence_number);
                yield Ok::<Bytes, String>(codex_sse_frame(&text_done));

                let content_part_done = json!({
                    "type": "response.content_part.done",
                    "item_id": &message_item_id,
                    "output_index": message_output_index,
                    "content_index": 0,
                    "part": {
                        "type": "output_text",
                        "text": &accumulated_text,
                        "annotations": []
                    }
                });
                let content_part_done = inject_seq(content_part_done, &mut sequence_number);
                yield Ok::<Bytes, String>(codex_sse_frame(&content_part_done));

                // Tool rounds are process commentary. Only a response with no tool
                // call is the authoritative final answer that remains expanded.
                let message_phase = if has_seen_tool_calls { "commentary" } else { "final_answer" };
                let message_item = json!({
                    "id": &message_item_id,
                    "type": "message",
                    "role": "assistant",
                    "phase": message_phase,
                    "status": "completed",
                    "content": [{
                        "type": "output_text",
                        "text": &accumulated_text,
                        "annotations": []
                    }]
                });

                let output_item_done = json!({
                    "type": "response.output_item.done",
                    "output_index": message_output_index,
                    "item": message_item.clone()
                });
                let output_item_done = inject_seq(output_item_done, &mut sequence_number);
                yield Ok::<Bytes, String>(codex_sse_frame(&output_item_done));

                final_outputs_map.insert(message_output_index, message_item);
            }

            let final_outputs: Vec<serde_json::Value> = final_outputs_map.into_values().collect();

            let missing_actionable_output = !message_item_emitted && !has_seen_tool_calls;
            let terminal_status = if missing_actionable_output {
                "incomplete"
            } else {
                match final_finish_reason.as_deref() {
                    Some("MAX_TOKENS")
                    | Some("SAFETY")
                    | Some("RECITATION")
                    | Some("BLOCKLIST")
                    | Some("PROHIBITED_CONTENT")
                    | Some("SPII")
                    | Some("IMAGE_SAFETY")
                    | Some("IMAGE_PROHIBITED_CONTENT")
                    | None => "incomplete",
                    _ => "completed",
                }
            };
            let terminal_type = format!("response.{terminal_status}");
            let incomplete_details = if terminal_status == "incomplete" {
                let reason = match final_finish_reason.as_deref() {
                    Some("MAX_TOKENS") => "max_output_tokens",
                    Some("SAFETY")
                    | Some("RECITATION")
                    | Some("BLOCKLIST")
                    | Some("PROHIBITED_CONTENT")
                    | Some("SPII")
                    | Some("IMAGE_SAFETY")
                    | Some("IMAGE_PROHIBITED_CONTENT") => "content_filter",
                    _ => "interrupted",
                };
                json!({"reason": reason})
            } else {
                Value::Null
            };
            let terminal_error = if missing_actionable_output {
                json!({
                    "code": "empty_response",
                    "message": "Gemini stream ended without a final assistant message or tool call."
                })
            } else if final_finish_reason.is_none() {
                json!({
                    "code": "upstream_interrupted",
                    "message": "Gemini stream ended without finishReason."
                })
            } else {
                Value::Null
            };
            let completed_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            if terminal_status == "completed" {
                if let Some(tx) = completion_tx.take() {
                    let (ack_tx, ack_rx) = tokio::sync::oneshot::channel();
                    if tx.send((final_outputs.clone(), ack_tx)).is_err() || ack_rx.await.is_err() {
                        return;
                    }
                }
            }

            thinking_acc.clone().commit(&session_id);
            if session_id != response_id {
                thinking_acc.commit(&response_id);
            }

            let mut completed_ev = json!({
                "type": terminal_type,
                "response": {
                    "id": &response_id,
                    "object": "response",
                    "created_at": created_at,
                    "completed_at": completed_at,
                    "status": terminal_status,
                    "model": &model,
                    "output": final_outputs,
                    "incomplete_details": incomplete_details,
                    "error": terminal_error
                }
            });

            if let Some(resp_obj) = completed_ev.get_mut("response").and_then(|r| r.as_object_mut()) {
                if let Some(ref usage) = final_usage {
                    resp_obj.insert("usage".to_string(), usage.to_responses_usage_value());
                } else {
                    resp_obj.insert(
                        "usage".to_string(),
                        json!({
                            "input_tokens": 0,
                            "input_tokens_details": {
                                "cached_tokens": 0
                            },
                            "output_tokens": 0,
                            "output_tokens_details": {
                                "reasoning_tokens": 0
                            },
                            "total_tokens": 0
                        }),
                    );
                }
            }

            let completed_ev = inject_seq(completed_ev, &mut sequence_number);
            yield Ok::<Bytes, String>(codex_sse_frame(&completed_ev));
        };
    Box::pin(stream)
}
