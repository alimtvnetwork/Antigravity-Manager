use super::*;

/// Canonical consolidated response builders.
///
/// Pure code motion from `monitor.rs` — no logic changes.
pub(crate) fn build_canonical_consolidated_response(
    thinking_content: String,
    mut thinking_signature: String,
    response_content: String,
    tool_calls: Vec<Value>,
    session_id: Option<&str>,
    log_id: &str,
    timing_obj: serde_json::Map<String, Value>,
    usage_obj: Option<serde_json::Map<String, Value>>,
) -> Value {
    // 权威网关签名回填：若当前签名为空，尝试通过工具调用ID或会话ID从网关状态机(内存 L1 / SQLite L2 DB 思考持久化)恢复
    if thinking_signature.is_empty() {
        for tc in &tool_calls {
            if let Some(call_id) = tc.get("id").and_then(|v| v.as_str()) {
                if !call_id.is_empty() {
                    if let Some(sig) =
                        crate::proxy::SignatureCache::global().get_tool_signature(call_id)
                    {
                        thinking_signature = sig;
                        break;
                    }
                }
            }
        }
        if thinking_signature.is_empty() {
            // 1. 优先按当前轮次的思考文本片段精准直捞专属签名
            if !thinking_content.is_empty() {
                let trimmed = thinking_content.trim();
                let snippet = if trimmed.len() > 32 {
                    &trimmed[..32]
                } else {
                    trimmed
                };
                if let Some(sig) =
                    crate::modules::proxy_db::lookup_signature_by_thought_snippet(snippet)
                {
                    thinking_signature = sig;
                }
            }

            // 2. 兜底按会话状态机与会话数据库查找最新签名
            if thinking_signature.is_empty() {
                if let Some(sid) = session_id {
                    if let Some(sig) =
                        crate::proxy::SignatureCache::global().get_session_signature(sid)
                    {
                        thinking_signature = sig;
                    } else if let Some(sig) =
                        crate::modules::proxy_db::lookup_latest_thinking_signature(sid)
                    {
                        thinking_signature = sig;
                    }
                }
            }
        }
    }

    let mut consolidated = serde_json::Map::new();

    // 1. 会话/思考唯一标识
    let candidate_id = session_id.unwrap_or(log_id);
    consolidated.insert(
        "_session_thinking_id".to_string(),
        Value::String(candidate_id.to_string()),
    );

    // 2. 思考文本与权威签名
    if !thinking_content.is_empty() {
        consolidated.insert("thinking".to_string(), Value::String(thinking_content));
    }
    if !thinking_signature.is_empty() {
        consolidated.insert(
            "thinking_signature".to_string(),
            Value::String(thinking_signature),
        );
    }

    // 3. 正文输出
    if !response_content.is_empty() {
        consolidated.insert("content".to_string(), Value::String(response_content));
    }

    // 4. 工具调用列表 (过滤 Null)
    let clean_tool_calls: Vec<Value> = tool_calls.into_iter().filter(|v| !v.is_null()).collect();
    if !clean_tool_calls.is_empty() {
        consolidated.insert("tool_calls".to_string(), Value::Array(clean_tool_calls));
    }

    // 5. 耗时诊断
    if !timing_obj.is_empty() {
        consolidated.insert("_timing".to_string(), Value::Object(timing_obj));
    }

    // 6. 用量统计
    if let Some(usage) = usage_obj {
        if !usage.is_empty() {
            consolidated.insert("usage".to_string(), Value::Object(usage));
        }
    }

    Value::Object(consolidated)
}

pub(crate) fn consolidate_non_streaming_response(
    raw_json: &Value,
    log: &ProxyRequestLog,
    headers_map: &serde_json::Map<String, Value>,
) -> Option<Value> {
    // 若本就已是统一的 consolidated 格式 (例如带有顶层 thinking/tool_calls 且无 choices/candidates)，直接返回 None
    let has_consolidated_fields = raw_json.get("thinking").is_some()
        || raw_json.get("tool_calls").is_some()
        || raw_json.get("_session_thinking_id").is_some();
    if has_consolidated_fields {
        let has_no_raw_wrappers =
            raw_json.get("choices").is_none() && raw_json.get("candidates").is_none();
        if has_no_raw_wrappers {
            return None;
        }
    }

    let mut thinking_content = String::new();
    let mut thinking_signature = String::new();
    let mut response_content = String::new();
    let mut tool_calls = Vec::new();

    // 1. OpenAI 格式 (choices)
    if let Some(choices) = raw_json.get("choices").and_then(|c| c.as_array()) {
        for choice in choices {
            if let Some(msg) = choice.get("message") {
                if let Some(rc) = msg.get("reasoning_content").and_then(|v| v.as_str()) {
                    thinking_content.push_str(rc);
                }
                if let Some(th) = msg.get("thinking").and_then(|v| v.as_str()) {
                    thinking_content.push_str(th);
                }
                if let Some(sig) = msg
                    .get("thoughtSignature")
                    .or_else(|| msg.get("thought_signature"))
                    .or_else(|| msg.get("signature"))
                    .and_then(|v| v.as_str())
                {
                    thinking_signature = sig.to_string();
                }
                if let Some(c) = msg.get("content").and_then(|v| v.as_str()) {
                    response_content.push_str(c);
                }
                if let Some(tcs) = msg.get("tool_calls").and_then(|t| t.as_array()) {
                    for tc in tcs {
                        let id = tc.get("id").and_then(|v| v.as_str()).unwrap_or("");
                        let name = tc
                            .get("function")
                            .and_then(|f| f.get("name"))
                            .and_then(|n| n.as_str())
                            .unwrap_or("");
                        let args = tc
                            .get("function")
                            .and_then(|f| f.get("arguments"))
                            .map(|a| {
                                if let Some(s) = a.as_str() {
                                    s.to_string()
                                } else {
                                    a.to_string()
                                }
                            })
                            .unwrap_or_else(|| "{}".to_string());
                        tool_calls.push(serde_json::json!({
                            "id": id,
                            "type": "function",
                            "function": { "name": name, "arguments": args }
                        }));
                    }
                }
            }
        }
    }
    // 2. Gemini 格式 (candidates)
    else if let Some(candidates) = raw_json.get("candidates").and_then(|c| c.as_array()) {
        for cand in candidates {
            if let Some(parts) = cand
                .get("content")
                .and_then(|c| c.get("parts"))
                .and_then(|p| p.as_array())
            {
                for part in parts {
                    if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                        if part
                            .get("thought")
                            .and_then(|t| t.as_bool())
                            .unwrap_or(false)
                        {
                            thinking_content.push_str(text);
                        } else {
                            response_content.push_str(text);
                        }
                    }
                    if let Some(sig) = part
                        .get("thoughtSignature")
                        .or_else(|| part.get("thought_signature"))
                        .or_else(|| part.get("signature"))
                        .or_else(|| {
                            part.get("functionCall")
                                .and_then(|fc| fc.get("thoughtSignature"))
                        })
                        .or_else(|| {
                            part.get("functionCall")
                                .and_then(|fc| fc.get("thought_signature"))
                        })
                        .and_then(|s| s.as_str())
                    {
                        thinking_signature = sig.to_string();
                    }
                    if let Some(fc) = part.get("functionCall") {
                        if let Some(name) = fc.get("name").and_then(|n| n.as_str()) {
                            let call_id = if let Some(id_str) = fc
                                .get("id")
                                .and_then(|i| i.as_str())
                                .filter(|s| !s.is_empty())
                            {
                                id_str.to_string()
                            } else {
                                crate::proxy::thinking_store::synthesize_tool_id(
                                    name,
                                    fc.get("args"),
                                    "root",
                                    tool_calls.len(),
                                )
                            };
                            let args = fc
                                .get("args")
                                .map(|a| a.to_string())
                                .unwrap_or_else(|| "{}".to_string());
                            tool_calls.push(serde_json::json!({
                                "id": call_id,
                                "type": "function",
                                "function": { "name": name, "arguments": args }
                            }));
                        }
                    }
                }
            }
        }
    }
    // 3. Claude 格式 (content 数组)
    else if let Some(content) = raw_json.get("content").and_then(|c| c.as_array()) {
        for block in content {
            match block.get("type").and_then(|t| t.as_str()) {
                Some("thinking") => {
                    if let Some(th) = block.get("thinking").and_then(|t| t.as_str()) {
                        thinking_content.push_str(th);
                    }
                    if let Some(sig) = block
                        .get("signature")
                        .or_else(|| block.get("thought_signature"))
                        .or_else(|| block.get("thoughtSignature"))
                        .and_then(|s| s.as_str())
                    {
                        thinking_signature = sig.to_string();
                    }
                }
                Some("text") => {
                    if let Some(txt) = block.get("text").and_then(|t| t.as_str()) {
                        response_content.push_str(txt);
                    }
                }
                Some("tool_use") => {
                    let id = block.get("id").and_then(|v| v.as_str()).unwrap_or("");
                    let name = block.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    let args = block
                        .get("input")
                        .map(|a| a.to_string())
                        .unwrap_or_else(|| "{}".to_string());
                    tool_calls.push(serde_json::json!({
                        "id": id,
                        "type": "function",
                        "function": { "name": name, "arguments": args }
                    }));
                }
                _ => {}
            }
        }
    }

    if thinking_content.is_empty()
        && thinking_signature.is_empty()
        && response_content.is_empty()
        && tool_calls.is_empty()
    {
        return None;
    }

    // 提取耗时诊断
    let mut timing_obj = serde_json::Map::new();
    for key in [
        "clean_ms",
        "norm_ms",
        "thinking_ms",
        "ttft_ms",
        "stream_ms",
        "total_ms",
    ] {
        let hdr_key = format!("x-timing-{}", key);
        if let Some(val) = headers_map.get(&hdr_key).and_then(|v| v.as_str()) {
            if let Ok(n) = val.parse::<f64>() {
                let out_key = key.replace("_ms", "_s");
                timing_obj.insert(out_key, serde_json::json!(n / 1000.0));
            }
        }
    }
    if !timing_obj.contains_key("total_s") {
        timing_obj.insert(
            "total_s".to_string(),
            serde_json::json!(log.duration as f64 / 1000.0),
        );
    }

    // 提取 Token 用量与缓存命中率
    let usage_source = raw_json
        .get("usage")
        .or_else(|| raw_json.get("usageMetadata"));
    let mut usage_obj = serde_json::Map::new();
    let input_toks = log
        .input_tokens
        .or_else(|| usage_source.and_then(extract_input_tokens))
        .unwrap_or(0);
    let output_toks = log
        .output_tokens
        .or_else(|| usage_source.and_then(extract_output_tokens))
        .unwrap_or(0);
    let cached_toks = log
        .cached_tokens
        .or_else(|| usage_source.and_then(extract_cached_tokens))
        .unwrap_or(0);
    let reasoning_toks = usage_source.and_then(extract_reasoning_tokens).unwrap_or(0);

    let total_in = if cached_toks > input_toks {
        input_toks + cached_toks
    } else {
        input_toks
    };
    let total_toks = total_in + output_toks;

    usage_obj.insert("input_tokens".to_string(), serde_json::json!(total_in));
    usage_obj.insert("output_tokens".to_string(), serde_json::json!(output_toks));
    usage_obj.insert("total_tokens".to_string(), serde_json::json!(total_toks));
    if cached_toks > 0 {
        usage_obj.insert("cached_tokens".to_string(), serde_json::json!(cached_toks));
        if total_in > 0 {
            let hit_rate = (cached_toks as f64 / total_in as f64 * 100.0)
                .min(100.0)
                .max(0.0);
            usage_obj.insert(
                "cache_hit_rate".to_string(),
                serde_json::json!(format!("{:.1}%", hit_rate)),
            );
        }
    }
    if reasoning_toks > 0 {
        usage_obj.insert(
            "reasoning_tokens".to_string(),
            serde_json::json!(reasoning_toks),
        );
    }

    Some(build_canonical_consolidated_response(
        thinking_content,
        thinking_signature,
        response_content,
        tool_calls,
        log.session_id.as_deref(),
        &log.id,
        timing_obj,
        Some(usage_obj),
    ))
}
