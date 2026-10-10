use super::*;
use serde_json::Value;
use std::sync::Arc;

impl ThinkingStore {
    pub fn end_session(&self, store_key: &str) -> EndSessionResult {
        let removed = self.sessions.remove(store_key);
        let (deleted_turns, deleted_bytes) = removed
            .map(|(_, e)| (e.turns.len(), e.bytes))
            .unwrap_or((0, 0));
        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
        crate::error::record_ignored(
            crate::modules::proxy_db::delete_thinking_records_for_session(store_key),
            "delete_thinking_records_for_session",
        );
        EndSessionResult {
            session_id: client_id_from_store_key(store_key).to_string(),
            deleted_turns,
            deleted_bytes,
        }
    }

    /// 精准定向净化指定会话中的异构污染签名（保留思考文本与健康签名）
    pub fn purge_corrupted_signatures(&self, store_key: &str, target_model: &str) -> usize {
        if store_key.is_empty() {
            return 0;
        }
        let is_gemini = target_model.to_lowercase().contains("gemini");
        let is_claude = target_model.to_lowercase().contains("claude");
        if !is_gemini && !is_claude {
            return 0;
        }

        let mut purged_count = 0;

        // 1. 精准净化内存缓存 (RAM)
        if let Some(mut entry) = self.sessions.get_mut(store_key) {
            let mut new_turns = Vec::with_capacity(entry.turns.len());
            for rec in &entry.turns {
                if let Some(ref sig) = rec.signature {
                    let is_foreign = if is_gemini {
                        !is_likely_gemini_signature(sig)
                    } else if is_claude {
                        !is_claude_signature(sig)
                    } else {
                        false
                    };
                    if is_foreign {
                        purged_count += 1;
                        let mut cleaned = (**rec).clone();
                        cleaned.signature = None;
                        new_turns.push(Arc::new(cleaned));
                        continue;
                    }
                }
                new_turns.push(rec.clone());
            }
            entry.turns = new_turns;
        }

        // 2. 精准净化持久化数据库 (SQLite)
        // Justification: best-effort call; failure logged without changing control flow
        crate::error::record_ignored(
            crate::modules::proxy_db::purge_foreign_signatures_for_session_with_model(
                store_key,
                target_model,
            ),
            "purge_foreign_signatures_for_session_with_model",
        );

        if purged_count > 0 {
            tracing::warn!(
                "[ThinkingStore] Surgically purged {} foreign signature(s) for session {} targeting {}",
                purged_count, store_key, target_model
            );
        }
        purged_count
    }

    /// Drop thinking records that no longer appear in the (possibly compressed) history.
    /// Always keeps the newest 2 turns so the latest unused response thinking is not lost.
    pub fn prune_orphaned_records(&self, store_key: &str, contents: &[Value]) {
        if !crate::proxy::config::is_thinking_store_enabled() || store_key.is_empty() {
            return;
        }

        let mem_turns = self
            .sessions
            .get(store_key)
            .map(|e| e.turns.len())
            .unwrap_or(0);
        if mem_turns == 0 {
            return;
        }
        let live_turn_count = contents.iter().filter(|c| is_model_or_assistant(c)).count();
        // Typical agent path: stored turns ≈ live model turns. Skip inspect/fingerprint/SQLite.
        if mem_turns <= live_turn_count.saturating_add(2) {
            return;
        }

        let mut live_tool_ids = std::collections::HashSet::new();
        let mut live_fps = std::collections::HashSet::new();
        let mut live_sigs = std::collections::HashSet::new();
        let mut live_visibles: Vec<String> = Vec::new();

        for (c_idx, content) in contents.iter().enumerate() {
            if !is_model_or_assistant(content) {
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
            let (visible, tool_ids, tool_names, _) = inspect_parts_with_anchor(parts, &anchor);
            live_fps.insert(fingerprint(&visible, &tool_ids, &tool_names));
            for id in tool_ids {
                live_tool_ids.insert(id);
            }
            for part in parts {
                if let Some(sig) = part
                    .get("thoughtSignature")
                    .or_else(|| part.get("thought_signature"))
                    .and_then(|s| s.as_str())
                    .filter(|s| is_real_signature(s))
                {
                    live_sigs.insert(sig.to_string());
                }
            }
            let norm = normalize_ws(&visible);
            if !norm.is_empty() {
                live_visibles.push(norm);
            }
        }

        let keep_fps = {
            let Some(mut entry) = self.sessions.get_mut(store_key) else {
                return;
            };
            if entry.turns.len() <= live_turn_count.saturating_add(2) {
                return;
            }

            let rec_norms: Vec<String> = entry
                .turns
                .iter()
                .map(|r| normalize_ws(&r.visible))
                .collect();
            let total = entry.turns.len();
            let keep_tail_start = total.saturating_sub(2);
            let mut keep: Vec<Arc<ThinkingRecord>> = Vec::new();
            for (i, rec) in entry.turns.iter().enumerate() {
                let matched_sig = rec
                    .signature
                    .as_deref()
                    .is_some_and(|s| live_sigs.contains(s));
                let matched_tool = rec.tool_ids.iter().any(|id| live_tool_ids.contains(id));
                let matched_fp = live_fps.contains(&rec.fingerprint);
                let norm_rec = &rec_norms[i];
                let matched_text = !norm_rec.is_empty()
                    && live_visibles.iter().any(|v| {
                        v == norm_rec || v.starts_with(norm_rec) || norm_rec.starts_with(v)
                    });
                if matched_sig || matched_tool || matched_fp || matched_text || i >= keep_tail_start
                {
                    keep.push(rec.clone());
                }
            }

            if keep.len() == entry.turns.len() {
                return;
            }

            let dropped = entry.turns.len() - keep.len();
            entry.bytes = keep.iter().map(|r| record_bytes(r)).sum();
            let fps: Vec<String> = keep.iter().map(|r| r.fingerprint.clone()).collect();
            entry.turns = keep;
            tracing::info!(
                "[ThinkingStore] Pruned {} orphaned thinking record(s) after context compression for session {}",
                dropped,
                store_key
            );
            fps
        };

        // Delete orphans by fingerprint. Never DELETE+re-INSERT the kept blobs.
        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
        crate::error::record_ignored(
            crate::modules::proxy_db::delete_thinking_records_except_fingerprints(
                store_key, &keep_fps,
            ),
            "delete_thinking_records_except_fingerprints",
        );
    }

    pub fn session_stats(&self, store_key: &str) -> Option<(usize, usize)> {
        self.sessions
            .get(store_key)
            .map(|e| (e.turns.len(), e.bytes))
    }

    pub fn clear(&self) {
        self.sessions.clear();
    }
}
