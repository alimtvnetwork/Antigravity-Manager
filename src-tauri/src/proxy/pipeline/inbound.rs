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
                    let mut thinking_part: Option<Value> = None;
                    let mut extra_thinking_parts = Vec::new();
                    let mut other_parts = Vec::new();

                    for part in parts.drain(..) {
                        let has_explicit_thought = part
                            .get("thought")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        let is_thought = if has_explicit_thought {
                            true
                        } else if part.get("functionCall").is_none() && part.get("functionResponse").is_none() {
                            part.get("thoughtSignature").is_some()
                        } else {
                            false
                        };

                        if is_thought {
                            let text = part.get("text").and_then(|v| v.as_str()).unwrap_or("");
                            let is_placeholder =
                                crate::proxy::thinking_store::is_placeholder_thought(text);

                            // Validate client signature validity and model compatibility
                            let is_claude = target_model.to_lowercase().contains("claude");
                            let mut effective_sig = None;
                            if let Some(sig) = part
                                .get("thoughtSignature")
                                .or_else(|| part.get("thought_signature"))
                                .or_else(|| part.get("signature"))
                                .and_then(|s| s.as_str())
                            {
                                if sig == crate::proxy::thinking_store::SENTINEL_SIGNATURE {
                                    // Claude models never accept Gemini sentinel signature to avoid 400 Invalid signature
                                    if !is_claude {
                                        effective_sig = Some(sig.to_string());
                                    }
                                } else if trusts_signature && sig.len() >= 50 {
                                    let cached_family = crate::proxy::SignatureCache::global()
                                        .get_signature_family(sig);
                                    let compatible = match cached_family {
                                        Some(family) => {
                                            if crate::proxy::mappers::common_utils::is_model_compatible(
                                                &family,
                                                target_model,
                                            ) {
                                                true
                                            } else if is_claude {
                                                family.to_lowercase().contains("claude")
                                            } else {
                                                false
                                            }
                                        }
                                        None => true,
                                    };
                                    if compatible {
                                        effective_sig = Some(sig.to_string());
                                    }
                                }
                            }

                            // Preserve raw thought text trailing newlines and whitespace without destructive trim
                            let is_blank_or_placeholder = is_placeholder || text.trim().is_empty();
                            let final_thought_text = if is_blank_or_placeholder {
                                if effective_sig.is_none() {
                                    "..."
                                } else {
                                    text
                                }
                            } else {
                                text
                            };

                            let mut thought_obj = json!({
                                "text": final_thought_text,
                                "thought": true,
                            });
                            if let Some(sig) = effective_sig {
                                thought_obj["thoughtSignature"] = json!(sig);
                            }

                            if thinking_part.is_none() {
                                thinking_part = Some(thought_obj);
                            } else {
                                // For multiple thinking blocks, downgrade subsequent ones to plain text
                                if !final_thought_text.is_empty() && final_thought_text != "..." {
                                    extra_thinking_parts
                                        .push(json!({ "text": final_thought_text }));
                                }
                            }
                        } else {
                            // Non-thinking part: plain text / progress commentary or functionCall
                            let is_call = part.get("functionCall").is_some() || part.get("functionResponse").is_some();
                            let is_plain_text = if is_call {
                                false
                            } else {
                                part.get("text").is_some()
                            };

                            if is_plain_text {
                                let raw_text =
                                    part.get("text").and_then(|v| v.as_str()).unwrap_or("");
                                if raw_text.trim().is_empty() {
                                    // Drop purely empty whitespace parts to prevent Gemini 400 validation error
                                    continue;
                                }

                                // Protocol-agnostic self-healing: check for legacy thinking prefix (**Thinking**)
                                if raw_text.trim_start().starts_with("**Thinking**") {
                                    let clean_thought = Self::strip_thinking_prefix(raw_text);
                                    if thinking_part.is_none() {
                                        let is_empty_clean = clean_thought.trim().is_empty();
                                        let final_thought = if is_empty_clean {
                                            "..."
                                        } else {
                                            &clean_thought
                                        };
                                        thinking_part = Some(json!({
                                            "text": final_thought,
                                            "thought": true,
                                            "thoughtSignature": crate::proxy::thinking_store::SENTINEL_SIGNATURE,
                                        }));
                                    }
                                    continue;
                                }

                                other_parts.push(part);
                            } else {
                                other_parts.push(part);
                            }
                        }
                    }

                    // Core prefix preservation: enforce single leading thinking_part, followed by other parts
                    let mut new_parts = Vec::with_capacity(parts.len() + 1);
                    if let Some(tp) = thinking_part {
                        new_parts.push(tp);
                    }
                    new_parts.extend(extra_thinking_parts);
                    new_parts.extend(other_parts);
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

    /// Strip legacy thinking prefix (**Thinking**)
    fn strip_thinking_prefix(text: &str) -> String {
        let trimmed = text.trim_start();
        if let Some(rest) = trimmed.strip_prefix("**Thinking**") {
            let rest = rest.trim_start_matches(':');
            rest.trim_start_matches(|c| c == '\r' || c == '\n' || c == ' ' || c == '\t')
                .to_string()
        } else {
            text.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preserves_process_commentary_alongside_tool_call() {
        let mut contents = vec![json!({
            "role": "model",
            "parts": [
                { "text": "Checking gateway and backend connection config." },
                {
                    "functionCall": {
                        "name": "inspect_case",
                        "args": { "case": "case_1" }
                    }
                }
            ]
        })];

        InboundThinkingPipeline::process_contents(
            &mut contents,
            ProxyProtocol::OpenAIResponses,
            "gemini-2.5-pro",
            true,
            None,
            false,
        );

        let parts = contents[0]["parts"].as_array().expect("parts array");
        assert!(parts[0]
            .get("thought")
            .and_then(Value::as_bool)
            .unwrap_or(false));
        assert_eq!(parts[1]["text"], "Checking gateway and backend connection config.");
        assert!(parts[2].get("functionCall").is_some());
    }

    #[test]
    fn test_preserves_multiple_plain_text_parts_intact() {
        let mut contents = vec![json!({
            "role": "model",
            "parts": [
                { "text": "Phase 1: Overview check." },
                { "text": "Phase 2: Deep diagnosis." },
                {
                    "functionCall": {
                        "name": "run_check",
                        "args": {}
                    }
                }
            ]
        })];

        InboundThinkingPipeline::process_contents(
            &mut contents,
            ProxyProtocol::OpenAIChat,
            "gemini-2.5-pro",
            false,
            None,
            false,
        );

        let parts = contents[0]["parts"].as_array().expect("parts array");
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0]["text"], "Phase 1: Overview check.");
        assert_eq!(parts[1]["text"], "Phase 2: Deep diagnosis.");
        assert!(parts[2].get("functionCall").is_some());
    }

    #[test]
    fn test_heals_legacy_thinking_prefix_without_corrupting_prose() {
        let mut contents = vec![json!({
            "role": "model",
            "parts": [
                { "text": "**Thinking**\n\nAnalyzed case data, preparing to invoke tool." },
                { "text": "Executing check now." },
                {
                    "functionCall": {
                        "name": "inspect",
                        "args": {}
                    }
                }
            ]
        })];

        InboundThinkingPipeline::process_contents(
            &mut contents,
            ProxyProtocol::OpenAIResponses,
            "gemini-2.5-pro",
            true,
            None,
            false,
        );

        let parts = contents[0]["parts"].as_array().expect("parts array");
        assert!(parts[0]
            .get("thought")
            .and_then(Value::as_bool)
            .unwrap_or(false));
        assert_eq!(parts[0]["text"], "Analyzed case data, preparing to invoke tool.");
        assert_eq!(parts[1]["text"], "Executing check now.");
        assert!(parts[2].get("functionCall").is_some());
    }
}
