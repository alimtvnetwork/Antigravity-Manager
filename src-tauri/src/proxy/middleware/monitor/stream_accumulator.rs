use super::*;

/// Accumulates parsed content from SSE stream lines across the OpenAI,
/// Gemini, and Claude protocols.
///
/// Extracted from `monitor_middleware` via extract-method: each `apply_*`
/// method handles one protocol's event shape. Pure code motion — no logic changes.
pub(crate) struct SseParseAccumulator {
    pub(crate) thinking_content: String,
    pub(crate) response_content: String,
    pub(crate) thinking_signature: String,
    pub(crate) tool_calls: Vec<Value>,
    pub(crate) cached_tokens: Option<u32>,
    pub(crate) reasoning_tokens: Option<u32>,
}

impl SseParseAccumulator {
    pub(crate) fn new() -> Self {
        Self {
            thinking_content: String::new(),
            response_content: String::new(),
            thinking_signature: String::new(),
            tool_calls: Vec::new(),
            cached_tokens: None,
            reasoning_tokens: None,
        }
    }

    /// Apply one parsed SSE JSON event in OpenAI `choices[].delta` format.
    pub(crate) fn apply_openai_delta(&mut self, json: &Value) {
        if let Some(choices) = json.get("choices").and_then(|c| c.as_array()) {
            for choice in choices {
                if let Some(delta) = choice.get("delta") {
                    // Thinking/reasoning content
                    if let Some(thinking) = delta.get("reasoning_content").and_then(|v| v.as_str())
                    {
                        self.thinking_content.push_str(thinking);
                    }
                    // Thinking signature in OpenAI delta
                    if let Some(sig) = delta
                        .get("signature")
                        .or_else(|| delta.get("thought_signature"))
                        .and_then(|v| v.as_str())
                    {
                        self.thinking_signature = sig.to_string();
                    }
                    // Main response content
                    if let Some(content) = delta.get("content").and_then(|v| v.as_str()) {
                        self.response_content.push_str(content);
                    }
                    // Tool calls
                    if let Some(delta_tool_calls) =
                        delta.get("self.tool_calls").and_then(|t| t.as_array())
                    {
                        for tc in delta_tool_calls {
                            if let Some(index) = tc.get("index").and_then(|i| i.as_u64()) {
                                let idx = index as usize;
                                while self.tool_calls.len() <= idx {
                                    self.tool_calls.push(serde_json::json!({
                                        "id": "",
                                        "type": "function",
                                        "function": { "name": "", "arguments": "" }
                                    }));
                                }
                                let current_tc = &mut self.tool_calls[idx];
                                if let Some(id) = tc.get("id").and_then(|v| v.as_str()) {
                                    current_tc["id"] = Value::String(id.to_string());
                                }
                                if let Some(func) = tc.get("function") {
                                    if let Some(name) = func.get("name").and_then(|v| v.as_str()) {
                                        current_tc["function"]["name"] =
                                            Value::String(name.to_string());
                                    }
                                    if let Some(args) =
                                        func.get("arguments").and_then(|v| v.as_str())
                                    {
                                        let old_args = current_tc["function"]["arguments"]
                                            .as_str()
                                            .unwrap_or("");
                                        current_tc["function"]["arguments"] =
                                            Value::String(format!("{}{}", old_args, args));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Apply one parsed SSE JSON event in Gemini `candidates[].content.parts` format.
    pub(crate) fn apply_gemini_candidate(&mut self, json: &Value) {
        if let Some(candidates) = json.get("candidates").and_then(|c| c.as_array()) {
            for cand in candidates {
                if let Some(content) = cand.get("content") {
                    if let Some(parts) = content.get("parts").and_then(|p| p.as_array()) {
                        for part in parts {
                            if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                                if part
                                    .get("thought")
                                    .and_then(|t| t.as_bool())
                                    .unwrap_or(false)
                                {
                                    self.thinking_content.push_str(text);
                                } else {
                                    self.response_content.push_str(text);
                                }
                            }
                            let sig = part
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
                                .and_then(|s| s.as_str());
                            if let Some(s) = sig {
                                self.thinking_signature = s.to_string();
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
                                            self.tool_calls.len(),
                                        )
                                    };
                                    let args = fc
                                        .get("args")
                                        .map(|a| a.to_string())
                                        .unwrap_or_else(|| "{}".to_string());
                                    self.tool_calls.push(serde_json::json!({
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
        }
    }

    /// Apply one parsed SSE JSON event in Claude/Anthropic streaming format
    /// (content_block_start / content_block_delta / message_delta / Responses API),
    /// plus the legacy simplified-stream delta fallback.
    pub(crate) fn apply_claude_event(&mut self, json: &Value, log: &mut ProxyRequestLog) {
        let msg_type = json.get("type").and_then(|t| t.as_str());
        match msg_type {
            Some("content_block_start") => {
                if let (Some(index), Some(block)) = (
                    json.get("index").and_then(|i| i.as_u64()),
                    json.get("content_block"),
                ) {
                    let idx = index as usize;
                    if block.get("type").and_then(|t| t.as_str()) == Some("tool_use") {
                        let id = block.get("id").and_then(|v| v.as_str()).unwrap_or("");
                        let name = block.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        while self.tool_calls.len() <= idx {
                            self.tool_calls.push(Value::Null);
                        }
                        self.tool_calls[idx] = serde_json::json!({
                            "id": id,
                            "type": "function",
                            "function": { "name": name, "arguments": "" }
                        });
                    }
                    if let Some(thinking) = block.get("thinking").and_then(|v| v.as_str()) {
                        self.thinking_content.push_str(thinking);
                    }
                    if let Some(sig) = block
                        .get("signature")
                        .or_else(|| block.get("thought_signature"))
                        .or_else(|| block.get("thoughtSignature"))
                        .and_then(|v| v.as_str())
                    {
                        self.thinking_signature = sig.to_string();
                    }
                }
            }
            Some("content_block_delta") => {
                if let (Some(index), Some(delta)) = (
                    json.get("index").and_then(|i| i.as_u64()),
                    json.get("delta"),
                ) {
                    let idx = index as usize;

                    // Tool use input delta (Anthropic 官方协议为 partial_json 增量)
                    let delta_json = delta
                        .get("partial_json")
                        .or_else(|| delta.get("input_json_delta"))
                        .or_else(|| delta.get("text"))
                        .and_then(|v| v.as_str());

                    if let Some(delta_str) = delta_json {
                        if idx < self.tool_calls.len() {
                            if !self.tool_calls[idx].is_null() {
                                let old_args = self.tool_calls[idx]["function"]["arguments"]
                                    .as_str()
                                    .unwrap_or("");
                                self.tool_calls[idx]["function"]["arguments"] =
                                    Value::String(format!("{}{}", old_args, delta_str));
                            }
                        }
                    }
                    // Legacy/Native thinking block
                    if let Some(thinking) = delta.get("thinking").and_then(|v| v.as_str()) {
                        self.thinking_content.push_str(thinking);
                    }
                    // Thinking signature in delta (Claude signature_delta or direct signature)
                    if let Some(sig) = delta
                        .get("signature")
                        .or_else(|| delta.get("thought_signature"))
                        .or_else(|| delta.get("thoughtSignature"))
                        .and_then(|v| v.as_str())
                    {
                        self.thinking_signature = sig.to_string();
                    }
                    // Text content
                    if let Some(text) = delta.get("text").and_then(|v| v.as_str()) {
                        self.response_content.push_str(text);
                    }
                }
            }
            Some("message_delta") => {
                if let Some(delta) = json.get("delta") {
                    if let Some(usage) = delta.get("usage") {
                        if let Some(output_tokens) =
                            usage.get("output_tokens").and_then(|v| v.as_u64())
                        {
                            log.output_tokens = Some(output_tokens as u32);
                        }
                    }
                }
            }
            Some("response.output_text.delta") => {
                if let Some(text) = json.get("delta").and_then(|v| v.as_str()) {
                    self.response_content.push_str(text);
                }
            }
            Some("response.reasoning_summary_text.delta") => {
                if let Some(text) = json.get("delta").and_then(|v| v.as_str()) {
                    self.thinking_content.push_str(text);
                }
            }
            Some("response.output_item.added") => {
                if let Some(item) = json.get("item") {
                    if item.get("type").and_then(|t| t.as_str()) == Some("function_call") {
                        let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        let id = item.get("call_id").and_then(|v| v.as_str()).unwrap_or("");
                        self.tool_calls.push(serde_json::json!({
                            "id": id,
                            "type": "function",
                            "function": { "name": name, "arguments": "" }
                        }));
                    }
                }
            }
            Some("response.function_call_arguments.delta") => {
                if let Some(delta) = json.get("delta").and_then(|v| v.as_str()) {
                    if let Some(last_tc) = self.tool_calls.last_mut() {
                        let old_args = last_tc["function"]["arguments"].as_str().unwrap_or("");
                        last_tc["function"]["arguments"] =
                            Value::String(format!("{}{}", old_args, delta));
                    }
                }
            }
            _ => {}
        }
    }

    /// Extract token usage from one parsed SSE JSON event into the log.
    pub(crate) fn apply_token_usage(&mut self, json: &Value, log: &mut ProxyRequestLog) {
        if let Some(usage) = json
            .get("usage")
            .or(json.get("usageMetadata"))
            .or(json.get("response").and_then(|r| r.get("usage")))
            .or(json.get("response").and_then(|r| r.get("usageMetadata")))
        {
            log.input_tokens = extract_input_tokens(usage);
            log.output_tokens = extract_output_tokens(usage);
            self.cached_tokens = self.cached_tokens.or_else(|| extract_cached_tokens(usage));
            log.cached_tokens = log.cached_tokens.or(self.cached_tokens);
            self.reasoning_tokens = self
                .reasoning_tokens
                .or_else(|| extract_reasoning_tokens(usage));

            if log.input_tokens.is_none() && log.output_tokens.is_none() {
                log.output_tokens = usage
                    .get("total_tokens")
                    .or(usage.get("totalTokenCount"))
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u32);
            }
        }
    }
}
