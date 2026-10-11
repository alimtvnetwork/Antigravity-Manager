use rusqlite::{params, Connection, OpenFlags};

use super::*;
use super::thinking_db::migrate_thinking_from_logs;

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

    // 高效复合索引：状态与时间戳倒序（针对错误筛选与分页排序，极大提升大数据量下的响应速度）
    // Justification: lookup index is a performance accelerator; queries work without it
    crate::error::record_ignored(
        conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_status_timestamp ON request_logs (status, timestamp DESC)",
        [],
    ),
        "create idx_status_timestamp index",
    );

    // 复合索引：模型与时间戳倒序（针对模型级日志过滤与排序）
    // Justification: lookup index is a performance accelerator; queries work without it
    crate::error::record_ignored(
        conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_model_timestamp ON request_logs (model, timestamp DESC)",
        [],
    ),
        "create idx_model_timestamp index",
    );

    // 复合索引：账号邮箱与时间戳倒序（针对多用户/多账号过滤）
    // Justification: lookup index is a performance accelerator; queries work without it
    crate::error::record_ignored(
        conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_account_timestamp ON request_logs (account_email, timestamp DESC)",
        [],
    ),
        "create idx_account_timestamp index",
    );

    // 复合索引：客户端IP与时间戳倒序（针对安全审计与IP过滤）
    // Justification: lookup index is a performance accelerator; queries work without it
    crate::error::record_ignored(
        conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_client_ip_timestamp ON request_logs (client_ip, timestamp DESC)",
        [],
    ),
        "create idx_client_ip_timestamp index",
    );

    // 复合索引：用户名与时间戳倒序
    // Justification: lookup index is a performance accelerator; queries work without it
    crate::error::record_ignored(
        conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_username_timestamp ON request_logs (username, timestamp DESC)",
        [],
    ),
        "create idx_username_timestamp index",
    );

    // 复合索引：会话与时间戳倒序（针对会话粒度运维分析）
    // Justification: lookup index is a performance accelerator; queries work without it
    crate::error::record_ignored(
        conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_session_timestamp ON request_logs (session_id, timestamp DESC)",
        [],
    ),
        "create idx_session_timestamp index",
    );

    // 单列索引：协议类型
    // Justification: lookup index is a performance accelerator; queries work without it
    crate::error::record_ignored(
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_protocol ON request_logs (protocol)",
            [],
        ),
        "create idx_protocol index",
    );

    // 单列索引：请求方法
    // Justification: lookup index is a performance accelerator; queries work without it
    crate::error::record_ignored(
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_method ON request_logs (method)",
            [],
        ),
        "create idx_method index",
    );

    // 持久化工具签名表 (支持代理重启后根据 tool_id 秒级恢复真实加密签名)
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
