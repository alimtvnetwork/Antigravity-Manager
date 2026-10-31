use rusqlite::{params, Connection, OpenFlags};

use super::*;

pub fn load_thinking_records(session_key: &str) -> Result<Vec<PersistedThinkingRecord>, String> {
    if session_key.is_empty() {
        return Ok(Vec::new());
    }
    let conn = thinking_db()?;
    let mut stmt = conn
        .prepare_cached(
            "SELECT fingerprint, thought, signature, tool_ids, tool_names, visible
             FROM thinking_records
             WHERE session_key = ?1
             ORDER BY id ASC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![session_key], |row| {
            let fp: String = row.get(0)?;
            let thought_raw: Vec<u8> = row.get(1)?;
            let signature: Option<String> = row.get(2)?;
            let tool_ids_str: String = row.get(3)?;
            let tool_names_str: String = row.get(4)?;
            let visible: String = row.get(5)?;
            Ok((
                fp,
                thought_raw,
                signature,
                tool_ids_str,
                tool_names_str,
                visible,
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for row in rows {
        if let Ok((fp, thought_raw, signature, tool_ids_str, tool_names_str, visible)) = row {
            let tool_ids: Vec<String> = serde_json::from_str(&tool_ids_str).unwrap_or_default();
            let tool_names: Vec<String> = serde_json::from_str(&tool_names_str).unwrap_or_default();
            result.push(PersistedThinkingRecord {
                fingerprint: fp,
                thought: unpack_thought(&thought_raw),
                signature: persist_signature(signature.as_deref()),
                tool_ids,
                tool_names,
                visible,
            });
        }
    }
    Ok(result)
}

/// Thoughts on accurately penetrating and enumerating history based on tool_id (causal pseudo-hash ID or native ID)
/// Adopt dual-track index for extremely fast enumeration + automatic silent self-healing mechanism for old data
pub fn load_thinking_by_tool_id(
    session_key: &str,
    tool_id: &str,
) -> Result<Option<PersistedThinkingRecord>, String> {
    if session_key.is_empty() || tool_id.is_empty() {
        return Ok(None);
    }
    let norm_id = crate::proxy::common::utils::normalize_tool_id(tool_id);
    let mut candidate_ids = vec![norm_id.as_ref()];
    if norm_id.as_ref() != tool_id {
        candidate_ids.push(tool_id);
    }

    let conn = thinking_db()?;

    for candidate in candidate_ids {
        // 1. Track 1 (Fastest Path): Prioritize the causal pseudo-hash ID idx_thinking_rec_causal exclusive local index (0.02ms nanosecond hit)
        let mut causal_stmt = conn
            .prepare_cached(
                "SELECT id, fingerprint, thought, signature, tool_ids, tool_names, visible
                 FROM thinking_records
                 WHERE session_key = ?1 AND causal_tool_id = ?2
                 ORDER BY id DESC LIMIT 1",
            )
            .map_err(|e| e.to_string())?;

        let mut causal_rows = causal_stmt
            .query(params![session_key, candidate])
            .map_err(|e| e.to_string())?;

        if let Some(row) = causal_rows.next().map_err(|e| e.to_string())? {
            let rec_id: i64 = row.get(0).map_err(|e| e.to_string())?;
            let fp: String = row.get(1).map_err(|e| e.to_string())?;
            let thought_raw: Vec<u8> = row.get(2).map_err(|e| e.to_string())?;
            let raw_signature: Option<String> = row.get(3).map_err(|e| e.to_string())?;
            let tool_ids_str: String = row.get(4).map_err(|e| e.to_string())?;
            let tool_names_str: String = row.get(5).map_err(|e| e.to_string())?;
            let visible: String = row.get(6).map_err(|e| e.to_string())?;
            let tool_ids: Vec<String> = serde_json::from_str(&tool_ids_str).unwrap_or_default();
            let tool_names: Vec<String> = serde_json::from_str(&tool_names_str).unwrap_or_default();
            let healed_sig = persist_signature(raw_signature.as_deref());

            // Reverse writeback optimization: If a damaged/non-standard signature is stored in the database, it will self-heal after the hit and write back the updated SQLite
            if let Some(ref h_sig) = healed_sig {
                if raw_signature.as_ref() != Some(h_sig) {
                    // Justification: in-place self-healing write-back is a cache repair; the healed value is returned to the caller
                    crate::error::record_ignored(
                        conn.execute(
                            "UPDATE thinking_records SET signature = ?1 WHERE id = ?2",
                            params![h_sig, rec_id],
                        ),
                        "heal thinking record signature",
                    );
                }
            }

            return Ok(Some(PersistedThinkingRecord {
                fingerprint: fp,
                thought: unpack_thought(&thought_raw),
                signature: healed_sig,
                tool_ids,
                tool_names,
                visible,
            }));
        }

        // 2. Track 2 (Legacy Path): Compatible with the old version primary_tool_id (check the idx_thinking_rec_tool index)
        let mut primary_stmt = conn
            .prepare_cached(
                "SELECT id, fingerprint, thought, signature, tool_ids, tool_names, visible
                 FROM thinking_records
                 WHERE session_key = ?1 AND primary_tool_id = ?2
                 ORDER BY id DESC LIMIT 1",
            )
            .map_err(|e| e.to_string())?;

        let mut primary_rows = primary_stmt
            .query(params![session_key, candidate])
            .map_err(|e| e.to_string())?;

        if let Some(row) = primary_rows.next().map_err(|e| e.to_string())? {
            let rec_id: i64 = row.get(0).map_err(|e| e.to_string())?;
            let fp: String = row.get(1).map_err(|e| e.to_string())?;
            let thought_raw: Vec<u8> = row.get(2).map_err(|e| e.to_string())?;
            let raw_signature: Option<String> = row.get(3).map_err(|e| e.to_string())?;
            let tool_ids_str: String = row.get(4).map_err(|e| e.to_string())?;
            let tool_names_str: String = row.get(5).map_err(|e| e.to_string())?;
            let visible: String = row.get(6).map_err(|e| e.to_string())?;
            let tool_ids: Vec<String> = serde_json::from_str(&tool_ids_str).unwrap_or_default();
            let tool_names: Vec<String> = serde_json::from_str(&tool_names_str).unwrap_or_default();
            let healed_sig = persist_signature(raw_signature.as_deref());

            // 3. Track 3 (In-Place Self-Healing): If the current request uses a causal pseudo-hash ID, the old data can be repaired silently
            if is_synthetic_tool_id(candidate) {
                // Justification: causal_tool_id backfill on a legacy record is a cache repair; the loaded record is returned regardless
                crate::error::record_ignored(
                    conn.execute(
                    "UPDATE thinking_records SET causal_tool_id = ?1 WHERE id = ?2 AND causal_tool_id IS NULL",
                    params![candidate, rec_id],
                ),
                    "backfill causal_tool_id on legacy record",
                );
            }
            if let Some(ref h_sig) = healed_sig {
                if raw_signature.as_ref() != Some(h_sig) {
                    // Justification: in-place self-healing write-back is a cache repair; the healed value is returned to the caller
                    crate::error::record_ignored(
                        conn.execute(
                            "UPDATE thinking_records SET signature = ?1 WHERE id = ?2",
                            params![h_sig, rec_id],
                        ),
                        "heal thinking record signature",
                    );
                }
            }

            return Ok(Some(PersistedThinkingRecord {
                fingerprint: fp,
                thought: unpack_thought(&thought_raw),
                signature: healed_sig,
                tool_ids,
                tool_names,
                visible,
            }));
        }

        // 4. Track 4 (Fallback Path): Compatible with the oldest record in extreme cases (fuzzy inclusion in tool_ids list)
        let pattern = format!("%\"{}\"%", candidate);
        let mut fallback_stmt = conn
            .prepare_cached(
                "SELECT id, fingerprint, thought, signature, tool_ids, tool_names, visible
                 FROM thinking_records
                 WHERE session_key = ?1 AND tool_ids LIKE ?2
                 ORDER BY id DESC LIMIT 1",
            )
            .map_err(|e| e.to_string())?;

        let mut fallback_rows = fallback_stmt
            .query(params![session_key, pattern])
            .map_err(|e| e.to_string())?;

        if let Some(row) = fallback_rows.next().map_err(|e| e.to_string())? {
            let rec_id: i64 = row.get(0).map_err(|e| e.to_string())?;
            let fp: String = row.get(1).map_err(|e| e.to_string())?;
            let thought_raw: Vec<u8> = row.get(2).map_err(|e| e.to_string())?;
            let raw_signature: Option<String> = row.get(3).map_err(|e| e.to_string())?;
            let tool_ids_str: String = row.get(4).map_err(|e| e.to_string())?;
            let tool_names_str: String = row.get(5).map_err(|e| e.to_string())?;
            let visible: String = row.get(6).map_err(|e| e.to_string())?;
            let tool_ids: Vec<String> = serde_json::from_str(&tool_ids_str).unwrap_or_default();
            let tool_names: Vec<String> = serde_json::from_str(&tool_names_str).unwrap_or_default();
            let healed_sig = persist_signature(raw_signature.as_deref());

            if is_synthetic_tool_id(candidate) {
                // Justification: causal_tool_id backfill on a legacy record is a cache repair; the loaded record is returned regardless
                crate::error::record_ignored(
                    conn.execute(
                    "UPDATE thinking_records SET causal_tool_id = ?1 WHERE id = ?2 AND causal_tool_id IS NULL",
                    params![candidate, rec_id],
                ),
                    "backfill causal_tool_id on legacy record",
                );
            }
            if let Some(ref h_sig) = healed_sig {
                if raw_signature.as_ref() != Some(h_sig) {
                    // Justification: in-place self-healing write-back is a cache repair; the healed value is returned to the caller
                    crate::error::record_ignored(
                        conn.execute(
                            "UPDATE thinking_records SET signature = ?1 WHERE id = ?2",
                            params![h_sig, rec_id],
                        ),
                        "heal thinking record signature",
                    );
                }
            }

            return Ok(Some(PersistedThinkingRecord {
                fingerprint: fp,
                thought: unpack_thought(&thought_raw),
                signature: healed_sig,
                tool_ids,
                tool_names,
                visible,
            }));
        }
    }

    Ok(None)
}

/// Accurately penetrate historical thinking based on signature (using idx_thinking_rec_sig index)
pub fn load_thinking_by_signature(
    session_key: &str,
    signature: &str,
) -> Result<Option<PersistedThinkingRecord>, String> {
    if session_key.is_empty() || signature.is_empty() {
        return Ok(None);
    }
    let conn = thinking_db()?;
    let mut stmt = conn
        .prepare_cached(
            "SELECT id, fingerprint, thought, signature, tool_ids, tool_names, visible
             FROM thinking_records
             WHERE session_key = ?1 AND signature = ?2
             ORDER BY id DESC LIMIT 1",
        )
        .map_err(|e| e.to_string())?;

    let mut rows = stmt
        .query(params![session_key, signature])
        .map_err(|e| e.to_string())?;

    if let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let rec_id: i64 = row.get(0).map_err(|e| e.to_string())?;
        let fp: String = row.get(1).map_err(|e| e.to_string())?;
        let thought_raw: Vec<u8> = row.get(2).map_err(|e| e.to_string())?;
        let raw_signature: Option<String> = row.get(3).map_err(|e| e.to_string())?;
        let tool_ids_str: String = row.get(4).map_err(|e| e.to_string())?;
        let tool_names_str: String = row.get(5).map_err(|e| e.to_string())?;
        let visible: String = row.get(6).map_err(|e| e.to_string())?;
        let tool_ids: Vec<String> = serde_json::from_str(&tool_ids_str).unwrap_or_default();
        let tool_names: Vec<String> = serde_json::from_str(&tool_names_str).unwrap_or_default();
        let healed_sig = persist_signature(raw_signature.as_deref());

        if let Some(ref h_sig) = healed_sig {
            if raw_signature.as_ref() != Some(h_sig) {
                // Justification: in-place self-healing write-back is a cache repair; the healed value is returned to the caller
                crate::error::record_ignored(
                    conn.execute(
                        "UPDATE thinking_records SET signature = ?1 WHERE id = ?2",
                        params![h_sig, rec_id],
                    ),
                    "heal thinking record signature",
                );
            }
        }

        Ok(Some(PersistedThinkingRecord {
            fingerprint: fp,
            thought: unpack_thought(&thought_raw),
            signature: healed_sig,
            tool_ids,
            tool_names,
            visible,
        }))
    } else {
        Ok(None)
    }
}

/// Provided for the UI presentation layer: Find the authoritative signature in the latest record by session (supports fault-tolerant matching with tenant prefix)
pub fn lookup_latest_thinking_signature(session_id: &str) -> Option<String> {
    if session_id.trim().is_empty() {
        return None;
    }
    let conn = thinking_db().ok()?;
    let suffix = format!("%:{}", session_id.trim());
    let (id, raw_sig): (i64, String) = conn
        .query_row(
            "SELECT id, signature FROM thinking_records 
         WHERE (session_key = ?1 OR session_key LIKE ?2) 
           AND signature IS NOT NULL 
         ORDER BY id DESC LIMIT 1",
            rusqlite::params![session_id.trim(), suffix],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .ok()?;
    let healed = normalize_and_heal_signature(&raw_sig);
    if let Some(ref h) = healed {
        if h != &raw_sig {
            // Justification: in-place self-healing write-back is a cache repair; the healed value is returned to the caller
            crate::error::record_ignored(
                conn.execute(
                    "UPDATE thinking_records SET signature = ?1 WHERE id = ?2",
                    rusqlite::params![h, id],
                ),
                "heal thinking record signature",
            );
        }
    }
    healed
}

/// Provide a bottom-up for the UI presentation layer: fuzzy search for authoritative signatures based on thought content fragments
pub fn lookup_signature_by_thought_snippet(snippet: &str) -> Option<String> {
    let clean = snippet.trim();
    if clean.is_empty() {
        return None;
    }
    let conn = thinking_db().ok()?;
    let pattern = format!("%{}%", clean);
    let (id, raw_sig): (i64, String) = conn
        .query_row(
            "SELECT id, signature FROM thinking_records 
         WHERE thought LIKE ?1 AND signature IS NOT NULL 
         ORDER BY id DESC LIMIT 1",
            rusqlite::params![pattern],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .ok()?;
    let healed = normalize_and_heal_signature(&raw_sig);
    if let Some(ref h) = healed {
        if h != &raw_sig {
            // Justification: in-place self-healing write-back is a cache repair; the healed value is returned to the caller
            crate::error::record_ignored(
                conn.execute(
                    "UPDATE thinking_records SET signature = ?1 WHERE id = ?2",
                    rusqlite::params![h, id],
                ),
                "heal thinking record signature",
            );
        }
    }
    healed
}

/// Check plain text historical thinking based on fingerprint precise penetration (using idx_thinking_rec_fp index)
pub fn load_thinking_by_fingerprint(
    session_key: &str,
    fingerprint: &str,
) -> Result<Option<PersistedThinkingRecord>, String> {
    if session_key.is_empty() || fingerprint.is_empty() {
        return Ok(None);
    }
    let conn = thinking_db()?;
    let mut stmt = conn
        .prepare_cached(
            "SELECT id, fingerprint, thought, signature, tool_ids, tool_names, visible
             FROM thinking_records
             WHERE session_key = ?1 AND fingerprint = ?2
             ORDER BY id DESC LIMIT 1",
        )
        .map_err(|e| e.to_string())?;

    let mut rows = stmt
        .query(params![session_key, fingerprint])
        .map_err(|e| e.to_string())?;

    if let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let rec_id: i64 = row.get(0).map_err(|e| e.to_string())?;
        let fp: String = row.get(1).map_err(|e| e.to_string())?;
        let thought_raw: Vec<u8> = row.get(2).map_err(|e| e.to_string())?;
        let raw_signature: Option<String> = row.get(3).map_err(|e| e.to_string())?;
        let tool_ids_str: String = row.get(4).map_err(|e| e.to_string())?;
        let tool_names_str: String = row.get(5).map_err(|e| e.to_string())?;
        let visible: String = row.get(6).map_err(|e| e.to_string())?;
        let tool_ids: Vec<String> = serde_json::from_str(&tool_ids_str).unwrap_or_default();
        let tool_names: Vec<String> = serde_json::from_str(&tool_names_str).unwrap_or_default();
        let healed_sig = persist_signature(raw_signature.as_deref());

        if let Some(ref h_sig) = healed_sig {
            if raw_signature.as_ref() != Some(h_sig) {
                // Justification: in-place self-healing write-back is a cache repair; the healed value is returned to the caller
                crate::error::record_ignored(
                    conn.execute(
                        "UPDATE thinking_records SET signature = ?1 WHERE id = ?2",
                        params![h_sig, rec_id],
                    ),
                    "heal thinking record signature",
                );
            }
        }

        Ok(Some(PersistedThinkingRecord {
            fingerprint: fp,
            thought: unpack_thought(&thought_raw),
            signature: healed_sig,
            tool_ids,
            tool_names,
            visible,
        }))
    } else {
        Ok(None)
    }
}
