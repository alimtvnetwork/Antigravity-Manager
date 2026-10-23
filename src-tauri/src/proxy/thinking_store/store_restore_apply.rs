use super::*;
use serde_json::{json, Value};
use std::sync::Arc;

impl ThinkingStore {
    /// Phase 4 (tail-first reverse fallback matching) + write-back application for
    /// [`ThinkingStore::restore_gemini_contents_with_model`].
    ///
    /// Split out so each source file stays within the 500-line limit.
    /// Pure code move from the original `thinking_store.rs` — no logic changes.
    pub(crate) fn apply_restore_matches(
        store_key: &str,
        contents: &mut Vec<Value>,
        target_model: Option<&str>,
        records: &[Arc<ThinkingRecord>],
        mut model_turns: Vec<ModelTurnMeta>,
        used: &mut [bool],
    ) -> usize {
        // Phase 4: Tail-first reverse pocket matching (only in plain text rounds, never borrow tool signatures across rounds to cause mutations in the next round!)
        if let Some(last_turn) = model_turns.last_mut() {
            if !last_turn.already_complete && last_turn.matched_record_idx.is_none() {
                let last_turn_has_tools =
                    !last_turn.tool_ids.is_empty() || !last_turn.tool_names.is_empty();
                if !last_turn_has_tools {
                    if let Some((last_unused_rec_idx, _)) =
                        records.iter().enumerate().rfind(|(idx, r)| {
                            if used[*idx] {
                                return false;
                            }
                            let r_has_tools = !r.tool_ids.is_empty() || !r.tool_names.is_empty();
                            !r_has_tools
                        })
                    {
                        last_turn.matched_record_idx = Some(last_unused_rec_idx);
                        used[last_unused_rec_idx] = true;
                    }
                }
            }
        }

        let mut restored = 0usize;
        for turn in model_turns {
            if turn.already_complete {
                continue;
            }
            let Some(rec_idx) = turn.matched_record_idx else {
                continue;
            };
            let rec = &records[rec_idx];
            if rec.thought.trim().is_empty() && rec.signature.is_none() {
                continue;
            }

            let Some(content) = contents.get_mut(turn.content_idx) else {
                continue;
            };
            let Some(parts) = content.get_mut("parts").and_then(|p| p.as_array_mut()) else {
                continue;
            };

            let has_unvalidated_function_call = parts
                .iter()
                .any(|p| p.get("functionCall").is_some() && !part_has_signature(p));

            let should_replace = is_placeholder_thought(&turn.existing_thought)
                || turn.existing_thought.len() < rec.thought.len()
                || (rec.signature.is_some() && !parts.iter().any(|p| part_has_signature(p)))
                || has_unvalidated_function_call;

            if !should_replace {
                continue;
            }

            parts.retain(|p| p.get("thought").and_then(|t| t.as_bool()) != Some(true));

            let has_meaningful_thought =
                !is_placeholder_thought(&rec.thought) && !rec.thought.trim().is_empty();

            let is_claude_target = target_model
                .map(|m| m.to_lowercase().contains("claude"))
                .unwrap_or_else(|| store_key.to_lowercase().contains("claude"));

            if has_meaningful_thought {
                let mut thought_part = json!({
                    "text": rec.thought.as_str(),
                    "thought": true,
                });

                // [DE-DUPLICATION & ELEVATION] Text residual thinking about text excision and deduplication:
                // Solve the problem of the thinking text that was downgraded to the main text when thinking is turned off, and is promoted and resurrected again when thinking is restarted, resulting in a double copy of the text remaining!
                let rec_thought_trimmed = rec.thought.trim();
                let rec_thought_norm = normalize_ws(&rec.thought);

                let mut cleaned_parts = Vec::with_capacity(parts.len());
                for part in parts.drain(..) {
                    let is_plain_text = part.get("text").is_some()
                        && part.get("functionCall").is_none()
                        && part.get("functionResponse").is_none()
                        && part.get("thought").and_then(|v| v.as_bool()) != Some(true);

                    if is_plain_text {
                        let text = part["text"].as_str().unwrap_or("");
                        let text_trimmed = text.trim();

                        // 0. If the text contains <think>...</think> tags, accurately cut off the tags and thinking content, and retain the remaining real text.
                        if let Some((_, rem)) = extract_think_tags(text) {
                            if !rem.is_empty() {
                                cleaned_parts.push(json!({ "text": rem }));
                            }
                            continue;
                        }

                        // 1. Identical (previously existing as an independent downgraded component): directly discard the component
                        if text_trimmed == rec_thought_trimmed
                            || (!rec_thought_norm.is_empty()
                                && normalize_ws(text) == rec_thought_norm)
                        {
                            continue;
                        }
                        // 2. The main text begins with a reflection text (the reflection text and the main text are merged into one component by the client): remove the prefix
                        if text.starts_with(&rec.thought) {
                            let remainder = &text[rec.thought.len()..];
                            let trimmed_rem =
                                remainder.trim_start_matches(|c| c == '\r' || c == '\n');
                            if !trimmed_rem.is_empty() {
                                cleaned_parts.push(json!({ "text": trimmed_rem }));
                            }
                            continue;
                        } else if text_trimmed.starts_with(rec_thought_trimmed) {
                            let remainder = &text_trimmed[rec_thought_trimmed.len()..];
                            let trimmed_rem =
                                remainder.trim_start_matches(|c| c == '\r' || c == '\n');
                            if !trimmed_rem.is_empty() {
                                cleaned_parts.push(json!({ "text": trimmed_rem }));
                            }
                            continue;
                        }
                    }
                    cleaned_parts.push(part);
                }
                *parts = cleaned_parts;

                if is_claude_target {
                    // Claude model: Upstream docking with Anthropic official signature verification engine!
                    // Anthropic official specification: The signature must and can only be on the thinking block (mapped to messages[x].content[0].signature)!
                    // Tool calls (tool_use / functionCall) must never carry signatures, and fake sentinels must not be injected!
                    if let Some(sig) = rec.signature.as_ref().filter(|s| is_real_signature(s)) {
                        // Key access control: Only when the historical signature is indeed Claude's signature, it will be mounted to thought_part!
                        // If the signature is generated by a heterogeneous model such as Gemini, it is absolutely prohibited to inject it into Claude to avoid 400 Invalid signature.
                        if is_claude_signature(sig) {
                            thought_part["thoughtSignature"] =
                                json!(ensure_google_claude_thought_signature(sig));
                        } else {
                            tracing::warn!(
                                "[ThinkingStore] Bypassing foreign non-Claude signature (len: {}) during restore for Claude model",
                                sig.len()
                            );
                        }
                    }
                    for part in parts.iter_mut() {
                        if let Some(obj) = part.as_object_mut() {
                            obj.remove("thoughtSignature");
                            obj.remove("thought_signature");
                        }
                    }
                } else {
                    // Gemini native: Return the real signature captured in this round to "the first non-thinking part of this round".
                    //
                    // The first think block remains pure think text (iron rule I4: think blocks never carry signatures), followed by
                    // `parts.insert(0, thought_part)` inserts index 0 - the anchor point naturally falls to index 1,
                    // It exactly reproduces the official arrangement of "thinking blocks + signed text".
                    let fallback = rec
                        .signature
                        .as_deref()
                        .filter(|s| is_real_signature(s) && is_likely_gemini_signature(s));
                    place_turn_signature(&mut *parts, fallback);
                }
                parts.insert(0, thought_part);
                restored += 1;
            } else {
                // 【2026-09-27】No substantial thinking content (placeholder/empty thinking): Never inject "..." placeholder thinking blocks!
                // The official standard form is "no thinking block + anchor point with signature" (baogao.txt 9/24 round).
                // The first non-thinking part serves as the anchor point to normally accept this round of signatures.
                if is_claude_target {
                    for part in parts.iter_mut() {
                        if let Some(obj) = part.as_object_mut() {
                            obj.remove("thoughtSignature");
                            obj.remove("thought_signature");
                        }
                    }
                } else {
                    let fallback = rec
                        .signature
                        .as_deref()
                        .filter(|s| is_real_signature(s) && is_likely_gemini_signature(s));
                    if place_turn_signature(&mut *parts, fallback).is_some() {
                        restored += 1;
                    }
                }
            }
        }

        if restored > 0 {
            tracing::info!(
                "[ThinkingStore] Restored {} full thinking block(s) for session {}",
                restored,
                store_key
            );
        }
        restored
    }
}
