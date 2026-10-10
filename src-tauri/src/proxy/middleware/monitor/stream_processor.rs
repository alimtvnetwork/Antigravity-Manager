use super::*;
use std::sync::Arc;
use std::time::Instant;

/// Process collected SSE stream data: parse events, build consolidated response,
/// extract tokens, and persist the log.
///
/// Extracted from `monitor_middleware` via extract-method (Phase 3: stream processing).
/// Pure code motion — no logic changes.
pub(crate) async fn process_collected_stream(
    all_stream_data: Vec<u8>,
    last_few_bytes: Vec<u8>,
    stream_ms: f64,
    total_ms: f64,
    mut log: ProxyRequestLog,
    user_token_identity: Option<UserTokenIdentity>,
    monitor: Arc<ProxyMonitor>,
    user_agent: Option<String>,
) {
    let mut headers_map: serde_json::Map<String, Value> = log
        .response_headers
        .as_ref()
        .and_then(|h| serde_json::from_str(h).ok())
        .unwrap_or_default();

    headers_map.insert(
        "x-timing-stream-ms".to_string(),
        serde_json::json!(format!("{:.3}", stream_ms)),
    );
    headers_map.insert(
        "x-timing-total-ms".to_string(),
        serde_json::json!(format!("{:.3}", total_ms)),
    );
    log.response_headers = serde_json::to_string(&Value::Object(headers_map.clone())).ok();

    // Parse and consolidate stream data into readable format
    if let Ok(full_response) = std::str::from_utf8(&all_stream_data) {
        let mut acc = SseParseAccumulator::new();

        for line in full_response.lines() {
            if !line.starts_with("data: ") {
                continue;
            }
            let json_str = line.trim_start_matches("data: ").trim();
            if json_str == "[DONE]" {
                continue;
            }

            if let Ok(json) = serde_json::from_str::<Value>(json_str) {
                acc.apply_openai_delta(&json);
                acc.apply_gemini_candidate(&json);
                acc.apply_claude_event(&json, &mut log);
                acc.apply_token_usage(&json, &mut log);
            }
        }

        // [Timing Diagnostics] 注入耗时诊断元数据 (秒)
        let mut timing_obj = serde_json::Map::new();
        if let Some(clean) = headers_map
            .get("x-timing-clean-ms")
            .and_then(|v| v.as_str())
        {
            if let Ok(n) = clean.parse::<f64>() {
                timing_obj.insert("clean_s".to_string(), serde_json::json!(n / 1000.0));
            }
        }
        if let Some(norm) = headers_map.get("x-timing-norm-ms").and_then(|v| v.as_str()) {
            if let Ok(n) = norm.parse::<f64>() {
                timing_obj.insert("norm_s".to_string(), serde_json::json!(n / 1000.0));
            }
        }
        if let Some(th) = headers_map
            .get("x-timing-thinking-ms")
            .and_then(|v| v.as_str())
        {
            if let Ok(n) = th.parse::<f64>() {
                timing_obj.insert("thinking_s".to_string(), serde_json::json!(n / 1000.0));
            }
        }
        if let Some(ttft) = headers_map.get("x-timing-ttft-ms").and_then(|v| v.as_str()) {
            if let Ok(n) = ttft.parse::<f64>() {
                timing_obj.insert("ttft_s".to_string(), serde_json::json!(n / 1000.0));
            }
        }
        timing_obj.insert(
            "stream_s".to_string(),
            serde_json::json!(stream_ms / 1000.0),
        );
        timing_obj.insert("total_s".to_string(), serde_json::json!(total_ms / 1000.0));

        // 优先从 tail 兜底提取 token（若当前尚无 token 数据）
        if log.input_tokens.is_none() && log.output_tokens.is_none() {
            if let Ok(full_tail) = std::str::from_utf8(&last_few_bytes) {
                for line in full_tail.lines().rev() {
                    if line.starts_with("data: ")
                        && (line.contains("\"usage\"") || line.contains("\"usageMetadata\""))
                    {
                        let json_str = line.trim_start_matches("data: ").trim();
                        if let Ok(json) = serde_json::from_str::<Value>(json_str) {
                            if let Some(usage) = json
                                .get("usage")
                                .or(json.get("usageMetadata"))
                                .or(json.get("response").and_then(|r| r.get("usage")))
                                .or(json.get("response").and_then(|r| r.get("usageMetadata")))
                            {
                                log.input_tokens = extract_input_tokens(usage);
                                log.output_tokens = extract_output_tokens(usage);
                                acc.cached_tokens =
                                    acc.cached_tokens.or_else(|| extract_cached_tokens(usage));
                                log.cached_tokens = log.cached_tokens.or(acc.cached_tokens);
                                acc.reasoning_tokens = acc
                                    .reasoning_tokens
                                    .or_else(|| extract_reasoning_tokens(usage));
                                break;
                            }
                        }
                    }
                }
            }
        }

        let has_actual_content = !acc.response_content.is_empty()
            || !acc.tool_calls.is_empty()
            || !acc.thinking_content.is_empty();

        let mut usage_obj = serde_json::Map::new();
        if has_actual_content || log.input_tokens.is_some() || log.output_tokens.is_some() {
            let input_toks = log.input_tokens.unwrap_or(0);
            let output_toks = log.output_tokens.unwrap_or(0);
            let cached_toks = acc.cached_tokens.or(log.cached_tokens).unwrap_or(0);
            let reasoning_toks = acc.reasoning_tokens.unwrap_or(0);

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
                usage_obj.insert(
                    "acc.cached_tokens".to_string(),
                    serde_json::json!(cached_toks),
                );
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
                    "acc.reasoning_tokens".to_string(),
                    serde_json::json!(reasoning_toks),
                );
            }
        }

        // 🌟 构建统一的满血简要版响应报文 (包含网关权威签名回填与确定性 Tool ID)
        let consolidated = build_canonical_consolidated_response(
            acc.thinking_content,
            acc.thinking_signature,
            acc.response_content,
            acc.tool_calls,
            log.session_id.as_deref(),
            &log.id,
            timing_obj,
            if usage_obj.is_empty() {
                None
            } else {
                Some(usage_obj)
            },
        );

        if consolidated
            .as_object()
            .map(|m| m.is_empty())
            .unwrap_or(true)
        {
            // Fallback: store raw SSE data if parsing failed
            log.response_body = Some(full_response.to_string());
        } else {
            log.response_body = Some(
                serde_json::to_string_pretty(&consolidated)
                    .unwrap_or_else(|_| full_response.to_string()),
            );
        }
    } else {
        log.response_body = Some(format!(
            "[Binary Stream Data: {} bytes]",
            all_stream_data.len()
        ));
    }

    // Fallback token extraction from tail if not already extracted
    if log.input_tokens.is_none() && log.output_tokens.is_none() {
        if let Ok(full_tail) = std::str::from_utf8(&last_few_bytes) {
            for line in full_tail.lines().rev() {
                if line.starts_with("data: ")
                    && (line.contains("\"usage\"") || line.contains("\"usageMetadata\""))
                {
                    let json_str = line.trim_start_matches("data: ").trim();
                    if let Ok(json) = serde_json::from_str::<Value>(json_str) {
                        if let Some(usage) = json
                            .get("usage")
                            .or(json.get("usageMetadata"))
                            .or(json.get("response").and_then(|r| r.get("usage")))
                            .or(json.get("response").and_then(|r| r.get("usageMetadata")))
                        {
                            log.input_tokens = extract_input_tokens(usage);
                            log.output_tokens = extract_output_tokens(usage);
                            log.cached_tokens =
                                log.cached_tokens.or_else(|| extract_cached_tokens(usage));
                            break;
                        }
                    }
                }
            }
        }
    }

    if log.status >= 400 {
        log.error = Some("Stream Error or Failed".to_string());
    }

    // Fallback input token estimation prefers the transit (upstream) body
    if log.input_tokens.is_none() {
        let estimated = log
            .upstream_request_body
            .as_ref()
            .or(log.request_body.as_ref())
            .map(|body| {
                crate::proxy::mappers::context_manager::estimate_raw_tokens_from_payload(body)
            })
            .unwrap_or(0);
        if estimated > 0 {
            log.input_tokens = Some(estimated);
        }
    }

    // Record User Token Usage
    record_user_token_usage(&user_token_identity, &log, user_agent.clone());

    monitor.log_request(log).await;
}
