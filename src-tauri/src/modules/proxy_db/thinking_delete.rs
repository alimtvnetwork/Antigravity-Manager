use rusqlite::{params, Connection, OpenFlags};

use super::*;

pub fn touch_thinking_session(session_key: &str) -> Result<usize, String> {
    if session_key.is_empty() {
        return Ok(0);
    }
    let conn = thinking_db()?;
    let now = chrono::Utc::now().timestamp_millis();
    // Touch a 1-row session table. Never UPDATE thinking_records here — that
    // rewrites every thought/visible TEXT blob for the session.
    conn.execute(
        "INSERT INTO thinking_sessions (session_key, last_accessed) VALUES (?1, ?2)
         ON CONFLICT(session_key) DO UPDATE SET last_accessed = excluded.last_accessed",
        params![session_key, now],
    )
    .map_err(|e| e.to_string())
}

pub fn delete_thinking_records_except_fingerprints(
    session_key: &str,
    keep_fps: &[String],
) -> Result<usize, String> {
    if session_key.is_empty() || keep_fps.is_empty() {
        return Ok(0);
    }
    let conn = thinking_db()?;
    let fps_json = serde_json::to_string(keep_fps).unwrap_or_else(|_| "[]".to_string());
    conn.execute(
        "DELETE FROM thinking_records
         WHERE session_key = ?1
         AND fingerprint NOT IN (SELECT value FROM json_each(?2))",
        params![session_key, fps_json],
    )
    .map_err(|e| e.to_string())
}

pub fn delete_thinking_records_for_session(session_key: &str) -> Result<usize, String> {
    let conn = thinking_db()?;
    // Justification: session-row cleanup is auxiliary; the records delete is the authoritative op
    crate::error::record_ignored(
        conn.execute(
            "DELETE FROM thinking_sessions WHERE session_key = ?1",
            params![session_key],
        ),
        "delete thinking session row",
    );
    conn.execute(
        "DELETE FROM thinking_records WHERE session_key = ?1",
        params![session_key],
    )
    .map_err(|e| e.to_string())
}

/// 精准净化思考记录表中的非法异构签名（保留思考文本与其它健康签名）
pub fn purge_foreign_signatures_for_session_with_model(
    session_key: &str,
    target_model: &str,
) -> Result<usize, String> {
    let is_gemini = target_model.to_lowercase().contains("gemini");
    let is_claude = target_model.to_lowercase().contains("claude");
    if (!is_gemini && !is_claude) || session_key.is_empty() {
        return Ok(0);
    }

    let conn = thinking_db()?;
    let mut stmt = conn
        .prepare_cached("SELECT id, signature FROM thinking_records WHERE session_key = ?1 AND signature IS NOT NULL")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![session_key], |row| {
            let id: i64 = row.get(0)?;
            let sig: String = row.get(1)?;
            Ok((id, sig))
        })
        .map_err(|e| e.to_string())?;

    let mut ids_to_null = Vec::new();
    for row in rows.flatten() {
        let (id, sig) = row;
        let is_foreign = if is_gemini {
            !crate::proxy::thinking_store::is_likely_gemini_signature(&sig)
        } else if is_claude {
            !crate::proxy::thinking_store::is_claude_signature(&sig)
        } else {
            false
        };
        if is_foreign {
            ids_to_null.push(id);
        }
    }

    let mut total_updated = 0;
    if !ids_to_null.is_empty() {
        let mut update_stmt = conn
            .prepare_cached("UPDATE thinking_records SET signature = NULL WHERE id = ?1")
            .map_err(|e| e.to_string())?;
        for id in ids_to_null {
            if let Ok(n) = update_stmt.execute(params![id]) {
                total_updated += n;
            }
        }
    }

    Ok(total_updated)
}

/// 兼容旧接口：默认按 Gemini 清洗
pub fn purge_foreign_signatures_for_session(session_key: &str) -> Result<usize, String> {
    purge_foreign_signatures_for_session_with_model(session_key, "gemini")
}

/// 全量清空思考块数据库 (仅清空 thinking_records / thinking_sessions / tool_signatures，绝不触碰 request_logs 日志)
pub fn clear_all_thinking_data() -> Result<usize, String> {
    let mut total_deleted = 0;
    // 1. 清空 thinking_store.db 中的记录与会话
    let conn = thinking_db()?;
    let deleted = conn
        .execute("DELETE FROM thinking_records", [])
        .map_err(|e| e.to_string())?;
    total_deleted += deleted;
    // Justification: cascade cleanup after the thinking records were deleted
    crate::error::record_ignored(
        conn.execute("DELETE FROM thinking_sessions", []),
        "delete thinking sessions during full clear",
    );
    // Justification: space reclamation after a full clear; the data is already gone
    crate::error::record_ignored(
        conn.execute("VACUUM", []),
        "vacuum thinking database after full clear",
    );

    // 2. 清空 proxy_logs.db 中残留的历史工具签名表与陈旧思考表 (绝不触碰 request_logs)
    if let Ok(log_conn) = connect_db() {
        // Justification: residual cleanup in the legacy logs database
        crate::error::record_ignored(
            log_conn.execute("DELETE FROM tool_signatures", []),
            "delete legacy tool signatures",
        );
        // Justification: residual cleanup in the legacy logs database
        crate::error::record_ignored(
            log_conn.execute("DELETE FROM thinking_records", []),
            "delete legacy thinking records",
        );
        // Justification: residual cleanup in the legacy logs database
        crate::error::record_ignored(
            log_conn.execute("DELETE FROM thinking_sessions", []),
            "delete legacy thinking sessions",
        );
    }

    Ok(total_deleted)
}
