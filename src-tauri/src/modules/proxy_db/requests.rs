use crate::proxy::monitor::ProxyRequestLog;
use rusqlite::{params, Connection, OpenFlags};
use std::sync::{Mutex, MutexGuard, OnceLock};

use super::*;

pub(crate) fn map_request_log_row(row: &rusqlite::Row) -> rusqlite::Result<ProxyRequestLog> {
    Ok(ProxyRequestLog {
        id: row.get(0)?,
        timestamp: row.get(1)?,
        method: row.get(2)?,
        url: row.get(3)?,
        status: row.get(4)?,
        duration: row.get(5)?,
        model: row.get(6)?,
        error: row.get(7)?,
        request_body: row.get(8).unwrap_or(None),
        upstream_request_body: row.get(9).unwrap_or(None),
        response_body: row.get(10).unwrap_or(None),
        input_tokens: row.get(11).unwrap_or(None),
        output_tokens: row.get(12).unwrap_or(None),
        cached_tokens: row.get(13).unwrap_or(None),
        account_email: row.get(14).unwrap_or(None),
        mapped_model: row.get(15).unwrap_or(None),
        protocol: row.get(16).unwrap_or(None),
        client_ip: row.get(17).unwrap_or(None),
        username: row.get(18).unwrap_or(None),
        request_headers: row.get(19).unwrap_or(None),
        upstream_request_headers: row.get(20).unwrap_or(None),
        response_headers: row.get(21).unwrap_or(None),
        session_id: row.get(22).unwrap_or(None),
    })
}

pub fn save_tool_signature(tool_id: &str, signature: &str) -> Result<(), String> {
    if tool_id.is_empty() || signature.is_empty() {
        return Ok(());
    }
    let norm_id = crate::proxy::common::utils::normalize_tool_id(tool_id);
    let healed_sig = match normalize_and_heal_signature(signature) {
        Some(s) => s,
        None => return Ok(()),
    };
    let conn = connect_db()?;
    let now = chrono::Utc::now().timestamp_millis();
    conn.execute(
        "INSERT OR REPLACE INTO tool_signatures (tool_id, signature, created_at) VALUES (?1, ?2, ?3)",
        params![norm_id.as_ref(), healed_sig, now],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn load_tool_signature(tool_id: &str) -> Result<Option<String>, String> {
    if tool_id.is_empty() {
        return Ok(None);
    }
    let norm_id = crate::proxy::common::utils::normalize_tool_id(tool_id);
    let db_path = get_proxy_db_path()?;
    let found = {
        let mut db = TOOL_SIGNATURE_DB
            .get_or_init(|| Mutex::new(None))
            .lock()
            .map_err(|e| format!("tool signature db lock: {e}"))?;
        if db.as_ref().map(|(path, _)| path) != Some(&db_path) {
            let conn = Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
                .map_err(|e| e.to_string())?;
            conn.busy_timeout(std::time::Duration::from_secs(5))
                .map_err(|e| e.to_string())?;
            *db = Some((db_path, conn));
        }
        let conn = &db
            .as_ref()
            .ok_or("tool signature db was not initialized")?
            .1;
        let mut stmt = conn
            .prepare_cached("SELECT signature FROM tool_signatures WHERE tool_id = ?1 LIMIT 1")
            .map_err(|e| e.to_string())?;
        let res: Option<String> = {
            let mut rows = stmt
                .query(params![norm_id.as_ref()])
                .map_err(|e| e.to_string())?;
            if let Some(row) = rows.next().map_err(|e| e.to_string())? {
                let sig: String = row.get(0).map_err(|e| e.to_string())?;
                Some(sig)
            } else {
                None
            }
        };
        if res.is_some() {
            res
        } else if norm_id.as_ref() != tool_id {
            let mut rows = stmt.query(params![tool_id]).map_err(|e| e.to_string())?;
            if let Some(row) = rows.next().map_err(|e| e.to_string())? {
                let sig: String = row.get(0).map_err(|e| e.to_string())?;
                Some(sig)
            } else {
                None
            }
        } else {
            None
        }
    };
    if let Some(sig) = found {
        if let Some(healed) = normalize_and_heal_signature(&sig) {
            if healed != sig {
                // Justification: signature heal write-back is a cache repair; the healed value is returned regardless
                crate::error::record_ignored(
                    save_tool_signature(norm_id.as_ref(), &healed),
                    "persist healed tool signature",
                );
            }
            return Ok(Some(healed));
        }
    }
    Ok(None)
}
