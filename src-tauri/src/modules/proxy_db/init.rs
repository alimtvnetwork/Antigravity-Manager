use rusqlite::{params, Connection, OpenFlags};

use super::thinking_db::migrate_thinking_from_logs;
use super::*;

pub fn init_db() -> Result<(), String> {
    let conn = Connection::open(get_proxy_db_path()?).map_err(|e| e.to_string())?;
    // Must precede WAL for new databases. Upgrade legacy databases if auto_vacuum is 0.
    let auto_vacuum: i64 = conn
        .pragma_query_value(None, "auto_vacuum", |r| r.get(0))
        .unwrap_or(0);
    if auto_vacuum == 0 {
        // Justification: legacy auto_vacuum upgrade is opportunistic; the database opens either way
        crate::error::record_ignored(
            conn.pragma_update(None, "auto_vacuum", "INCREMENTAL"),
            "set auto_vacuum INCREMENTAL on legacy database",
        );
        // Justification: legacy auto_vacuum upgrade is opportunistic; the database opens either way
        crate::error::record_ignored(
            conn.execute("VACUUM", []),
            "vacuum legacy database after auto_vacuum upgrade",
        );
    }
    apply_fast_pragmas(&conn)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS request_logs (
            id TEXT PRIMARY KEY,
            timestamp INTEGER,
            method TEXT,
            url TEXT,
            status INTEGER,
            duration INTEGER,
            model TEXT,
            error TEXT
        )",
        [],
    )
    .map_err(|e| e.to_string())?;

    // Try to add new columns (ignore errors if they exist)
    // Justification: idempotent schema migration; failure is expected when the column already exists
    crate::error::record_ignored(
        conn.execute("ALTER TABLE request_logs ADD COLUMN request_body TEXT", []),
        "add request_body column migration",
    );
    // Justification: idempotent schema migration; failure is expected when the column already exists
    crate::error::record_ignored(
        conn.execute(
            "ALTER TABLE request_logs ADD COLUMN upstream_request_body TEXT",
            [],
        ),
        "add upstream_request_body column migration",
    );
    // Justification: idempotent schema migration; failure is expected when the column already exists
    crate::error::record_ignored(
        conn.execute("ALTER TABLE request_logs ADD COLUMN response_body TEXT", []),
        "add response_body column migration",
    );
    // Justification: idempotent schema migration; failure is expected when the column already exists
    crate::error::record_ignored(
        conn.execute(
            "ALTER TABLE request_logs ADD COLUMN input_tokens INTEGER",
            [],
        ),
        "add input_tokens column migration",
    );
    // Justification: idempotent schema migration; failure is expected when the column already exists
    crate::error::record_ignored(
        conn.execute(
            "ALTER TABLE request_logs ADD COLUMN output_tokens INTEGER",
            [],
        ),
        "add output_tokens column migration",
    );
    // Justification: idempotent schema migration; failure is expected when the column already exists
    crate::error::record_ignored(
        conn.execute(
            "ALTER TABLE request_logs ADD COLUMN cached_tokens INTEGER",
            [],
        ),
        "add cached_tokens column migration",
    );
    // Justification: idempotent schema migration; failure is expected when the column already exists
    crate::error::record_ignored(
        conn.execute("ALTER TABLE request_logs ADD COLUMN account_email TEXT", []),
        "add account_email column migration",
    );
    // Justification: idempotent schema migration; failure is expected when the column already exists
    crate::error::record_ignored(
        conn.execute("ALTER TABLE request_logs ADD COLUMN mapped_model TEXT", []),
        "add mapped_model column migration",
    );
    // Justification: idempotent schema migration; failure is expected when the column already exists
    crate::error::record_ignored(
        conn.execute("ALTER TABLE request_logs ADD COLUMN protocol TEXT", []),
        "add protocol column migration",
    );
    // Justification: idempotent schema migration; failure is expected when the column already exists
    crate::error::record_ignored(
        conn.execute("ALTER TABLE request_logs ADD COLUMN client_ip TEXT", []),
        "add client_ip column migration",
    );
    // Justification: idempotent schema migration; failure is expected when the column already exists
    crate::error::record_ignored(
        conn.execute("ALTER TABLE request_logs ADD COLUMN username TEXT", []),
        "add username column migration",
    );
    // Justification: idempotent schema migration; failure is expected when the column already exists
    crate::error::record_ignored(
        conn.execute(
            "ALTER TABLE request_logs ADD COLUMN request_headers TEXT",
            [],
        ),
        "add request_headers column migration",
    );
    // Justification: idempotent schema migration; failure is expected when the column already exists
    crate::error::record_ignored(
        conn.execute(
            "ALTER TABLE request_logs ADD COLUMN upstream_request_headers TEXT",
            [],
        ),
        "add upstream_request_headers column migration",
    );
    // Justification: idempotent schema migration; failure is expected when the column already exists
    crate::error::record_ignored(
        conn.execute(
            "ALTER TABLE request_logs ADD COLUMN response_headers TEXT",
            [],
        ),
        "add response_headers column migration",
    );
    // Justification: idempotent schema migration; failure is expected when the column already exists
    crate::error::record_ignored(
        conn.execute("ALTER TABLE request_logs ADD COLUMN session_id TEXT", []),
        "add session_id column migration",
    );

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_timestamp ON request_logs (timestamp DESC)",
        [],
    )
    .map_err(|e| e.to_string())?;

    // Add status index for faster stats queries
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_status ON request_logs (status)",
        [],
    )
    .map_err(|e| e.to_string())?;

    // Efficient compound index: status and timestamp reverse order (for error filtering and paging sorting, greatly improving the response speed under large data volumes)
    // Justification: lookup index is a performance accelerator; queries work without it
    crate::error::record_ignored(
        conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_status_timestamp ON request_logs (status, timestamp DESC)",
        [],
    ),
        "create idx_status_timestamp index",
    );

    // Composite index: model and timestamp reverse order (for model-level log filtering and sorting)
    // Justification: lookup index is a performance accelerator; queries work without it
    crate::error::record_ignored(
        conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_model_timestamp ON request_logs (model, timestamp DESC)",
        [],
    ),
        "create idx_model_timestamp index",
    );

    // Composite index: account email and timestamp reverse order (for multi-user/multi-account filtering)
    // Justification: lookup index is a performance accelerator; queries work without it
    crate::error::record_ignored(
        conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_account_timestamp ON request_logs (account_email, timestamp DESC)",
        [],
    ),
        "create idx_account_timestamp index",
    );

    // Composite index: client IP and timestamp in reverse order (for security auditing and IP filtering)
    // Justification: lookup index is a performance accelerator; queries work without it
    crate::error::record_ignored(
        conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_client_ip_timestamp ON request_logs (client_ip, timestamp DESC)",
        [],
    ),
        "create idx_client_ip_timestamp index",
    );

    // Composite index: username and timestamp in reverse order
    // Justification: lookup index is a performance accelerator; queries work without it
    crate::error::record_ignored(
        conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_username_timestamp ON request_logs (username, timestamp DESC)",
        [],
    ),
        "create idx_username_timestamp index",
    );

    // Composite index: session and timestamp reverse order (for session granular operation and maintenance analysis)
    // Justification: lookup index is a performance accelerator; queries work without it
    crate::error::record_ignored(
        conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_session_timestamp ON request_logs (session_id, timestamp DESC)",
        [],
    ),
        "create idx_session_timestamp index",
    );

    // Single column index: protocol type
    // Justification: lookup index is a performance accelerator; queries work without it
    crate::error::record_ignored(
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_protocol ON request_logs (protocol)",
            [],
        ),
        "create idx_protocol index",
    );

    // Single column index: request method
    // Justification: lookup index is a performance accelerator; queries work without it
    crate::error::record_ignored(
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_method ON request_logs (method)",
            [],
        ),
        "create idx_method index",
    );

    // Persistence tool signature table (supports recovery of real encrypted signatures in seconds based on tool_id after agent restart)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tool_signatures (
            tool_id TEXT PRIMARY KEY,
            signature TEXT NOT NULL,
            created_at INTEGER NOT NULL
        )",
        [],
    )
    .map_err(|e| e.to_string())?;
    // Justification: lookup index is a performance accelerator; queries work without it
    crate::error::record_ignored(
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tool_sig_created ON tool_signatures (created_at DESC)",
            [],
        ),
        "create idx_tool_sig_created index",
    );

    drop(conn);
    migrate_thinking_from_logs()?;

    Ok(())
}
