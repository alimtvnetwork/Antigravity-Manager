use rusqlite::{params, Connection, OpenFlags};
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, OnceLock};

use super::*;

pub(crate) static THINKING_DB: OnceLock<Mutex<Option<(PathBuf, Connection)>>> = OnceLock::new();

pub struct ThinkingDbGuard(pub(crate) MutexGuard<'static, Option<(PathBuf, Connection)>>);

impl std::ops::Deref for ThinkingDbGuard {
    type Target = Connection;
    fn deref(&self) -> &Self::Target {
        &self.0.as_ref().expect("thinking db connection").1
    }
}

impl std::ops::DerefMut for ThinkingDbGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0.as_mut().expect("thinking db connection").1
    }
}

/// Process-lifetime connection to thinking_store.db.
/// Fill/hydrate must not open proxy_logs.db (it can be multi-GB on HDD).
/// Automatically tracks data directory changes and reuses connection with fast pragmas.
fn thinking_db() -> Result<ThinkingDbGuard, String> {
    let db_path = get_thinking_db_path()?;
    let slot = THINKING_DB.get_or_init(|| Mutex::new(None));
    let mut guard = slot.lock().map_err(|e| format!("thinking db lock: {e}"))?;
    if guard.as_ref().map(|(p, _)| p) != Some(&db_path) {
        let conn = open_thinking_db_at(&db_path)?;
        *guard = Some((db_path, conn));
    }
    Ok(ThinkingDbGuard(guard))
}

fn mark_thinking_imported(conn: &Connection) {
    // Justification: session-row backfill after log import; best-effort
    crate::error::record_ignored(
        conn.execute(
            "INSERT OR REPLACE INTO thinking_meta (k, v) VALUES ('imported_from_proxy_logs', '1')",
            [],
        ),
        "backfill thinking_sessions after import",
    );
}

/// Copy old thinking rows out of proxy_logs.db into thinking_store.db.
/// Never deletes the log DB. Old uncompressed rows stay readable via unpack_thought.
fn migrate_thinking_from_logs() -> Result<(), String> {
    let conn = thinking_db()?;
    let imported: Option<String> = conn
        .query_row(
            "SELECT v FROM thinking_meta WHERE k = 'imported_from_proxy_logs'",
            [],
            |r| r.get(0),
        )
        .ok();
    if imported.as_deref() == Some("1") {
        return Ok(());
    }

    let logs_path = get_proxy_db_path()?;
    if !logs_path.exists() {
        mark_thinking_imported(&conn);
        return Ok(());
    }

    let escaped = logs_path
        .to_string_lossy()
        .replace('\\', "/")
        .replace('\'', "''");
    if conn
        .execute(&format!("ATTACH DATABASE '{}' AS logs", escaped), [])
        .is_err()
    {
        return Ok(());
    }

    let has_table: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM logs.sqlite_master WHERE type='table' AND name='thinking_records'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    if has_table == 0 {
        // Justification: releases the attached logs database; cleanup
        crate::error::record_ignored(
            conn.execute("DETACH DATABASE logs", []),
            "detach logs database",
        );
        mark_thinking_imported(&conn);
        return Ok(());
    }

    // Copy only rows not already present. Do not gzip/rewrite on import — that
    // would stall HDD by touching every old thought blob at startup.
    let copy_with_accessed = "INSERT INTO thinking_records (session_key, fingerprint, thought, signature, tool_ids, tool_names, visible, created_at, last_accessed)
             SELECT src.session_key, src.fingerprint, src.thought, src.signature, src.tool_ids, src.tool_names, src.visible, src.created_at,
                    COALESCE(src.last_accessed, src.created_at)
             FROM logs.thinking_records src
             WHERE NOT EXISTS (
                SELECT 1 FROM thinking_records t
                WHERE t.session_key = src.session_key
                  AND t.fingerprint = src.fingerprint
                  AND t.created_at = src.created_at
             )";
    let copy_basic = "INSERT INTO thinking_records (session_key, fingerprint, thought, signature, tool_ids, tool_names, visible, created_at, last_accessed)
             SELECT src.session_key, src.fingerprint, src.thought, src.signature, src.tool_ids, src.tool_names, src.visible, src.created_at, src.created_at
             FROM logs.thinking_records src
             WHERE NOT EXISTS (
                SELECT 1 FROM thinking_records t
                WHERE t.session_key = src.session_key
                  AND t.fingerprint = src.fingerprint
                  AND t.created_at = src.created_at
             )";
    let copied = match conn.execute(copy_with_accessed, []) {
        Ok(n) => n,
        Err(_) => {
            match conn.execute(copy_basic, []) {
                Ok(n) => n,
                Err(e) => {
                    // Justification: releases the attached logs database after a failed import; cleanup
                    crate::error::record_ignored(
                        conn.execute("DETACH DATABASE logs", []),
                        "detach logs database after failed import",
                    );
                    tracing::warn!("[ThinkingStore] Import from proxy_logs.db failed (will retry next start): {e}");
                    return Ok(());
                }
            }
        }
    };
    // Justification: session-row backfill after log import; best-effort
    crate::error::record_ignored(
        conn.execute(
            "INSERT OR IGNORE INTO thinking_sessions (session_key, last_accessed)
         SELECT session_key, MAX(created_at) FROM thinking_records GROUP BY session_key",
            [],
        ),
        "backfill thinking_sessions after import",
    );
    // Justification: releases the attached logs database after import; cleanup
    crate::error::record_ignored(
        conn.execute("DETACH DATABASE logs", []),
        "detach logs database after import",
    );
    mark_thinking_imported(&conn);
    if copied > 0 {
        tracing::info!(
            "[ThinkingStore] Imported {} thinking row(s) from proxy_logs.db (old file kept as backup)",
            copied
        );
    }
    Ok(())
}
