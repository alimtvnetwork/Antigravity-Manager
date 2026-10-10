use rusqlite::{params, Connection, OpenFlags};

use super::*;

#[derive(Debug, Clone)]
pub struct PersistedThinkingRecord {
    pub fingerprint: String,
    pub thought: String,
    pub signature: Option<String>,
    pub tool_ids: Vec<String>,
    pub tool_names: Vec<String>,
    pub visible: String,
}

pub fn save_thinking_record(
    session_key: &str,
    fingerprint: &str,
    thought: &str,
    signature: Option<&str>,
    tool_ids: &[String],
    _tool_names: &[String],
    visible: &str,
) -> Result<(), String> {
    if session_key.is_empty() {
        return Ok(());
    }
    let conn = thinking_db()?;
    let now = chrono::Utc::now().timestamp_millis();
    let normalized_tool_ids: Vec<String> = tool_ids
        .iter()
        .map(|id| crate::proxy::common::utils::normalize_tool_id(id).into_owned())
        .collect();
    let tool_ids_json =
        serde_json::to_string(&normalized_tool_ids).unwrap_or_else(|_| "[]".to_string());
    let causal_tool_id = normalized_tool_ids
        .iter()
        .find(|id| is_synthetic_tool_id(id))
        .map(|s| s.as_str());
    let primary_tool_id = normalized_tool_ids.first().map(|s| s.as_str());
    // tool_names / full visible for tool turns are reconstructable from the next
    // request JSON at fill time. Do not write them.
    let visible_persist = persist_visible(&normalized_tool_ids, visible);
    let packed_thought = pack_thought(thought);
    let signature = persist_signature(signature);

    // 智能防叠加与幂等查重：只允许合并/更新当前会话中的【最新一条】活跃轮次（流式碎片拼接或更长思考补齐）
    // 绝不能回溯更新历史早期轮次！
    let latest_row: Option<(
        i64,
        usize,
        Option<String>,
        String,
        Option<String>,
        Option<String>,
    )> = conn
        .query_row(
            "SELECT id, length(thought), signature, fingerprint, primary_tool_id, causal_tool_id
             FROM thinking_records
             WHERE session_key = ?1
             ORDER BY id DESC LIMIT 1",
            params![session_key],
            |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                ))
            },
        )
        .ok();

    let existing_id: Option<(i64, usize, Option<String>)> = match latest_row {
        Some((id, len, sig, ref last_fp, ref last_tool_id, ref last_causal_id)) => {
            let is_match = if let Some(c_id) = causal_tool_id {
                last_causal_id.as_deref() == Some(c_id)
                    || last_tool_id.as_deref() == Some(c_id)
                    || last_causal_id
                        .as_deref()
                        .map(|s| crate::proxy::common::utils::normalize_tool_id(s))
                        .as_deref()
                        == Some(c_id)
                    || last_tool_id
                        .as_deref()
                        .map(|s| crate::proxy::common::utils::normalize_tool_id(s))
                        .as_deref()
                        == Some(c_id)
            } else if let Some(p_id) = primary_tool_id {
                last_tool_id.as_deref() == Some(p_id)
                    || last_tool_id
                        .as_deref()
                        .map(|s| crate::proxy::common::utils::normalize_tool_id(s))
                        .as_deref()
                        == Some(p_id)
            } else {
                last_fp == fingerprint && last_tool_id.is_none() && last_causal_id.is_none()
            };
            if is_match {
                Some((id, len, sig))
            } else {
                None
            }
        }
        None => None,
    };

    if let Some((id, old_thought_len, old_sig)) = existing_id {
        // 已存在记录：检查是否需要更新（防止将已有实质思考覆盖为占位符，但允许补全更长思考或有效签名）
        let incoming_has_meaningful_thought =
            !crate::proxy::thinking_store::is_placeholder_thought(thought)
                && !thought.trim().is_empty();
        let old_is_dummy = old_thought_len <= 10; // "RAW1..." 或占位符非常短

        let should_update_thought = incoming_has_meaningful_thought || old_is_dummy;
        let healed_old_sig = old_sig.as_deref().and_then(normalize_and_heal_signature);
        let effective_sig = signature.as_deref().or(healed_old_sig.as_deref());

        if should_update_thought {
            let mut stmt = conn
                .prepare_cached(
                    "UPDATE thinking_records
                     SET thought = ?1, signature = ?2, tool_ids = ?3, visible = ?4, created_at = ?5, primary_tool_id = ?6, causal_tool_id = ?7
                     WHERE id = ?8",
                )
                .map_err(|e| e.to_string())?;
            stmt.execute(params![
                packed_thought.as_slice(),
                effective_sig,
                &tool_ids_json,
                visible_persist,
                now,
                primary_tool_id,
                causal_tool_id,
                id,
            ])
            .map_err(|e| e.to_string())?;
        } else if (signature.is_some() && signature.as_deref() != old_sig.as_deref())
            || (healed_old_sig.as_deref() != old_sig.as_deref())
        {
            // 仅更新签名，保留已有的高质量实质思考（同时修复旧签名的脏数据）
            let mut stmt = conn
                .prepare_cached(
                    "UPDATE thinking_records
                     SET signature = ?1, created_at = ?2
                     WHERE id = ?3",
                )
                .map_err(|e| e.to_string())?;
            stmt.execute(params![effective_sig, now, id])
                .map_err(|e| e.to_string())?;
        }
    } else {
        // 全新轮次：插入新记录（同时写入 primary_tool_id 与 causal_tool_id 列）
        let mut stmt = conn
            .prepare_cached(
                "INSERT INTO thinking_records (session_key, fingerprint, thought, signature, tool_ids, tool_names, visible, created_at, primary_tool_id, causal_tool_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, '[]', ?6, ?7, ?8, ?9)",
            )
            .map_err(|e| e.to_string())?;
        stmt.execute(params![
            session_key,
            fingerprint,
            packed_thought.as_slice(),
            signature.as_deref(),
            &tool_ids_json,
            visible_persist,
            now,
            primary_tool_id,
            causal_tool_id,
        ])
        .map_err(|e| e.to_string())?;
    }

    let mut session_stmt = conn
        .prepare_cached(
            "INSERT INTO thinking_sessions (session_key, last_accessed) VALUES (?1, ?2)
             ON CONFLICT(session_key) DO UPDATE SET last_accessed = excluded.last_accessed",
        )
        .map_err(|e| e.to_string())?;
    // Justification: auxiliary session touch-up; the thinking record itself was already persisted
    crate::error::record_ignored(
        session_stmt.execute(params![session_key, now]),
        "touch thinking session last_accessed",
    );

    Ok(())
}
