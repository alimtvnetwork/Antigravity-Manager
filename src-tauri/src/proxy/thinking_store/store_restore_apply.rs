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
        // Phase 4: 尾部优先的逆向兜底匹配（仅限纯文本轮次，绝不跨轮借用工具签名造成下轮突变！）
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

                // [DE-DUPLICATION & ELEVATION] 正文残留思考文本切除与去重：
                // 解决关思考时降级到正文的思考文本，在重新开思考时被再次提升复活后，导致正文残留双份复读的问题！
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

                        // 0. 若正文带有 <think>...</think> 标签，精准切除标签与思考内容，保留剩余真实正文
                        if let Some((_, rem)) = extract_think_tags(text) {
                            if !rem.is_empty() {
                                cleaned_parts.push(json!({ "text": rem }));
                            }
                            continue;
                        }

                        // 1. 完全相同（之前作为独立降级部件存在）：直接丢弃该部件
                        if text_trimmed == rec_thought_trimmed
                            || (!rec_thought_norm.is_empty()
                                && normalize_ws(text) == rec_thought_norm)
                        {
                            continue;
                        }
                        // 2. 正文以思考文本开头（思考文本与正文被客户端合并为一个部件）：切除前缀
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
                    // Claude 模型：上游对接 Anthropic 官方验签引擎！
                    // Anthropic 官方规范：签名必须且只能在思考块 (thinking block) 上 (映射为 messages[x].content[0].signature)！
                    // 工具调用 (tool_use / functionCall) 绝不携带签名，亦绝对不可注入假哨兵！
                    if let Some(sig) = rec.signature.as_ref().filter(|s| is_real_signature(s)) {
                        // 关键门禁：只有当历史签名确属 Claude 签名时，才挂载到 thought_part！
                        // 若是 Gemini 等异构模型生成的签名，绝对禁止注入给 Claude，避免 400 Invalid signature
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
                    // Gemini 原生：把本轮捕获到的真实签名归位到「该轮第一个非思考 part」。
                    //
                    // 首位思考块保持纯净思考文本（铁律 I4：思考块绝不携带签名），随后由
                    // `parts.insert(0, thought_part)` 插入 index 0 —— 锚点自然落到 index 1，
                    // 正好复现官方的「思考块 + 带签名正文」排列。
                    let fallback = rec
                        .signature
                        .as_deref()
                        .filter(|s| is_real_signature(s) && is_likely_gemini_signature(s));
                    place_turn_signature(&mut *parts, fallback);
                }
                parts.insert(0, thought_part);
                restored += 1;
            } else {
                // 【2026-09-27】无实质思考内容（占位/空思考）：绝不注入 "..." 占位思考块！
                // 官方标准形态为「无思考块 + 锚点带签名」（baogao.txt 9/24 轮）。
                // 首个非思考 part 作为锚点正常承接该轮签名。
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
