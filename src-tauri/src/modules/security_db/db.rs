use rusqlite::{params, Connection};
use std::path::PathBuf;

use super::*;

/// 获取安全数据库路径
pub fn get_security_db_path() -> Result<PathBuf, String> {
    let data_dir = crate::modules::account::get_data_dir()?;
    Ok(data_dir.join("security.db"))
}

/// 连接数据库
pub(crate) fn connect_db() -> Result<Connection, String> {
    let db_path = get_security_db_path()?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    // Enable WAL mode for better concurrency
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|e| e.to_string())?;

    // Set busy timeout
    conn.pragma_update(None, "busy_timeout", 5000)
        .map_err(|e| e.to_string())?;

    conn.pragma_update(None, "synchronous", "NORMAL")
        .map_err(|e| e.to_string())?;

    Ok(conn)
}

/// 初始化安全数据库
pub fn init_db() -> Result<(), String> {
    let conn = connect_db()?;

    // IP 访问日志表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS ip_access_logs (
            id TEXT PRIMARY KEY,
            client_ip TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            method TEXT,
            path TEXT,
            user_agent TEXT,
            status INTEGER,
            duration INTEGER,
            api_key_hash TEXT,
            blocked INTEGER DEFAULT 0,
            block_reason TEXT
        )",
        [],
    )
    .map_err(|e| e.to_string())?;

    // IP 黑名单表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS ip_blacklist (
            id TEXT PRIMARY KEY,
            ip_pattern TEXT NOT NULL UNIQUE,
            reason TEXT,
            created_at INTEGER NOT NULL,
            expires_at INTEGER,
            created_by TEXT DEFAULT 'manual',
            hit_count INTEGER DEFAULT 0
        )",
        [],
    )
    .map_err(|e| e.to_string())?;

    // IP 白名单表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS ip_whitelist (
            id TEXT PRIMARY KEY,
            ip_pattern TEXT NOT NULL UNIQUE,
            description TEXT,
            created_at INTEGER NOT NULL
        )",
        [],
    )
    .map_err(|e| e.to_string())?;

    // 创建索引
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_ip_access_ip ON ip_access_logs (client_ip)",
        [],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_ip_access_timestamp ON ip_access_logs (timestamp DESC)",
        [],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_ip_access_blocked ON ip_access_logs (blocked)",
        [],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_blacklist_pattern ON ip_blacklist (ip_pattern)",
        [],
    )
    .map_err(|e| e.to_string())?;

    // Migration: Add username column to ip_access_logs
    // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
    crate::error::record_ignored(
        conn.execute("ALTER TABLE ip_access_logs ADD COLUMN username TEXT", []),
        "db execute",
    );

    Ok(())
}
