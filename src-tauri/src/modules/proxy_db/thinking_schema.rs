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

    // Dynamic upgrade: Add primary_tool_id column for old version compatibility check
    // Justification: idempotent schema migration; failure is expected when the column already exists
    crate::error::record_ignored(
        conn.execute(
            "ALTER TABLE thinking_records ADD COLUMN primary_tool_id TEXT",
            [],
        ),
        "add primary_tool_id column migration",
    );

    // Dynamic upgrade: Add causal_tool_id column for deterministic causal pseudo-hash ID extremely fast penetration enumeration
    // Justification: idempotent schema migration; failure is expected when the column already exists
    crate::error::record_ignored(
        conn.execute(
            "ALTER TABLE thinking_records ADD COLUMN causal_tool_id TEXT",
            [],
        ),
        "add causal_tool_id column migration",
    );

    // 1. Covers the forward sequence scan (ORDER BY id ASC) of load_thinking_records and perfectly accepts the reverse sequence scan (ORDER BY id DESC).
    // Justification: lookup index is a performance accelerator; queries work without it
    crate::error::record_ignored(
        conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_thinking_rec_seq ON thinking_records (session_key, id ASC)",
        [],
    ),
        "create idx_thinking_rec_seq index",
    );
    // 2. Cover fast penetration enumeration based on causal_tool_id (minimalist Partial Index, ultimate nanosecond response)
    // Justification: lookup index is a performance accelerator; queries work without it
    crate::error::record_ignored(
        conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_thinking_rec_causal ON thinking_records (session_key, causal_tool_id) WHERE causal_tool_id IS NOT NULL",
        [],
    ),
        "create idx_thinking_rec_causal index",
    );
    // 3. Cover the fast penetration enumeration based on primary_tool_id (compatible with old version data)
    // Justification: lookup index is a performance accelerator; queries work without it
    crate::error::record_ignored(
        conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_thinking_rec_tool ON thinking_records (session_key, primary_tool_id) WHERE primary_tool_id IS NOT NULL",
        [],
    ),
        "create idx_thinking_rec_tool index",
    );
    // 4. Cover fingerprint-based fingerprint enumeration
    // Justification: lookup index is a performance accelerator; queries work without it
    crate::error::record_ignored(
        conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_thinking_rec_fp ON thinking_records (session_key, fingerprint)",
        [],
    ),
        "create idx_thinking_rec_fp index",
    );
    // 5. Overwrite historical cleanup time index
    // Justification: lookup index is a performance accelerator; queries work without it
    crate::error::record_ignored(
        conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_thinking_rec_accessed ON thinking_records (last_accessed ASC)",
        [],
    ),
        "create idx_thinking_rec_accessed index",
    );
    // 6. Cover precise penetration checking based on signature (minimalist Partial Index, WHERE signature IS NOT NULL)
    // Justification: lookup index is a performance accelerator; queries work without it
    crate::error::record_ignored(
        conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_thinking_rec_sig ON thinking_records (session_key, signature) WHERE signature IS NOT NULL",
        [],
    ),
        "create idx_thinking_rec_sig index",
    );

    // 7. Index downsizing: safely clean up physically redundant duplicate indexes and reduce write amplification overhead
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
