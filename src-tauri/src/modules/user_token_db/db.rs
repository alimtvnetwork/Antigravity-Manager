use rusqlite::{params, Connection, OptionalExtension};
use std::path::PathBuf;

use super::*;

/// Get database path
pub fn get_db_path() -> Result<PathBuf, String> {
    let mut path = crate::modules::account::get_data_dir()?;
    path.push("user_tokens.db");
    Ok(path)
}

/// Connect to database
pub fn connect_db() -> Result<Connection, String> {
    let path = get_db_path()?;
    let conn = Connection::open(&path).map_err(|e| format!("Failed to open database: {}", e))?;
    // Justification: best-effort SQLite pragma; logged
    crate::error::record_ignored(
        conn.pragma_update(None, "journal_mode", "WAL"),
        "pragma_update",
    );
    // Justification: best-effort SQLite pragma; the connection remains usable without it
    crate::error::record_ignored(
        conn.pragma_update(None, "busy_timeout", 5000),
        "busy_timeout",
    );
    // Justification: best-effort SQLite pragma; logged
    crate::error::record_ignored(
        conn.pragma_update(None, "synchronous", "NORMAL"),
        "pragma_update",
    );
    Ok(conn)
}

/// Initialize database
pub fn init_db() -> Result<(), String> {
    let conn = connect_db()?;

    // Create user_tokens table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS user_tokens (
            id TEXT PRIMARY KEY,
            token TEXT UNIQUE NOT NULL,
            username TEXT NOT NULL,
            description TEXT,
            enabled BOOLEAN NOT NULL DEFAULT 1,
            expires_type TEXT NOT NULL,
            expires_at INTEGER,
            max_ips INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            last_used_at INTEGER,
            total_requests INTEGER NOT NULL DEFAULT 0,
            total_tokens_used INTEGER NOT NULL DEFAULT 0,
            curfew_start TEXT,
            curfew_end TEXT
        )",
        [],
    )
    .map_err(|e| format!("Failed to create user_tokens table: {}", e))?;

    // Try adding new columns (for old database migrations, ignore existing errors)
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(
        conn.execute("ALTER TABLE user_tokens ADD COLUMN expires_type TEXT", []),
        "db execute",
    );
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(
        conn.execute("ALTER TABLE user_tokens ADD COLUMN expires_at INTEGER", []),
        "db execute",
    );
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(
        conn.execute(
            "ALTER TABLE user_tokens ADD COLUMN max_ips INTEGER DEFAULT 0",
            [],
        ),
        "db execute",
    );
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(
        conn.execute(
            "ALTER TABLE user_tokens ADD COLUMN total_requests INTEGER DEFAULT 0",
            [],
        ),
        "db execute",
    );
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(
        conn.execute(
            "ALTER TABLE user_tokens ADD COLUMN total_tokens_used INTEGER DEFAULT 0",
            [],
        ),
        "db execute",
    );
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(
        conn.execute(
            "ALTER TABLE user_tokens ADD COLUMN last_used_at INTEGER",
            [],
        ),
        "db execute",
    );
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(
        conn.execute("ALTER TABLE user_tokens ADD COLUMN curfew_start TEXT", []),
        "db execute",
    );
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(
        conn.execute("ALTER TABLE user_tokens ADD COLUMN curfew_end TEXT", []),
        "db execute",
    );

    // Create token_ip_bindings table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS token_ip_bindings (
            id TEXT PRIMARY KEY,
            token_id TEXT NOT NULL,
            ip_address TEXT NOT NULL,
            first_seen_at INTEGER NOT NULL,
            last_seen_at INTEGER NOT NULL,
            request_count INTEGER NOT NULL DEFAULT 0,
            user_agent TEXT,
            FOREIGN KEY(token_id) REFERENCES user_tokens(id) ON DELETE CASCADE,
            UNIQUE(token_id, ip_address)
        )",
        [],
    )
    .map_err(|e| format!("Failed to create token_ip_bindings table: {}", e))?;

    // Create token_usage_logs table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS token_usage_logs (
            id TEXT PRIMARY KEY,
            token_id TEXT NOT NULL,
            ip_address TEXT,
            model TEXT,
            input_tokens INTEGER,
            output_tokens INTEGER,
            request_time INTEGER NOT NULL,
            status INTEGER,
            FOREIGN KEY(token_id) REFERENCES user_tokens(id) ON DELETE CASCADE
        )",
        [],
    )
    .map_err(|e| format!("Failed to create token_usage_logs table: {}", e))?;

    // Create index
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_token_usage_logs_token_id ON token_usage_logs(token_id)",
        [],
    ), "db execute");
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(conn.execute("CREATE INDEX IF NOT EXISTS idx_token_usage_logs_request_time ON token_usage_logs(request_time)", []), "db execute");
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(conn.execute("CREATE INDEX IF NOT EXISTS idx_token_usage_logs_token_time ON token_usage_logs(token_id, request_time DESC)", []), "db execute");

    // [FIX Issue #1719] Data cleaning: fix NULL fields caused by old version upgrade
    // These fields may not exist in old versions and default to NULL after ALTER TABLE is added, causing deserialization to fail.
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(conn.execute("UPDATE user_tokens SET expires_type = 'never' WHERE expires_type IS NULL OR expires_type = ''", []), "db execute");
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(
        conn.execute(
            "UPDATE user_tokens SET max_ips = 0 WHERE max_ips IS NULL",
            [],
        ),
        "db execute",
    );
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(
        conn.execute(
            "UPDATE user_tokens SET total_requests = 0 WHERE total_requests IS NULL",
            [],
        ),
        "db execute",
    );
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(
        conn.execute(
            "UPDATE user_tokens SET total_tokens_used = 0 WHERE total_tokens_used IS NULL",
            [],
        ),
        "db execute",
    );
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(
        conn.execute(
            "UPDATE user_tokens SET enabled = 1 WHERE enabled IS NULL",
            [],
        ),
        "db execute",
    );

    Ok(())
}
