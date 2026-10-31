use super::*;
use serde_json::{json, Value};

pub fn capture_gemini_contents(store_key: &str, contents: &[Value]) {
    if !crate::proxy::config::is_thinking_store_enabled() || store_key.is_empty() {
        return;
    }
    for (c_idx, content) in contents.iter().enumerate() {
        let role = content.get("role").and_then(|v| v.as_str()).unwrap_or("");
        if role != "model" && role != "assistant" {
            continue;
        }
        if let Some(parts) = content.get("parts").and_then(|p| p.as_array()) {
            let preceding_turn = if c_idx > 0 {
                contents.get(c_idx - 1)
            } else {
                None
            };
            let anchor = compute_causal_anchor(preceding_turn);
            capture_gemini_parts_with_anchor(store_key, parts, &anchor);
        }
    }
}

/// Capture client-supplied thinking, restore missing blocks, then prune compressed-away history.
///
/// Client histories usually have no real thinking (Claude/OpenAI). After a session is
/// warm in memory, this path is RAM-only: no SQLite open, no placeholder ingest, no prune
/// rewrite. JSON fill still copies stored thought text into the freshly built request.
pub fn hydrate_gemini_contents(store_key: &str, contents: &mut Vec<Value>) -> usize {
    hydrate_gemini_contents_with_model(store_key, contents, None)
}

pub fn hydrate_gemini_contents_with_model(
    store_key: &str,
    contents: &mut Vec<Value>,
    target_model: Option<&str>,
) -> usize {
    if store_key.is_empty() {
        return 0;
    }
    let store = ThinkingStore::global();
    store.touch_session(store_key);
    // Prioritize topology restoration to ensure that historical records are aligned with real signatures
    let restored = store.restore_gemini_contents_with_model(store_key, contents, target_model);
    // Only after the restoration is completed, if there are still legal substantive thinking blocks that come with the client, can they be safely absorbed into the database.
    if contents_have_capturable_thought(contents) {
        store.ingest_from_contents(store_key, contents);
    }
    store.prune_orphaned_records(store_key, contents);
    restored
}

pub(crate) fn is_model_or_assistant(content: &Value) -> bool {
    matches!(
        content.get("role").and_then(|v| v.as_str()),
        Some("model") | Some("assistant")
    )
}

pub(crate) fn contents_have_capturable_thought(contents: &[Value]) -> bool {
    for content in contents {
        if !is_model_or_assistant(content) {
            continue;
        }
        let Some(parts) = content.get("parts").and_then(|p| p.as_array()) else {
            continue;
        };
        for part in parts {
            let is_thought = part
                .get("thought")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !is_thought {
                continue;
            }
            let text = part.get("text").and_then(|t| t.as_str()).unwrap_or("");
            let sig = part
                .get("thoughtSignature")
                .or_else(|| part.get("thought_signature"))
                .and_then(|s| s.as_str());
            if is_capturable_thought(text, sig) {
                return true;
            }
        }
    }
    false
}

pub fn capture_gemini_parts(store_key: &str, parts: &[Value]) {
    capture_gemini_parts_with_anchor(store_key, parts, "root");
}

pub fn capture_gemini_parts_with_anchor(store_key: &str, parts: &[Value], anchor: &str) {
    let mut acc = TurnAccumulator::with_anchor(anchor);
    for part in parts {
        acc.ingest_part(part);
    }
    acc.commit(store_key);
}

pub fn capture_gemini_response(store_key: &str, response: &Value) {
    capture_gemini_response_with_preceding(store_key, response, None);
}

pub fn capture_gemini_response_with_preceding(
    store_key: &str,
    response: &Value,
    preceding: Option<&Value>,
) {
    let raw = response.get("response").unwrap_or(response);
    if let Some(parts) = raw
        .get("candidates")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("content"))
        .and_then(|c| c.get("parts"))
        .and_then(|p| p.as_array())
    {
        let anchor = compute_causal_anchor(preceding);
        capture_gemini_parts_with_anchor(store_key, parts, &anchor);
    }
}

/// The four major protocols unify the thinking and completion pipeline: ensure that all model rounds in Gemini contents must have legal thinking blocks and signatures when starting thinking.
pub fn finalize_gemini_contents_thinking(contents: &mut [Value], is_thinking_enabled: bool) {
    finalize_gemini_contents_thinking_with_model(contents, is_thinking_enabled, None);
}

pub fn finalize_gemini_contents_thinking_with_model(
    contents: &mut [Value],
    is_thinking_enabled: bool,
    target_model: Option<&str>,
) {
    // Precompute causal anchors for each round so that calls to Gemini's native tools without IDs can also synthesize deterministic IDs
    let anchors: Vec<String> = (0..contents.len())
        .map(|i| {
            let preceding = if i > 0 { contents.get(i - 1) } else { None };
            compute_causal_anchor(preceding)
        })
        .collect();

    for (msg_idx, msg) in contents.iter_mut().enumerate() {
        let _anchor = &anchors[msg_idx];
        let is_model = matches!(
            msg.get("role").and_then(|r| r.as_str()),
            Some("model") | Some("assistant")
        );

        if !is_model {
            // Non-model rounds (such as functionResponse of user rounds): clean up possible incorrect tag names that may be mixed in
            if let Some(parts) = msg.get_mut("parts").and_then(|p| p.as_array_mut()) {
                for part in parts.iter_mut() {
                    if let Some(obj) = part.as_object_mut() {
                        obj.remove("thought_signature");
                        if obj.contains_key("functionResponse") {
                            obj.remove("thoughtSignature");
                        }
                    }
                }
            }
            continue;
        }

        if let Some(parts) = msg.get_mut("parts").and_then(|p| p.as_array_mut()) {
            let mut thinking_parts = Vec::new();
            let mut other_parts = Vec::new();

            for mut part in parts.drain(..) {
                if let Some(obj) = part.as_object_mut() {
                    // Unified cleaning of non-standard snake fields sent to Google
                    obj.remove("thought_signature");
                }
                // Iron rule: only recognize thought: true. Both functionCall and plain text will carry thoughtSignature.
                // It must not be judged as a thinking block based on the signature alone, otherwise the first thought will be missed and the tool will be accidentally deleted when thinking.
                let is_thought = is_thought_part(&part);
                if is_thought {
                    thinking_parts.push(part);
                } else {
                    other_parts.push(part);
                }
            }

            let is_claude_turn = target_model
                .map(|m| m.to_lowercase().contains("claude"))
                .unwrap_or(false);

            // 1. Extract the legal real signature of the current round (purely check the signature of the current round component)
            // [DECOUPLE 2026-09-26] No longer assume that "the signature is only on functionCall" - official new rules:
            // The first non-thinking part of any round (text or functionCall) may carry a signature.
            // Any non-thinking part comes with a legal signature, which is turn_real_sig.
            let turn_real_sig: Option<String> = other_parts.iter().find_map(|p| {
                if p.get("functionCall").is_some() || p.get("text").is_some() {
                    let sig = p
                        .get("thoughtSignature")
                        .and_then(|s| s.as_str())
                        .filter(|s| is_real_signature(s))
                        .map(str::to_string);
                    sig.filter(|s| {
                        if is_claude_turn {
                            is_claude_signature(s)
                        } else {
                            is_likely_gemini_signature(s)
                        }
                    })
                } else {
                    None
                }
            });

            // 1.1 [FIX 2026-09-26] Under Gemini target, pure fc round (the first non-thinking part is functionCall) if itself
            //     No signature is carried, and must be backfilled by tool_id from the signature cache - official/upstream rules:
            //     "The first non-thinking part of each round must have thought_signature, and subsequent parts do not need to have it."
            //     400 on-site irrefutable evidence: `Function call is missing a thought_signature... call default_api:grep, position 4`,
            //     The signature of tool_id is actually cached in tool_signatures (1384 B), but it is never backfilled by id during assembly.
            let cached_tool_sig: Option<String> = if !is_claude_turn && turn_real_sig.is_none() {
                other_parts.iter().find_map(|p| {
                    let fc = p.get("functionCall")?;
                    let id = fc.get("id").and_then(|v| v.as_str())?;
                    crate::proxy::SignatureCache::global()
                        .get_tool_signature(id)
                        .filter(|s| is_likely_gemini_signature(s))
                })
            } else {
                None
            };

            // 2. Signature return (final outbound gatekeeper):
            // The core iron rule: the signature is only written on the "first non-thinking part of the round", and the signature field is deleted in all other parts.
            // Based on 3 official messages / 23 signatures: no more than 1 signature per round, must land at the anchor point; sentry appears 0 times.
            if is_claude_turn {
                // Claude model: Anthropic official specifications require that signatures must and can only be on the thinking block. Tool calls must never carry signatures, let alone fake sentinels.
                for part in other_parts.iter_mut() {
                    if let Some(obj) = part.as_object_mut() {
                        obj.remove("thoughtSignature");
                        obj.remove("thought_signature");
                    }
                }
            } else {
                // Gemini native: The real signature is given priority to be kept as it is. If it is missing, the backfill source will be used. If there is none, it will be left blank.
                // Backfill source priority: the current round comes with turn_real_sig → check the signature cache cached_tool_sig by tool_id.
                // (The historical fc round retransmitted by the client does not have its own signature, but it is in the tool_signatures cache and must be backfilled.
                //   Otherwise "the first non-thinking part=fc and no signature" will be rejected by the upstream 400. )
                let fallback = turn_real_sig
                    .as_deref()
                    .filter(|s| is_likely_gemini_signature(s))
                    .or_else(|| {
                        cached_tool_sig
                            .as_deref()
                            .filter(|s| is_likely_gemini_signature(s))
                    });
                place_turn_signature(&mut other_parts, fallback);
            }

            // 3. Governance Thinking Parts (thinking_parts)
            if is_thinking_enabled {
                if is_claude_turn {
                    // Claude Model: The Anthropic engine enforces that signatures must and can only be on think blocks!
                    // Tool calls (functionCall) completely strip signatures and never inject fake sentinels
                    if thinking_parts.is_empty() {
                        if let Some(ref real_sig) = turn_real_sig {
                            if is_claude_signature(real_sig) {
                                let mut thought_obj = json!({
                                    "text": "...",
                                    "thought": true,
                                });
                                thought_obj["thoughtSignature"] =
                                    json!(ensure_google_claude_thought_signature(real_sig));
                                thinking_parts.push(thought_obj);
                            }
                        }
                    } else if let Some(ref real_sig) = turn_real_sig {
                        if is_claude_signature(real_sig) {
                            let wrapped = ensure_google_claude_thought_signature(real_sig);
                            for tp in thinking_parts.iter_mut() {
                                tp["thoughtSignature"] = json!(wrapped);
                            }
                        } else {
                            for tp in thinking_parts.iter_mut() {
                                if let Some(obj) = tp.as_object_mut() {
                                    obj.remove("thoughtSignature");
                                }
                            }
                        }
                    } else {
                        let mut valid_thinking = Vec::new();
                        for mut tp in thinking_parts.drain(..) {
                            let has_valid_sig = tp
                                .get("thoughtSignature")
                                .and_then(|s| s.as_str())
                                .map(|s| {
                                    s != SENTINEL_SIGNATURE
                                        && s.len() >= 50
                                        && is_claude_signature(s)
                                })
                                .unwrap_or(false);
                            if has_valid_sig {
                                if let Some(sig) =
                                    tp.get("thoughtSignature").and_then(|s| s.as_str())
                                {
                                    tp["thoughtSignature"] =
                                        json!(ensure_google_claude_thought_signature(sig));
                                }
                                valid_thinking.push(tp);
                            } else {
                                // Thinking blocks without legal signatures (such as Gemini historical thinking blocks that cross-cut to Claude):
                                // Anthropic enforces that think block signatures must be valid. When there is no valid Claude signature,
                                // Downgrade the thinking content to the main text with the <think> tag and put it first. This not only retains the thinking context completely, but also completely avoids Anthropic 400 verification errors!
                                if let Some(text) = tp.get("text").and_then(|t| t.as_str()) {
                                    if text != "..." && !text.trim().is_empty() {
                                        let wrapped = if text.trim_start().starts_with("<think>") {
                                            text.to_string()
                                        } else {
                                            format!("<think>\n{}\n</think>\n\n", text.trim())
                                        };
                                        other_parts.insert(0, json!({ "text": wrapped }));
                                    }
                                }
                            }
                        }
                        thinking_parts = valid_thinking;
                    }
                } else {
                    // Gemini native: Thought blocks never carry signatures (iron rule I4), and placeholder think blocks are never injected out of thin air.
                    //
                    // In the official message, the functionCall wheel is "pure" - only functionCall, without any
                    // thought part (thought × functionCall appears 0 times in 33 model rounds).
                    // Injecting {text:"...", thought:true} creates a permutation that is never produced officially,
                    // And squeeze the signature anchor from parts[0] to parts[1].
                    for tp in thinking_parts.iter_mut() {
                        if let Some(obj) = tp.as_object_mut() {
                            obj.remove("thoughtSignature");
                            obj.remove("thought_signature");
                        }
                    }
                }

                // The think block is always forced to come first, with other components following closely behind.
                parts.extend(thinking_parts);
            } else {
                // When thinking mode is off:
                // 1. Never actively inject any placeholder blocks (such as "...");
                // 2. If the thinking block contains substantive thinking content, the single outbound message will be downgraded to ordinary text to prevent loss of semantics, and the thought: true mark will be removed;
                // 3. Pure placeholders are directly eliminated and never sent to the thought: true structure.
                for tp in thinking_parts {
                    let text = tp.get("text").and_then(|t| t.as_str()).unwrap_or("");
                    if is_meaningful_thought(text) {
                        let wrapped = if is_claude_turn {
                            if text.trim_start().starts_with("<think>") {
                                text.to_string()
                            } else {
                                format!("<think>\n{}\n</think>\n\n", text.trim())
                            }
                        } else {
                            text.to_string()
                        };
                        parts.push(json!({ "text": wrapped }));
                    }
                }
            }

            parts.extend(other_parts);
        }
    }
}
