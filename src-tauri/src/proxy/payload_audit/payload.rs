use super::*;
use super::simplify::simplify_content;
use super::simplify::simplify_responses_input_item;
use super::simplify::simplify_tools;
use super::simplify::truncate_chars;

pub fn simplify_payload_json(value: &Value) -> Value {
    let inner = value.get("request").unwrap_or(value);
    let mut concise = Map::new();

    // 1. Identifiers and session routing
    for key in [
        "_session_thinking_id",
        "sessionId",
        "session_id",
        "conversation_id",
        "chat_id",
        "thread_id",
        "previous_response_id",
        "requestId",
        "request_id",
        "id",
        "model",
        "stream",
        "temperature",
        "top_p",
        "top_k",
        "max_tokens",
        "max_output_tokens",
        "thinking",
        "reasoning_effort",
        "reasoning",
        "reasoning_content",
        "error",
        "gateway_error",
        "upstream_error",
        "type",
        "code",
        "status",
    ] {
        if let Some(v) = inner.get(key).or_else(|| value.get(key)) {
            concise.insert(key.to_string(), v.clone());
        }
    }

    // 2. Thought signature and thinking
    for sig_key in [
        "thinking_signature",
        "thought_signature",
        "signature",
        "thoughtSignature",
    ] {
        if let Some(v) = inner.get(sig_key).or_else(|| value.get(sig_key)) {
            concise.insert(sig_key.to_string(), v.clone());
        }
    }

    // 3. Response and top-level content
    if let Some(content) = inner.get("content").or_else(|| value.get("content")) {
        concise.insert("content".into(), simplify_content(content));
    }

    // 4. Top-level tool_calls
    if let Some(tool_calls) = inner.get("tool_calls").or_else(|| value.get("tool_calls")) {
        if let Some(arr) = tool_calls.as_array() {
            concise.insert(
                "tool_calls".into(),
                Value::Array(
                    arr.iter()
                        .map(|tc| {
                            if let Some(tc_obj) = tc.as_object() {
                                let mut tc_out = tc_obj.clone();
                                if let Some(func) = tc_obj.get("function") {
                                    tc_out.insert("function".into(), func.clone());
                                }
                                Value::Object(tc_out)
                            } else {
                                tc.clone()
                            }
                        })
                        .collect(),
                ),
            );
        } else {
            concise.insert("tool_calls".into(), tool_calls.clone());
        }
    }

    // 5. System Instructions
    if let Some(sys) = inner.get("system").or_else(|| value.get("system")) {
        concise.insert("system".into(), sys.clone());
    }
    if let Some(sys) = inner
        .get("systemInstruction")
        .or_else(|| value.get("systemInstruction"))
    {
        concise.insert("systemInstruction".into(), sys.clone());
    }

    // 5.5 OpenAI Responses 协议：instructions（系统指令）与 input（对话上下文）
    //     这两个是 Responses / Codex 的核心载荷，且不是 messages 的别名 —— 不保留会让
    //     日志里"系统提示词块与上下文块整段消失"（客户端原文看不到，而中转报文是 Gemini
    //     格式故看起来完整）。
    if let Some(instructions) = inner
        .get("instructions")
        .or_else(|| value.get("instructions"))
    {
        concise.insert("instructions".into(), instructions.clone());
    }
    if let Some(input) = inner.get("input").or_else(|| value.get("input")) {
        concise.insert(
            "input".into(),
            match input {
                Value::Array(arr) => {
                    Value::Array(arr.iter().map(simplify_responses_input_item).collect())
                }
                other => other.clone(),
            },
        );
    }

    // 6. Configs (generationConfig, thinkingConfig, toolConfig, tool_config, safetySettings)
    if let Some(cfg) = inner
        .get("generationConfig")
        .or_else(|| value.get("generationConfig"))
    {
        concise.insert("generationConfig".into(), cfg.clone());
    }
    if let Some(cfg) = inner
        .get("generation_config")
        .or_else(|| value.get("generation_config"))
    {
        concise.insert("generation_config".into(), cfg.clone());
    }
    if let Some(cfg) = inner.get("toolConfig").or_else(|| value.get("toolConfig")) {
        concise.insert("toolConfig".into(), cfg.clone());
    }
    if let Some(cfg) = inner
        .get("tool_config")
        .or_else(|| value.get("tool_config"))
    {
        concise.insert("tool_config".into(), cfg.clone());
    }
    if let Some(tc) = inner
        .get("tool_choice")
        .or_else(|| value.get("tool_choice"))
    {
        concise.insert("tool_choice".into(), tc.clone());
    }
    if let Some(ss) = inner
        .get("safetySettings")
        .or_else(|| value.get("safetySettings"))
    {
        concise.insert("safetySettings".into(), ss.clone());
    }

    // 7. Messages & Contents
    if let Some(messages) = inner.get("messages").or_else(|| value.get("messages")) {
        if let Some(arr) = messages.as_array() {
            concise.insert(
                "messages".into(),
                Value::Array(arr.iter().map(simplify_message).collect()),
            );
        }
    }
    if let Some(contents) = inner.get("contents").or_else(|| value.get("contents")) {
        if let Some(arr) = contents.as_array() {
            concise.insert(
                "contents".into(),
                Value::Array(arr.iter().map(simplify_message).collect()),
            );
        }
    }

    // 8. Tools (full Schema)
    if let Some(tools) = inner.get("tools").or_else(|| value.get("tools")) {
        concise.insert("tools".into(), simplify_tools(tools));
    }

    // 9. Usage Statistics
    if let Some(usage) = inner.get("usage").or_else(|| value.get("usage")) {
        concise.insert("usage".into(), usage.clone());
    }
    if let Some(usage) = inner
        .get("usageMetadata")
        .or_else(|| value.get("usageMetadata"))
    {
        concise.insert("usageMetadata".into(), usage.clone());
    }

    // 9.5 耗时诊断指标 (_timing)
    if let Some(timing) = inner.get("_timing").or_else(|| value.get("_timing")) {
        concise.insert("_timing".into(), timing.clone());
    }

    // 10. Choices & Candidates
    if let Some(choices) = value.get("choices").or_else(|| inner.get("choices")) {
        if let Some(arr) = choices.as_array() {
            concise.insert(
                "choices".into(),
                Value::Array(
                    arr.iter()
                        .map(|choice| {
                            if let Some(obj) = choice.as_object() {
                                let mut choice_out = obj.clone();
                                if let Some(msg) = obj.get("message") {
                                    choice_out.insert("message".into(), simplify_message(msg));
                                }
                                if let Some(delta) = obj.get("delta") {
                                    choice_out.insert("delta".into(), simplify_message(delta));
                                }
                                Value::Object(choice_out)
                            } else {
                                choice.clone()
                            }
                        })
                        .collect(),
                ),
            );
        } else {
            concise.insert("choices".into(), choices.clone());
        }
    }
    if let Some(candidates) = inner.get("candidates").or_else(|| value.get("candidates")) {
        if let Some(arr) = candidates.as_array() {
            concise.insert(
                "candidates".into(),
                Value::Array(
                    arr.iter()
                        .map(|cand| {
                            if let Some(obj) = cand.as_object() {
                                let mut cand_out = obj.clone();
                                if let Some(content) = obj.get("content") {
                                    cand_out.insert("content".into(), simplify_message(content));
                                }
                                Value::Object(cand_out)
                            } else {
                                cand.clone()
                            }
                        })
                        .collect(),
                ),
            );
        } else {
            concise.insert("candidates".into(), candidates.clone());
        }
    }

    if concise.is_empty() {
        return simplify_unknown(value);
    }

    // 按关注度重排字段：核心信息前置；未列入 CORE_FIELD_ORDER 的字段保持原相对顺序追加
    reorder_payload_fields(&Value::Object(concise))
}

fn simplify_unknown(value: &Value) -> Value {
    match value {
        Value::String(s) => Value::String(s.clone()),
        Value::Array(arr) => Value::Array(arr.iter().take(50).map(simplify_unknown).collect()),
        Value::Object(map) => {
            let mut out = Map::new();
            for (k, v) in map.iter().take(50) {
                out.insert(k.clone(), simplify_unknown(v));
            }
            Value::Object(out)
        }
        other => other.clone(),
    }
}

pub fn apply_storage_mode_to_body(raw: Option<String>, mode: &str) -> Option<String> {
    let raw = raw?;
    if mode != "simple" {
        return Some(raw);
    }
    match serde_json::from_str::<Value>(&raw) {
        Ok(json) => serde_json::to_string_pretty(&simplify_payload_json(&json))
            .ok()
            .or(Some(raw)),
        Err(_) => Some(truncate_chars(&raw, 8000)),
    }
}
