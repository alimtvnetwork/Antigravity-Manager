use super::*;
use dashmap::DashMap;
use serde_json::Value;
use std::sync::{Arc, OnceLock};
use std::time::Instant;

#[derive(Debug)]
pub(crate) struct SessionEntry {
    pub(crate) turns: Vec<Arc<ThinkingRecord>>,
    pub(crate) last_access: Instant,
    pub(crate) last_persist_touch: Instant,
    pub(crate) bytes: usize,
    pub(crate) l2_loaded: bool,
}

impl SessionEntry {
    fn new() -> Self {
        Self {
            turns: Vec::new(),
            last_access: Instant::now(),
            last_persist_touch: Instant::now(),
            bytes: 0,
            l2_loaded: false,
        }
    }
}
pub struct ThinkingStore {
    pub(crate) sessions: DashMap<String, SessionEntry>,
}

impl ThinkingStore {
    pub(crate) fn new() -> Self {
        Self {
            sessions: DashMap::new(),
        }
    }

    pub fn global() -> &'static ThinkingStore {
        static INSTANCE: OnceLock<ThinkingStore> = OnceLock::new();
        INSTANCE.get_or_init(ThinkingStore::new)
    }

    fn maybe_evict(&self, keep_key: &str) {
        if self.sessions.len() <= MAX_SESSIONS {
            return;
        }
        self.sessions
            .retain(|_, e| e.last_access.elapsed() < idle_ttl());
        if self.sessions.len() <= MAX_SESSIONS {
            return;
        }
        if let Some(oldest_key) = self
            .sessions
            .iter()
            .min_by_key(|e| e.last_access)
            .map(|e| e.key().clone())
        {
            if oldest_key != keep_key {
                self.sessions.remove(&oldest_key);
            }
        }
    }
    pub fn record(&self, store_key: &str, rec: ThinkingRecord) {
        if !crate::proxy::config::is_thinking_store_enabled() {
            return;
        }
        if rec.thought.trim().is_empty() && rec.signature.is_none() {
            return;
        }
        // Placeholder "..." / sentinel-only blocks are injected for Gemini protocol
        // compliance. Recording them as new turns made every 300K-context request
        // append N dummies, then prune rewrite the whole SQLite session.
        if !is_capturable_thought(&rec.thought, rec.signature.as_deref()) {
            return;
        }

        let rec_bytes = record_bytes(&rec);

        self.maybe_evict(store_key);

        // Always hydrate L2 before appending. Otherwise the first capture after a
        // process start can mark l2_loaded=true with only the new turn and permanently
        // shadow older SQLite history on subsequent hydrate/restore calls.
        let needs_l2 = self
            .sessions
            .get(store_key)
            .map(|e| !e.l2_loaded)
            .unwrap_or(true);
        if needs_l2 {
            // Justification: load_turns hydrates the in-memory session as a side effect; the returned Vec is intentionally unused here.
            let _ = self.load_turns(store_key);
        }

        let persist = {
            let mut entry = self
                .sessions
                .entry(store_key.to_string())
                .or_insert_with(SessionEntry::new);
            entry.last_access = Instant::now();
            entry.l2_loaded = true;

            let merge_last = entry
                .turns
                .last()
                .is_some_and(|last| last.fingerprint == rec.fingerprint);

            if merge_last {
                let (stronger, old_text_bytes) = {
                    let last = entry.turns.last().expect("merge_last");
                    (
                        rec.thought.len() >= last.thought.len()
                            || rec.signature.as_ref().map(|s| s.len()).unwrap_or(0)
                                > last.signature.as_ref().map(|s| s.len()).unwrap_or(0),
                        last.thought.len() + last.visible.len(),
                    )
                };
                if stronger {
                    entry.bytes = entry.bytes.saturating_sub(old_text_bytes);
                    {
                        let last_arc = entry.turns.last_mut().expect("merge_last");
                        *Arc::make_mut(last_arc) = rec;
                    }
                    entry.bytes = entry.bytes.saturating_add(rec_bytes);
                    entry.turns.last().cloned()
                } else {
                    None
                }
            } else {
                entry.turns.push(Arc::new(rec));
                entry.bytes = entry.bytes.saturating_add(rec_bytes);
                while entry.turns.len() > max_turns_per_session()
                    || entry.bytes > MAX_BYTES_PER_SESSION
                {
                    if let Some(old) = entry.turns.first() {
                        let old_bytes = old.thought.len()
                            + old.signature.as_ref().map(|s| s.len()).unwrap_or(0)
                            + old.visible.len();
                        entry.bytes = entry.bytes.saturating_sub(old_bytes);
                    }
                    if entry.turns.is_empty() {
                        break;
                    }
                    entry.turns.remove(0);
                }
                entry.turns.last().cloned()
            }
        };

        if let Some(saved) = persist {
            // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
            crate::error::record_ignored(
                crate::modules::proxy_db::save_thinking_record(
                    store_key,
                    &saved.fingerprint,
                    &saved.thought,
                    saved.signature.as_deref(),
                    &saved.tool_ids,
                    &saved.tool_names,
                    &saved.visible,
                ),
                "save_thinking_record",
            );
        }
    }

    /// Refresh in-memory expiry. SQLite last_accessed is debounced so HDD
    /// never sees a write on the fill hot path after the session is warm.
    pub fn touch_session(&self, store_key: &str) {
        if !crate::proxy::config::is_thinking_store_enabled() || store_key.is_empty() {
            return;
        }
        let mut persist = false;
        if let Some(mut entry) = self.sessions.get_mut(store_key) {
            entry.last_access = Instant::now();
            if entry.last_persist_touch.elapsed() >= TOUCH_PERSIST_INTERVAL {
                entry.last_persist_touch = Instant::now();
                persist = true;
            }
        }
        if persist {
            // Justification: best-effort call; failure logged without changing control flow
            crate::error::record_ignored(
                crate::modules::proxy_db::touch_thinking_session(store_key),
                "touch_thinking_session",
            );
        }
    }

    pub(crate) fn load_turns(&self, store_key: &str) -> Vec<Arc<ThinkingRecord>> {
        if let Some(e) = self.sessions.get(store_key) {
            // Trust warm non-empty memory. An empty l2_loaded entry is treated as
            // stale (e.g. first hydrate before any capture) and reloads from SQLite.
            if e.l2_loaded && !e.turns.is_empty() {
                let turns = e.turns.clone();
                drop(e);
                if let Some(mut entry) = self.sessions.get_mut(store_key) {
                    entry.last_access = Instant::now();
                }
                return turns;
            }
        }

        let persisted =
            crate::modules::proxy_db::load_thinking_records(store_key).unwrap_or_default();
        let loaded_len = persisted.len();
        let mut entry = self
            .sessions
            .entry(store_key.to_string())
            .or_insert_with(SessionEntry::new);
        if entry.turns.is_empty() && !persisted.is_empty() {
            for p in persisted {
                let rec = ThinkingRecord {
                    fingerprint: p.fingerprint,
                    thought: p.thought,
                    signature: p.signature,
                    tool_ids: p.tool_ids,
                    tool_names: p.tool_names,
                    visible: p.visible,
                };
                entry.bytes += record_bytes(&rec);
                entry.turns.push(Arc::new(rec));
            }
            tracing::info!(
                "[ThinkingStore] Restored {} turns from SQLite L2 for session {}",
                entry.turns.len(),
                store_key
            );
        }
        entry.l2_loaded = true;
        entry.last_access = Instant::now();
        entry.last_persist_touch = Instant::now();
        let turns = entry.turns.clone();
        drop(entry);
        if loaded_len > 0 {
            // Justification: best-effort call; failure logged without changing control flow
            crate::error::record_ignored(
                crate::modules::proxy_db::touch_thinking_session(store_key),
                "touch_thinking_session",
            );
        }
        turns
    }

    /// Capture real thinking from the inbound request without re-appending
    /// history that is already stored. Placeholder blocks are ignored.
    pub fn ingest_from_contents(&self, store_key: &str, contents: &[Value]) {
        if !crate::proxy::config::is_thinking_store_enabled() || store_key.is_empty() {
            return;
        }

        let mut incoming: Vec<ThinkingRecord> = Vec::new();
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
            let mut acc = TurnAccumulator::with_anchor(&anchor);
            for part in parts {
                acc.ingest_part(part);
            }
            if !acc.should_capture() {
                continue;
            }
            incoming.push(acc.into_record());
        }
        if incoming.is_empty() {
            return;
        }

        let existing = self.load_turns(store_key);
        let mut used = vec![false; existing.len()];
        let mut to_append = Vec::new();
        let mut to_upgrade: Vec<(usize, ThinkingRecord)> = Vec::new();

        for rec in incoming {
            if let Some(idx) = match_existing_record(&rec, &existing, &used) {
                used[idx] = true;
                if is_stronger_record(&rec, &existing[idx]) {
                    to_upgrade.push((idx, rec));
                }
            } else {
                to_append.push(rec);
            }
        }

        if !to_upgrade.is_empty() {
            if let Some(mut entry) = self.sessions.get_mut(store_key) {
                for (idx, rec) in &to_upgrade {
                    if *idx >= entry.turns.len() {
                        continue;
                    }
                    let new_bytes = record_bytes(rec);
                    let old_bytes = record_bytes(&entry.turns[*idx]);
                    entry.bytes = entry
                        .bytes
                        .saturating_sub(old_bytes)
                        .saturating_add(new_bytes);
                    *Arc::make_mut(&mut entry.turns[*idx]) = rec.clone();
                }
            }
            if let Some((idx, rec)) = to_upgrade.last() {
                if *idx + 1 == existing.len() {
                    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
                    crate::error::record_ignored(
                        crate::modules::proxy_db::save_thinking_record(
                            store_key,
                            &rec.fingerprint,
                            &rec.thought,
                            rec.signature.as_deref(),
                            &rec.tool_ids,
                            &rec.tool_names,
                            &rec.visible,
                        ),
                        "save_thinking_record",
                    );
                }
            }
        }

        for rec in to_append {
            self.record(store_key, rec);
        }
    }
}
fn is_stronger_record(new: &ThinkingRecord, old: &ThinkingRecord) -> bool {
    new.thought.len() > old.thought.len()
        || new.signature.as_ref().map(|s| s.len()).unwrap_or(0)
            > old.signature.as_ref().map(|s| s.len()).unwrap_or(0)
}
fn match_existing_record(
    rec: &ThinkingRecord,
    existing: &[Arc<ThinkingRecord>],
    used: &[bool],
) -> Option<usize> {
    if let Some(ref sig) = rec.signature.as_ref().filter(|s| is_real_signature(s)) {
        for (i, ex) in existing.iter().enumerate() {
            if used[i] {
                continue;
            }
            if let Some(ref ex_sig) = ex.signature {
                if signatures_match(sig, ex_sig) {
                    return Some(i);
                }
            }
        }
    }
    if !rec.tool_ids.is_empty() {
        for (i, ex) in existing.iter().enumerate() {
            if used[i] {
                continue;
            }
            if rec
                .tool_ids
                .iter()
                .any(|id| ex.tool_ids.iter().any(|x| x == id))
            {
                return Some(i);
            }
        }
    }
    let rec_has_tools = !rec.tool_ids.is_empty() || !rec.tool_names.is_empty();
    for (i, ex) in existing.iter().enumerate() {
        if used[i] {
            continue;
        }
        if ex.fingerprint != rec.fingerprint {
            continue;
        }
        let ex_has_tools = !ex.tool_ids.is_empty() || !ex.tool_names.is_empty();
        if rec_has_tools == ex_has_tools {
            return Some(i);
        }
    }
    None
}
