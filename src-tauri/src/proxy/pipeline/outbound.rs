use super::events::CanonicalStreamEvent;
use super::policy::ProxyProtocol;
use super::usage::CanonicalUsage;
use serde_json::{json, Value};

/// Unified outbound extraction payload
#[derive(Debug, Clone, Default)]
pub struct CanonicalEgressPayload {
    pub text: String,
    pub thought: Option<String>,
    pub signature: Option<String>,
    pub tool_calls: Vec<Value>,
    pub usage: CanonicalUsage,
    pub finish_reason: Option<String>,
}

/// Unified outbound thinking and response pipeline (OutboundThinkingPipeline)
/// Converges upstream responses into domain objects, triggers reverse ingestion and auditing,
/// and diffuses to target client protocols.
pub struct OutboundThinkingPipeline;

impl OutboundThinkingPipeline {
    /// Extract and converge domain object from Google Gemini complete response
    pub fn extract_payload(gemini_resp: &Value) -> CanonicalEgressPayload {
        let mut text = String::new();
        let mut thought = None;
        let mut signature = None;
        let mut tool_calls = Vec::new();

        let raw = gemini_resp.get("response").unwrap_or(gemini_resp);

        if let Some(cands) = raw.get("candidates").and_then(|c| c.as_array()) {
            if let Some(first) = cands.first() {
                if let Some(parts) = first
                    .get("content")
                    .and_then(|c| c.get("parts"))
                    .and_then(|p| p.as_array())
                {
                    for part in parts {
                        let is_thought = part
                            .get("thought")
                            .and_then(|t| t.as_bool())
                            .unwrap_or(false);
                        if is_thought {
                            if let Some(t) = part.get("text").and_then(|s| s.as_str()) {
                                thought = Some(t.to_string());
                            }
                            if let Some(sig) = part
                                .get("thoughtSignature")
                                .or_else(|| part.get("thought_signature"))
                                .and_then(|s| s.as_str())
                            {
                                signature = Some(sig.to_string());
                            }
                        } else if let Some(fc) = part.get("functionCall") {
                            tool_calls.push(fc.clone());
                            if signature.is_none() {
                                if let Some(sig) = part
                                    .get("thoughtSignature")
                                    .or_else(|| part.get("thought_signature"))
                                    .and_then(|s| s.as_str())
                                {
                                    signature = Some(sig.to_string());
                                }
                            }
                        } else if let Some(t) = part.get("text").and_then(|s| s.as_str()) {
                            text.push_str(t);
                        }
                    }
                }
            }
        }

        let usage = CanonicalUsage::from_gemini(raw);
        let finish_reason = raw
            .get("candidates")
            .and_then(|c| c.get(0))
            .and_then(|cand| cand.get("finishReason"))
            .and_then(|f| f.as_str())
            .map(|s| s.to_string());

        CanonicalEgressPayload {
            text,
            thought,
            signature,
            tool_calls,
            usage,
            finish_reason,
        }
    }

    /// Execute reverse ingestion (minimalist principle: only persist substantive thoughts and valid cryptographic signatures)
    pub fn auto_ingest(store_key: &str, payload: &CanonicalEgressPayload, message_count: usize) {
        if let Some(ref sig) = payload.signature {
            if sig != crate::proxy::thinking_store::SENTINEL_SIGNATURE && sig.len() >= 50 {
                // Cache session signature
                crate::proxy::SignatureCache::global().cache_session_signature(
                    store_key,
                    sig.clone(),
                    message_count,
                );

                // Associate tool call signature
                for tc in &payload.tool_calls {
                    if let Some(id) = tc.get("id").and_then(|i| i.as_str()) {
                        crate::proxy::SignatureCache::global()
                            .cache_tool_signature(id, sig.clone());
                    }
                }
            }
        }

        // Ingest into ThinkingStore
        if let Some(ref th) = payload.thought {
            if crate::proxy::thinking_store::is_meaningful_thought(th) {
                let tool_ids: Vec<String> = payload
                    .tool_calls
                    .iter()
                    .filter_map(|tc| tc.get("id").and_then(|i| i.as_str()).map(str::to_string))
                    .collect();

                let fp = crate::proxy::thinking_store::fingerprint(&payload.text, &tool_ids, &[]);

                crate::proxy::thinking_store::ThinkingStore::global().record(
                    store_key,
                    crate::proxy::thinking_store::ThinkingRecord {
                        fingerprint: fp,
                        thought: th.clone(),
                        signature: payload.signature.clone(),
                        tool_ids,
                        tool_names: Vec::new(),
                        visible: payload.text.clone(),
                    },
                );
            }
        }
    }

    /// Diffuse to OpenAI Chat protocol response (suppress signature, inject standard usage)
    pub fn diffuse_to_openai_chat(
        payload: &CanonicalEgressPayload,
        model: &str,
        request_id: &str,
    ) -> Value {
        let mut message_obj = json!({
            "role": "assistant",
            "content": if payload.text.is_empty() { Value::Null } else { json!(payload.text) },
        });

        // Chat protocol only accepts reasoning_content, never emits signature
        if let Some(ref th) = payload.thought {
            message_obj["reasoning_content"] = json!(th);
        }

        if !payload.tool_calls.is_empty() {
            let mut formatted_tcs = Vec::new();
            for (idx, tc) in payload.tool_calls.iter().enumerate() {
                let name = tc.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
                let args = tc.get("args").cloned().unwrap_or(json!({}));
                let id = tc
                    .get("id")
                    .and_then(|i| i.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("call_{}_{}", name, idx));
                formatted_tcs.push(json!({
                    "id": id,
                    "type": "function",
                    "function": {
                        "name": name,
                        "arguments": serde_json::to_string(&args).unwrap_or_else(|_| "{}".to_string())
                    }
                }));
            }
            message_obj["tool_calls"] = json!(formatted_tcs);
        }

        json!({
            "id": request_id,
            "object": "chat.completion",
            "created": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(),
            "model": model,
            "choices": [
                {
                    "index": 0,
                    "message": message_obj,
                    "finish_reason": payload.finish_reason.as_deref().unwrap_or("stop")
                }
            ],
            "usage": payload.usage.to_openai_chat_usage()
        })
    }

    /// Diffuse to Anthropic Claude protocol response (emit content_block::thinking and signature)
    pub fn diffuse_to_claude(
        payload: &CanonicalEgressPayload,
        model: &str,
        request_id: &str,
        scaling_enabled: bool,
        context_limit: u32,
    ) -> Value {
        let mut content = Vec::new();

        // Inject thinking block and signature
        if let Some(ref th) = payload.thought {
            let mut th_block = json!({
                "type": "thinking",
                "thinking": th,
            });
            if let Some(ref sig) = payload.signature {
                th_block["signature"] = json!(sig);
            }
            content.push(th_block);
        }

        if !payload.text.is_empty() {
            content.push(json!({
                "type": "text",
                "text": payload.text
            }));
        }

        for (idx, tc) in payload.tool_calls.iter().enumerate() {
            let name = tc.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
            let args = tc.get("args").cloned().unwrap_or(json!({}));
            let id = tc
                .get("id")
                .and_then(|i| i.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("call_{}_{}", name, idx));
            content.push(json!({
                "type": "tool_use",
                "id": id,
                "name": name,
                "input": args
            }));
        }

        json!({
            "id": request_id,
            "type": "message",
            "role": "assistant",
            "model": model,
            "content": content,
            "stop_reason": payload.finish_reason.as_deref().unwrap_or("end_turn"),
            "stop_sequence": Value::Null,
            "usage": payload.usage.to_claude_usage(scaling_enabled, context_limit)
        })
    }

    /// Diffuse streaming incremental event (Streaming SSE Event Diffuser)
    pub fn diffuse_stream_event(
        event: &CanonicalStreamEvent,
        protocol: ProxyProtocol,
    ) -> Option<String> {
        match event {
            CanonicalStreamEvent::ThoughtDelta(th) => match protocol {
                ProxyProtocol::OpenAIChat => Some(format!(
                    "data: {}\n\n",
                    json!({
                        "choices": [{
                            "index": 0,
                            "delta": { "reasoning_content": th }
                        }]
                    })
                )),
                ProxyProtocol::AnthropicClaude => Some(format!(
                    "event: content_block_delta\ndata: {}\n\n",
                    json!({
                        "type": "content_block_delta",
                        "index": 0,
                        "delta": { "type": "thinking_delta", "thinking": th }
                    })
                )),
                _ => None,
            },
            CanonicalStreamEvent::ThoughtSignature(sig) => {
                // OpenAI Chat protocol strictly drops signatures, never emits to client
                if !protocol.emits_signature_to_client() {
                    return None;
                }
                match protocol {
                    ProxyProtocol::AnthropicClaude => Some(format!(
                        "event: content_block_delta\ndata: {}\n\n",
                        json!({
                            "type": "content_block_delta",
                            "index": 0,
                            "delta": { "type": "signature_delta", "signature": sig }
                        })
                    )),
                    _ => None,
                }
            }
            CanonicalStreamEvent::TextDelta(txt) => match protocol {
                ProxyProtocol::OpenAIChat => Some(format!(
                    "data: {}\n\n",
                    json!({
                        "choices": [{
                            "index": 0,
                            "delta": { "content": txt }
                        }]
                    })
                )),
                ProxyProtocol::AnthropicClaude => Some(format!(
                    "event: content_block_delta\ndata: {}\n\n",
                    json!({
                        "type": "content_block_delta",
                        "index": 1,
                        "delta": { "type": "text_delta", "text": txt }
                    })
                )),
                _ => None,
            },
            CanonicalStreamEvent::UsageUpdate(u) => match protocol {
                ProxyProtocol::OpenAIChat => Some(format!(
                    "data: {}\n\n",
                    json!({
                        "choices": [],
                        "usage": u.to_openai_chat_usage()
                    })
                )),
                _ => None,
            },
            CanonicalStreamEvent::Finish { reason, usage } => match protocol {
                ProxyProtocol::OpenAIChat => {
                    let mut obj = json!({
                        "choices": [{
                            "index": 0,
                            "delta": {},
                            "finish_reason": reason
                        }]
                    });
                    if let Some(u) = usage {
                        obj["usage"] = u.to_openai_chat_usage();
                    }
                    Some(format!("data: {}\n\ndata: [DONE]\n\n", obj))
                }
                ProxyProtocol::AnthropicClaude => {
                    Some("event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n".to_string())
                }
                _ => None,
            },
            _ => None,
        }
    }
}
