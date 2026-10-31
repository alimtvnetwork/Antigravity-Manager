use super::*;

impl InboundThinkingPipeline {
    pub fn process_contents(
        contents: &mut Vec<Value>,
        protocol: ProxyProtocol,
        target_model: &str,
        is_thinking_enabled: bool,
        session_id: Option<&str>,
        is_retry: bool,
    ) {
        let trusts_signature = protocol.trusts_client_signature();
        let is_claude = target_model.to_lowercase().contains("claude");

        // 0. Tool call ID unified and normalized management (Pipeline First):
        // Completely solve the problem of inconsistency in tool call ID format between different clients (such as missing underscore `call573077` vs `call_573077`),
        // Align `functionCall.id` and `functionResponse.id` into the standard form with `_` (`call_...`).
        Self::normalize_tool_call_ids(contents);

        // 0.1 Unified context structure alignment (Pipeline First unified governance):
        // Merge consecutive user messages until the next model into multiple blocks (parts) of one user round, maintaining strict order.
        // This completely eliminates the round misalignment caused by the Adapter layer's siled approach, making the four major protocols 100% isomorphic after entering the pipeline!
        // Consecutive user rounds remain independent (aligned with official form).
        //
        // The historical implementation will merge the parts of consecutive user rounds into the previous content because
        // "Gemini enforces user/model alternation". However, there are consecutive user rounds in the official Antigravity message.
        // is the norm (`f81eae5c`’s contents[11] user message follows contents[12] SYSTEM_MESSAGE,
        // The two are independent), v1internal upstream does not require strict alternation.
        //
        // Actual measurement (2026-09-26, `gemini-3.8-flash-tiered`@daily):
        //   Two independent user content → 200, correct context understanding
        //   Merged into a single user content → 200, context understood correctly
        // Both are true, so they are kept independent according to the official form to obtain consistent prefix bytes across protocols.

        // Precompute the causal anchor for each round so that calls to Gemini's native tools without ID can also losslessly synthesize deterministic IDs
        let anchors: Vec<String> = (0..contents.len())
            .map(|i| {
                let preceding = if i > 0 { contents.get(i - 1) } else { None };
                crate::proxy::thinking_store::compute_causal_anchor(preceding)
            })
            .collect();

        // 1. Protocol policy cleaning and location standardization
        for (msg_idx, content) in contents.iter_mut().enumerate() {
            let _anchor = &anchors[msg_idx];
            let is_model = matches!(
                content.get("role").and_then(|r| r.as_str()),
                Some("model") | Some("assistant")
            );

            if let Some(parts) = content.get_mut("parts").and_then(|p| p.as_array_mut()) {
                if is_model {
                    let mut thinking_part = None;
                    let mut extra_thinking_parts = Vec::new();
                    let mut other_parts = Vec::new();
                    // [2026-09-27] When the placeholder block is discarded, if it carries a real signature, it will be temporarily stored here.
                    // During the assembly phase, transfer to the first non-thinking part (anchor point) of the round.
                    let mut placeholder_sig: Option<String> = None;

                    for mut part in parts.drain(..) {
                        // Iron rule: only recognize thought: true. The body part of the real machine message also carries the signature.
                        // Thinking blocks must not be judged based on "signature" - otherwise the visible answer will be rewritten as internal thinking,
                        // and silently discards the round's only signature.
                        let is_thought = crate::proxy::thinking_store::is_thought_part(&part);

                        if is_thought {
                            let text = part.get("text").and_then(|v| v.as_str()).unwrap_or("");
                            let is_placeholder =
                                crate::proxy::thinking_store::is_placeholder_thought(text);

                            // Verify client signature validity and model compatibility
                            let mut effective_sig = None;
                            if let Some(sig) = part
                                .get("thoughtSignature")
                                .or_else(|| part.get("thought_signature"))
                                .or_else(|| part.get("signature"))
                                .and_then(|s| s.as_str())
                            {
                                if sig == crate::proxy::thinking_store::SENTINEL_SIGNATURE {
                                    // The Claude model will never accept Gemini sentinel signatures to avoid triggering 400 Invalid signature
                                    if !is_claude {
                                        effective_sig = Some(sig.to_string());
                                    }
                                } else if trusts_signature && sig.len() >= 50 {
                                    let cached_family = crate::proxy::SignatureCache::global()
                                        .get_signature_family(sig);
                                    let compatible = match cached_family {
                                        Some(family) => {
                                            crate::proxy::mappers::common_utils::is_model_compatible(
                                                &family,
                                                target_model,
                                            ) || (is_claude
                                                && family.to_lowercase().contains("claude"))
                                        }
                                        None => {
                                            if target_model.to_lowercase().contains("gemini") {
                                                crate::proxy::thinking_store::is_likely_gemini_signature(sig)
                                            } else if is_claude {
                                                crate::proxy::thinking_store::is_claude_signature(
                                                    sig,
                                                )
                                            } else {
                                                true
                                            }
                                        }
                                    };
                                    if compatible {
                                        let final_sig = if is_claude {
                                            crate::proxy::thinking_store::ensure_google_claude_thought_signature(sig)
                                        } else {
                                            sig.to_string()
                                        };
                                        effective_sig = Some(final_sig);
                                    } else if target_model.to_lowercase().contains("gemini") {
                                        tracing::warn!(
                                            "[InboundPipeline] Stripping foreign signature (len: {}) from thought block for Gemini model {}",
                                            sig.len(), target_model
                                        );
                                        effective_sig = None;
                                    } else if is_claude {
                                        tracing::warn!(
                                            "[InboundPipeline] Stripping foreign signature (len: {}) from thought block for Claude model {}",
                                            sig.len(), target_model
                                        );
                                        effective_sig = None;
                                    }
                                }
                            }

                            // [2026-09-27] The placeholder thinking block is discarded directly (no thinking block is written), and the signature falls on the anchor point as usual.
                            //
                            // Official sample (baogao.txt 24 rounds): 9 rounds without thinking block but first non-thinking part with signature,
                            // "No thinking block + anchor with signature" is the official standard form. placeholder block (empty /
                            // "." / "..." / space / "·", etc.) have no information and should not appear in outbound messages:
                            // It used to be a "defensive placeholder" for the gateway (FIX #3382 / Layer-2 compression has been removed).
                            //
                            // If the placeholder block itself carries a real signature (the client mistakenly attaches the signature to the thinking block),
                            // The signature is not discarded with the thinking block - it is hung to the first non-thinking part (anchor) after being taken out.
                            // The place_turn_signature of finalize will eventually return according to the anchor rules.
                            let text_trimmed = text.trim();
                            if is_placeholder || text_trimmed.is_empty() {
                                if let Some(ref sig) = effective_sig {
                                    // The signature carried by the placeholder thinking block: transferred to the anchor point of this round (the first non-thinking part)
                                    placeholder_sig = Some(sig.clone());
                                    tracing::debug!(
                                        "[InboundPipeline] Placeholder thought block dropped; its signature (len: {}) kept for anchor backfill.",
                                        sig.len(),
                                    );
                                } else {
                                    tracing::debug!(
                                        "[InboundPipeline] Placeholder thought block dropped (text={:?}).",
                                        text_trimmed,
                                    );
                                }
                                continue;
                            }

                            let final_thought_text = text;

                            let mut thought_obj = json!({
                                "text": final_thought_text,
                                "thought": true,
                            });
                            if let Some(sig) = effective_sig {
                                // Iron Rule I4: A Gemini target's thought block **never** carries a signature.
                                // The client carrying a signature on the thinking block is a **contradictory combination**——
                                // In the official 33 model rounds, `thought:true` and signature appear together 0 times;
                                // This is especially true when the text is a placeholder (`...` / `·` / `[undefined]`, etc.)
                                // Typical client-side placeholder dirty data pollution.
                                //
                                // Stripping only thinkblock signatures for Gemini targets - Claude target press
                                // Anthropic specifications MUST carry signatures on think blocks.
                                // Don’t shut down your thinking because of this: that would reversely rewrite the semantics of the historical wheel.
                                if is_claude {
                                    thought_obj["thoughtSignature"] = json!(sig);
                                } else {
                                    tracing::debug!(
                                        "[InboundPipeline] Stripped signature from thought block for Gemini target (len: {}). I4: thought parts never carry signatures.",
                                        sig.len(),
                                    );
                                }
                            }

                            if thinking_part.is_none() {
                                thinking_part = Some(thought_obj);
                            } else {
                                // When there are multiple think blocks, the redundant think blocks that are not the first will be demoted to ordinary text.
                                if !final_thought_text.is_empty() {
                                    extra_thinking_parts
                                        .push(json!({ "text": final_thought_text }));
                                }
                            }
                        } else {
                            if target_model.to_lowercase().contains("gemini") {
                                // Gemini native: The real signature that comes with the client is passed through as it is, and no sentinel is created.
                                // Basis (3 official documents / 23 signatures): Sentinel appears 0 times, it does not belong
                                // Antigravity protocol; the absence of signatures is tolerated by upstream (absent in flywheel).
                                // Returning (including clearing non-anchor signatures) is uniformly handed over to place_turn_signature for final review.
                                if let Some(obj) = part.as_object_mut() {
                                    obj.remove("thought_signature");
                                }
                            } else if is_claude {
                                if let Some(obj) = part.as_object_mut() {
                                    obj.remove("thoughtSignature");
                                    obj.remove("thought_signature");
                                }
                            }
                            // Non-thinking parts: It may be ordinary text/process progress description (commentary), or it may be functionCall, etc.
                            let is_plain_text = part.get("text").is_some()
                                && part.get("functionCall").is_none()
                                && part.get("functionResponse").is_none();

                            if is_plain_text {
                                let raw_text =
                                    part.get("text").and_then(|v| v.as_str()).unwrap_or("");
                                if raw_text.trim().is_empty() {
                                    // Discard plain white text parts to avoid triggering Gemini 400 checks or breaking prefix cache hash stability
                                    continue;
                                }

                                // Cross-family protocol self-healing: Check whether the thinking content wrapped in the <think> tag is included (such as switching back to Gemini from Claude)
                                if let Some((extracted_thought, clean_visible)) =
                                    crate::proxy::thinking_store::extract_think_tags(raw_text)
                                {
                                    if thinking_part.is_none() && is_thinking_enabled {
                                        if crate::proxy::thinking_store::is_meaningful_thought(
                                            &extracted_thought,
                                        ) {
                                            // Iron Rule I4: Think blocks never carry signatures
                                            // (The part of thought:true in the official message does not have thoughtSignature)
                                            let t_obj = json!({
                                                "text": extracted_thought.as_str(),
                                                "thought": true,
                                            });
                                            thinking_part = Some(t_obj);
                                        }
                                    }
                                    if !clean_visible.is_empty() {
                                        other_parts.push(json!({ "text": clean_visible }));
                                    }
                                    continue;
                                }

                                // Protocol-independent self-healing: Check whether old version legacy thinking prefixes are included (such as **Thinking**)
                                if raw_text.trim_start().starts_with("**Thinking**") {
                                    let clean_thought = Self::strip_thinking_prefix(raw_text);
                                    if thinking_part.is_none() && is_thinking_enabled {
                                        if crate::proxy::thinking_store::is_meaningful_thought(
                                            &clean_thought,
                                        ) {
                                            // Iron Rule I4: Think blocks never carry signatures
                                            let t_obj = json!({
                                                "text": clean_thought,
                                                "thought": true,
                                            });
                                            thinking_part = Some(t_obj);
                                        }
                                    }
                                    // If there is already a thinking block, the remaining thinking block will be stripped off as an old copy to prevent secondary contamination of the text.
                                    continue;
                                }

                                other_parts.push(part);
                            } else {
                                other_parts.push(part);
                            }
                        }
                    }

                    // Core prefix order preservation: the first one is mandatory and there is only one thinking_part, and the rest of the text and tool calls follow closely.
                    let mut new_parts = Vec::with_capacity(parts.len() + 1);
                    if let Some(tp) = thinking_part {
                        new_parts.push(tp);
                    }
                    new_parts.extend(extra_thinking_parts);
                    new_parts.extend(other_parts);

                    // [2026-09-27] Signature transfer when placeholder block is discarded:
                    // If all the thinking blocks in this round are placeholders/empty (no thinking_part), or the placeholder blocks have real signatures,
                    // Hang the signature on the first non-thinking part (anchor) - the anchor may be text or
                    // is functionCall. The place_turn_signature of finalize will press "anchor=first
                    // The "non-thinking part" rule is finally returned to its place; if it is already hung here, place will also be used.
                    //
                    // ⚠️ The anchor point must be found after other_parts is merged: after the placeholder thinking is discarded, if this round
                    // The first non-thinking part is the text (including extra_thinking when none and thinking_part when none),
                    // Anchors only exist in other_parts - a premature search will miss the signature.
                    if let Some(sig) = placeholder_sig.take() {
                        if let Some(anchor) = new_parts
                            .iter_mut()
                            .find(|p| !crate::proxy::thinking_store::is_thought_part(p))
                        {
                            if let Some(obj) = anchor.as_object_mut() {
                                obj["thoughtSignature"] = json!(sig);
                            }
                        }
                    }

                    *parts = new_parts;
                } else {
                    // Universal inbound management of role == "user": in-depth deconstruction of multi-modal tool response (functionResponse)
                    // Ensure that the images included in the tool results of all protocols (OpenAI / Claude / Gemini / Responses) are promoted to independent inlineData visual perception input
                    let mut extra_inline_parts = Vec::new();
                    for part in parts.iter_mut() {
                        if let Some(fr) = part.get_mut("functionResponse") {
                            if let Some(resp) = fr.get_mut("response") {
                                for key in ["result", "output"] {
                                    if let Some(v) = resp.get_mut(key) {
                                        if let Some(s) = v.as_str() {
                                            if s.contains("data:image/") {
                                                let clean_s = crate::proxy::mappers::common_utils::extract_multimodal_from_tool_text(s, &mut extra_inline_parts);
                                                *v = json!(clean_s);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    parts.extend(extra_inline_parts);
                }
            }
        }

        // 2. Lossless resurrection of state machine (Hydration)
        // Global resurrection when thinking is turned on; if tool calls are included when thinking is turned off, resurrection is also performed to retrieve historical tool anti-counterfeiting signatures
        let has_function_call = contents.iter().any(|c| {
            c.get("parts")
                .and_then(|p| p.as_array())
                .map_or(false, |parts| {
                    parts.iter().any(|p| p.get("functionCall").is_some())
                })
        });
        if (is_thinking_enabled || has_function_call) && !is_retry {
            if let Some(s_id) = session_id {
                crate::proxy::thinking_store::hydrate_gemini_contents_with_model(
                    s_id,
                    contents,
                    Some(target_model),
                );
            }
        }

        // 3. Final review and desensitization standardization (Finalize)
        crate::proxy::thinking_store::finalize_gemini_contents_thinking_with_model(
            contents,
            is_thinking_enabled,
            Some(target_model),
        );

        // 4. Tool receipt role normalization (aligned with the official Antigravity form).
        //    Must be placed in the **last position**: if placed in front, the receipt wheel will enter the `is_model` branch above,
        //    Thus skipping the multimodal deconstruction in the `role == "user"` branch (the image is promoted to inlineData).
        Self::normalize_function_response_roles(contents);
    }
}
