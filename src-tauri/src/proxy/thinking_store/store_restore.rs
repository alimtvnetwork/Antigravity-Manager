use super::*;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

impl ThinkingStore {
    pub fn restore_gemini_contents(&self, store_key: &str, contents: &mut Vec<Value>) -> usize {
        self.restore_gemini_contents_with_model(store_key, contents, None)
    }

    pub fn restore_gemini_contents_with_model(
        &self,
        store_key: &str,
        contents: &mut Vec<Value>,
        target_model: Option<&str>,
    ) -> usize {
        if !crate::proxy::config::is_thinking_store_enabled() {
            return 0;
        }
        if contents.is_empty() || store_key.is_empty() {
            return 0;
        }

        let mut records = self.load_turns(store_key);

        let mut model_turns: Vec<ModelTurnMeta> = Vec::new();
        for (c_idx, content) in contents.iter().enumerate() {
            let role = content.get("role").and_then(|v| v.as_str()).unwrap_or("");
            if role != "model" && role != "assistant" {
                continue;
            }
            let Some(parts) = content.get("parts").and_then(|p| p.as_array()) else {
                continue;
            };
            let preceding_turn = if c_idx > 0 {
                contents.get(c_idx - 1)
            } else {
                None
            };
            let anchor = compute_causal_anchor(preceding_turn);
            let (visible, tool_ids, tool_names, existing_thought) =
                inspect_parts_with_anchor(parts, &anchor);
            let existing_sig = parts.iter().find_map(|p| {
                let sig = p
                    .get("thoughtSignature")
                    .or_else(|| p.get("thought_signature"))
                    .and_then(|s| s.as_str())
                    .filter(|s| is_real_signature(s))
                    .map(str::to_string);
                sig.or_else(|| {
                    p.get("functionCall")
                        .and_then(|fc| fc.get("id"))
                        .and_then(|id| id.as_str())
                        .and_then(|id| {
                            crate::proxy::SignatureCache::global().get_tool_signature(id)
                        })
                        .filter(|s| is_real_signature(s))
                })
            });
            let already_complete = !turn_needs_restore(parts, &existing_thought);
            // Agent tool turns match by tool_id (Phase 1). Skip fingerprint /
            // whitespace-normalize until a later phase actually needs them.
            model_turns.push(ModelTurnMeta {
                content_idx: c_idx,
                visible,
                norm_visible: String::new(),
                tool_ids,
                tool_names,
                existing_thought,
                existing_sig,
                fp: String::new(),
                matched_record_idx: None,
                already_complete,
            });
        }

        if model_turns.is_empty() {
            return 0;
        }

        let mut used = vec![false; records.len()];
        let mut by_sig: HashMap<String, usize> = HashMap::new();
        let mut by_tool: HashMap<String, Vec<usize>> = HashMap::new();
        let mut by_fp: HashMap<&str, Vec<usize>> = HashMap::new();
        for (rec_idx, rec) in records.iter().enumerate() {
            if let Some(ref sig) = rec.signature.as_ref().filter(|s| is_real_signature(s)) {
                let norm = normalize_signature_for_comparison(sig);
                by_sig.insert(norm.into_owned(), rec_idx);
            }
            for id in &rec.tool_ids {
                let norm = crate::proxy::common::utils::normalize_tool_id(id);
                by_tool.entry(norm.to_string()).or_default().push(rec_idx);
                if norm.as_ref() != id.as_str() {
                    by_tool.entry(id.clone()).or_default().push(rec_idx);
                }
            }
            by_fp
                .entry(rec.fingerprint.as_str())
                .or_default()
                .push(rec_idx);
        }

        // Phase 0: Direct verification of real signatures (highest priority: if the client history has its own real signature, direct and accurate reverse matching)
        for turn in model_turns.iter_mut() {
            if turn.already_complete || turn.matched_record_idx.is_some() {
                continue;
            }
            if let Some(ref sig) = turn.existing_sig {
                let norm = normalize_signature_for_comparison(sig);
                if let Some(&rec_idx) = by_sig.get(norm.as_ref()) {
                    if !used[rec_idx] {
                        // Anti-mismatch protection: tool call rounds must never match plain text records, and plain text rounds must never match tool records!
                        let turn_has_tools =
                            !turn.tool_ids.is_empty() || !turn.tool_names.is_empty();
                        let rec_has_tools = !records[rec_idx].tool_ids.is_empty()
                            || !records[rec_idx].tool_names.is_empty();
                        if turn_has_tools == rec_has_tools {
                            let tool_names_match = if turn.tool_names.is_empty()
                                || records[rec_idx].tool_names.is_empty()
                            {
                                true
                            } else {
                                turn.tool_names == records[rec_idx].tool_names
                            };
                            if tool_names_match {
                                turn.matched_record_idx = Some(rec_idx);
                                used[rec_idx] = true;
                            }
                        }
                    }
                }
            }
        }

        // Phase 1: Accurate anchoring of tool call IDs (including native unique IDs and deterministic synthetic IDs, forward order-preserving matching)
        for turn in model_turns.iter_mut() {
            if turn.already_complete
                || turn.matched_record_idx.is_some()
                || turn.tool_ids.is_empty()
            {
                continue;
            }
            for id in &turn.tool_ids {
                let norm = crate::proxy::common::utils::normalize_tool_id(id);
                let idxs = by_tool
                    .get(norm.as_ref())
                    .or_else(|| by_tool.get(id.as_str()));
                let Some(idxs) = idxs else {
                    continue;
                };
                if let Some(&rec_idx) = idxs.iter().find(|&&i| {
                    if used[i] {
                        return false;
                    }
                    // Tightly defend against tool name mismatches: Prevent ID collisions from causing thoughts and signatures from other tools to be linked to the current tool!
                    if !turn.tool_names.is_empty() && !records[i].tool_names.is_empty() {
                        if turn.tool_names != records[i].tool_names {
                            return false;
                        }
                    }
                    true
                }) {
                    turn.matched_record_idx = Some(rec_idx);
                    used[rec_idx] = true;
                    break;
                }
            }
        }

        // Phase 2: Complete fingerprint matching (forward order-preserving matching, eliminating reverse sliding window phase misalignment after pruning)
        for turn in model_turns.iter_mut() {
            if turn.already_complete || turn.matched_record_idx.is_some() {
                continue;
            }
            if turn.fp.is_empty() {
                turn.fp = fingerprint(&turn.visible, &turn.tool_ids, &turn.tool_names);
            }
            let turn_has_tools = !turn.tool_ids.is_empty() || !turn.tool_names.is_empty();
            let Some(idxs) = by_fp.get(turn.fp.as_str()) else {
                continue;
            };
            if let Some(&rec_idx) = idxs.iter().find(|&&i| {
                if used[i] {
                    return false;
                }
                let rec_has_tools =
                    !records[i].tool_ids.is_empty() || !records[i].tool_names.is_empty();
                if rec_has_tools != turn_has_tools {
                    return false;
                }
                if !turn.tool_names.is_empty() && !records[i].tool_names.is_empty() {
                    if turn.tool_names != records[i].tool_names {
                        return false;
                    }
                }
                true
            }) {
                turn.matched_record_idx = Some(rec_idx);
                used[rec_idx] = true;
            }
        }

        // Phase 3: Plain text prefix/text similarity matching (plain text round only)
        // Normalize each record once. The old inner-loop split_whitespace().collect().join()
        // was O(turns * records * visible_len) and stalled 10s+ at ~300K context.
        let needs_phase3 = model_turns.iter().any(|t| {
            !t.already_complete
                && t.matched_record_idx.is_none()
                && t.tool_ids.is_empty()
                && t.tool_names.is_empty()
                && !t.visible.trim().is_empty()
        });
        let rec_norms: Vec<String> = if needs_phase3 {
            records.iter().map(|r| normalize_ws(&r.visible)).collect()
        } else {
            Vec::new()
        };
        if needs_phase3 {
            for turn in model_turns.iter_mut() {
                if turn.already_complete || turn.matched_record_idx.is_some() {
                    continue;
                }
                let turn_has_tools = !turn.tool_ids.is_empty() || !turn.tool_names.is_empty();
                if turn_has_tools || turn.visible.trim().is_empty() {
                    continue;
                }
                if turn.norm_visible.is_empty() {
                    turn.norm_visible = normalize_ws(&turn.visible);
                }
                if turn.norm_visible.is_empty() {
                    continue;
                }
                for (rec_idx, rec) in records.iter().enumerate() {
                    let rec_has_tools = !rec.tool_ids.is_empty() || !rec.tool_names.is_empty();
                    if used[rec_idx] || rec_has_tools || rec_norms[rec_idx].is_empty() {
                        continue;
                    }
                    let norm_rec = &rec_norms[rec_idx];
                    let norm_vis = &turn.norm_visible;
                    let thought_matches_prefix = !rec.thought.trim().is_empty()
                        && norm_vis.starts_with(&normalize_ws(&rec.thought));
                    if norm_rec == norm_vis
                        || norm_rec.starts_with(norm_vis)
                        || norm_vis.starts_with(norm_rec)
                        || (norm_rec.len() >= 10 && norm_vis.ends_with(norm_rec))
                        || (thought_matches_prefix && norm_vis.ends_with(norm_rec))
                    {
                        turn.matched_record_idx = Some(rec_idx);
                        used[rec_idx] = true;
                        break;
                    }
                }
            }
        }

        // Phase 3.5: L2 SQLite precise penetration recovery (for historical rounds that exceed the memory capacity and are eliminated or cold started)
        // Core principle: Never blindly demote placeholders in the elimination round! Prioritize precise retrieval from SQLite index through signature / tool_id / fingerprint
        for turn in model_turns.iter_mut() {
            if turn.already_complete || turn.matched_record_idx.is_some() {
                continue;
            }

            let mut fetched_rec: Option<ThinkingRecord> = None;

            // 0. Prioritize signature penetration for accurate penetration
            if let Some(ref sig) = turn.existing_sig {
                let mut found =
                    crate::modules::proxy_db::load_thinking_by_signature(store_key, sig);
                if found.as_ref().map(|o| o.is_none()).unwrap_or(true) && is_claude_signature(sig) {
                    let google_sig = ensure_google_claude_thought_signature(sig);
                    if google_sig != *sig {
                        found = crate::modules::proxy_db::load_thinking_by_signature(
                            store_key,
                            &google_sig,
                        );
                    }
                }
                if let Ok(Some(persisted)) = found {
                    let turn_has_tools = !turn.tool_ids.is_empty() || !turn.tool_names.is_empty();
                    let rec_has_tools =
                        !persisted.tool_ids.is_empty() || !persisted.tool_names.is_empty();
                    if turn_has_tools == rec_has_tools {
                        fetched_rec = Some(ThinkingRecord {
                            fingerprint: persisted.fingerprint,
                            thought: persisted.thought,
                            signature: persisted.signature,
                            tool_ids: persisted.tool_ids,
                            tool_names: persisted.tool_names,
                            visible: persisted.visible,
                        });
                    }
                }
            }

            // 1. Tool call precise penetration check (using primary_tool_id Partial Index, nanosecond level hit)
            if fetched_rec.is_none() && !turn.tool_ids.is_empty() {
                for id in &turn.tool_ids {
                    if let Ok(Some(persisted)) =
                        crate::modules::proxy_db::load_thinking_by_tool_id(store_key, id)
                    {
                        let tool_names_match =
                            if turn.tool_names.is_empty() || persisted.tool_names.is_empty() {
                                true
                            } else {
                                turn.tool_names == persisted.tool_names
                            };
                        if tool_names_match {
                            fetched_rec = Some(ThinkingRecord {
                                fingerprint: persisted.fingerprint,
                                thought: persisted.thought,
                                signature: persisted.signature,
                                tool_ids: persisted.tool_ids,
                                tool_names: persisted.tool_names,
                                visible: persisted.visible,
                            });
                            break;
                        }
                    }
                }
            } else if fetched_rec.is_none() && !turn.visible.trim().is_empty() {
                // 2. Accurate penetration and enumeration of plain text rounds (using idx_thinking_rec_fp index)
                if turn.fp.is_empty() {
                    turn.fp = fingerprint(&turn.visible, &turn.tool_ids, &turn.tool_names);
                }
                if let Ok(Some(persisted)) =
                    crate::modules::proxy_db::load_thinking_by_fingerprint(store_key, &turn.fp)
                {
                    fetched_rec = Some(ThinkingRecord {
                        fingerprint: persisted.fingerprint,
                        thought: persisted.thought,
                        signature: persisted.signature,
                        tool_ids: persisted.tool_ids,
                        tool_names: persisted.tool_names,
                        visible: persisted.visible,
                    });
                }
            }

            if let Some(rec) = fetched_rec {
                let rec_arc = Arc::new(rec);
                records.push(rec_arc.clone());
                used.push(true);
                let new_idx = records.len() - 1;
                turn.matched_record_idx = Some(new_idx);
                // Synchronously register back to the memory session entity to ensure that subsequent rounds do not require repeated enumeration.
                if let Some(mut entry) = self.sessions.get_mut(store_key) {
                    entry.bytes = entry.bytes.saturating_add(record_bytes(&rec_arc));
                    entry.turns.push(rec_arc);
                }
            }
        }

        Self::apply_restore_matches(
            store_key,
            contents,
            target_model,
            &records,
            model_turns,
            &mut used,
        )
    }
}
