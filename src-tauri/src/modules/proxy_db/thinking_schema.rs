use rusqlite::{params, Connection, OpenFlags};
use std::path::PathBuf;

use super::*;

pub fn is_synthetic_tool_id(id: &str) -> bool {
    id.starts_with("call_") && id.chars().filter(|&c| c == '_').count() >= 3
}

pub(crate) fn init_thinking_schema(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS thinking_records (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_key TEXT NOT NULL,
            fingerprint TEXT NOT NULL,
            thought TEXT NOT NULL,
            signature TEXT,
            tool_ids TEXT NOT NULL,
            tool_names TEXT NOT NULL,
            visible TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            last_accessed INTEGER
        )",
        [],
    )
    .map_err(|e| e.to_string())?;

    // 动态升级：增加 primary_tool_id 列用于旧版兼容点查
    // Justification: idempotent schema migration; failure is expected when the column already exists
    crate::error::record_ignored(
        conn.execute(
            "ALTER TABLE thinking_records ADD COLUMN primary_tool_id TEXT",
            [],
        ),
        "add primary_tool_id column migration",
    );

    // 动态升级：增加 causal_tool_id 列用于确定性因果伪哈希 ID 极速穿透点查
    // Justification: idempotent schema migration; failure is expected when the column already exists
    crate::error::record_ignored(
        conn.execute(
            "ALTER TABLE thinking_records ADD COLUMN causal_tool_id TEXT",
            [],
        ),
        "add causal_tool_id column migration",
    );

    // 1. 覆盖 load_thinking_records 的正向序列扫描 (ORDER BY id ASC)，同时完美承接逆序扫描 (ORDER BY id DESC)
    // Justification: lookup index is a performance accelerator; queries work without it
    crate::error::record_ignored(
        conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_thinking_rec_seq ON thinking_records (session_key, id ASC)",
        [],
    ),
        "create idx_thinking_rec_seq index",
    );
    // 2. 覆盖基于 causal_tool_id 的快速穿透点查 (极简 Partial Index，极致纳秒响应)
    // Justification: lookup index is a performance accelerator; queries work without it
    crate::error::record_ignored(
        conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_thinking_rec_causal ON thinking_records (session_key, causal_tool_id) WHERE causal_tool_id IS NOT NULL",
        [],
    ),
        "create idx_thinking_rec_causal index",
    );
    // 3. 覆盖基于 primary_tool_id 的快速穿透点查 (兼容旧版数据)
    // Justification: lookup index is a performance accelerator; queries work without it
    crate::error::record_ignored(
        conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_thinking_rec_tool ON thinking_records (session_key, primary_tool_id) WHERE primary_tool_id IS NOT NULL",
        [],
    ),
        "create idx_thinking_rec_tool index",
    );
    // 4. 覆盖基于 fingerprint 的指纹点查
    // Justification: lookup index is a performance accelerator; queries work without it
    crate::error::record_ignored(
        conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_thinking_rec_fp ON thinking_records (session_key, fingerprint)",
        [],
    ),
        "create idx_thinking_rec_fp index",
    );
    // 5. 覆盖历史清理时间索引
    // Justification: lookup index is a performance accelerator; queries work without it
    crate::error::record_ignored(
        conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_thinking_rec_accessed ON thinking_records (last_accessed ASC)",
        [],
    ),
        "create idx_thinking_rec_accessed index",
    );
    // 6. 覆盖基于 signature 的精准穿透点查 (极简 Partial Index，WHERE signature IS NOT NULL)
    // Justification: lookup index is a performance accelerator; queries work without it
    crate::error::record_ignored(
        conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_thinking_rec_sig ON thinking_records (session_key, signature) WHERE signature IS NOT NULL",
        [],
    ),
        "create idx_thinking_rec_sig index",
    );

    // 7. 索引大瘦身：安全清理物理冗余的重复索引，削减写放大开销
    // Justification: removes a redundant legacy index; failure leaves a harmless duplicate
    crate::error::record_ignored(
        conn.execute("DROP INDEX IF EXISTS idx_thinking_rec_latest", []),
        "drop legacy idx_thinking_rec_latest index",
    );
    // Justification: removes a redundant legacy index; failure leaves a harmless duplicate
    crate::error::record_ignored(
        conn.execute("DROP INDEX IF EXISTS idx_thinking_rec_session", []),
        "drop legacy idx_thinking_rec_session index",
    );
    conn.execute(
        "CREATE TABLE IF NOT EXISTS thinking_sessions (
            session_key TEXT PRIMARY KEY,
            last_accessed INTEGER NOT NULL
        )",
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS thinking_meta (
            k TEXT PRIMARY KEY,
            v TEXT NOT NULL
        )",
        [],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) fn open_thinking_db_at(db_path: &PathBuf) -> Result<Connection, String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    apply_fast_pragmas(&conn)?;
    init_thinking_schema(&conn)?;
    Ok(conn)
}

pub(crate) fn open_thinking_db() -> Result<Connection, String> {
    let db_path = get_thinking_db_path()?;
    open_thinking_db_at(&db_path)
}
