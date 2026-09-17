use super::policy::ProxyProtocol;
use serde_json::{json, Value};

/// Unified inbound thinking pipeline (InboundThinkingPipeline)
/// Receives Google contents format converged from any client protocol and executes unidirectional processing:
/// 1. Protocol policy signature sanitization (Chat protocol drops client signatures; other protocols validate signatures)
/// 2. Thought block head-ordering enforcement and placeholder normalization
/// 3. State-machine lossless thought history hydration
/// 4. Finalization and prefix-cache format normalization (Finalize)
pub struct InboundThinkingPipeline;

impl InboundThinkingPipeline {
    /// Execute unified inbound processing
    pub fn process_contents(
        contents: &mut Vec<Value>,
        protocol: ProxyProtocol,
        target_model: &str,
        is_thinking_enabled: bool,
        session_id: Option<&str>,
        _is_retry: bool,
    ) {
        let trusts_signature = protocol.trusts_client_signature();

        // 1. Protocol policy sanitization and position normalization
        for content in contents.iter_mut() {
            let is_model = matches!(
                content.get("role").and_then(|r| r.as_str()),
                Some("model") | Some("assistant")
            );

            if let Some(parts) = content.get_mut("parts").and_then(|p| p.as_array_mut()) {
                if is_model {
                    let mut new_parts = Vec::with_capacity(parts.len());
                    let mut saw_non_thinking = false;

                    for part in parts.drain(..) {
                        let is_thought = part
                            .get("thought")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false)
                            || (part.get("thoughtSignature").is_some()
                                && part.get("functionCall").is_none()
                                && part.get("functionResponse").is_none());

                        if is_thought {
                            let text = part.get("text").and_then(|v| v.as_str()).unwrap_or("");
                            let is_placeholder =
                                crate::proxy::thinking_store::is_placeholder_thought(text);
                            let final_thought_text = if is_placeholder || text.is_empty() {
                                "..."
                            } else {
                                text.trim()
                            };

                            // If not in head position, or previous text parts already encountered, downgrade to plain text
                            if saw_non_thinking || !new_parts.is_empty() {
                                if !final_thought_text.is_empty() {
                                    new_parts.push(json!({ "text": final_thought_text }));
                                    saw_non_thinking = true;
                                }
                                continue;
                            }

                            // Validate client signature validity and model compatibility
                            let mut effective_sig = None;
                            if let Some(sig) = part
                                .get("thoughtSignature")
                                .or_else(|| part.get("thought_signature"))
                                .or_else(|| part.get("signature"))
                                .and_then(|s| s.as_str())
                            {
                                if sig == crate::proxy::thinking_store::SENTINEL_SIGNATURE {
                                    effective_sig = Some(sig.to_string());
                                } else if trusts_signature && sig.len() >= 50 {
                                    let cached_family = crate::proxy::SignatureCache::global()
                                        .get_signature_family(sig);
                                    let compatible = match cached_family {
                                        Some(family) => {
                                            crate::proxy::mappers::common_utils::is_model_compatible(
                                                &family,
                                                target_model,
                                            )
                                        }
                                        None => true,
                                    };
                                    if compatible {
                                        effective_sig = Some(sig.to_string());
                                    }
                                }
                            }

                            let mut thought_obj = json!({
                                 "text": final_thought_text,
                                 "thought": true,
                            });
                            if let Some(sig) = effective_sig {
                                thought_obj["thoughtSignature"] = json!(sig);
                            }
                            new_parts.push(thought_obj);
                        } else {
                            saw_non_thinking = true;
                            new_parts.push(part);
                        }
                    }
                    *parts = new_parts;
                }
            }
        }

        // 2. State-machine lossless hydration
        if is_thinking_enabled {
            if let Some(s_id) = session_id {
                crate::proxy::thinking_store::hydrate_gemini_contents(s_id, contents);
            }
        }

        // 3. Finalization and sanitization normalization (Finalize)
        crate::proxy::thinking_store::finalize_gemini_contents_thinking(
            contents,
            is_thinking_enabled,
        );
    }
}
