use base64::prelude::*;
use rusqlite::{params, Connection, OptionalExtension};
use std::fs;
use std::path::PathBuf;

use super::*;

// ---------------------------------------------------------------------------
// Database Paths & Connections (Split Architecture)
// ---------------------------------------------------------------------------

/// Path to dedicated split email accounts & config SQLite database
pub fn get_email_vault_db_path() -> Result<PathBuf, String> {
    let mut path = crate::modules::account::get_data_dir()?;
    path.push("email_vault.db");
    Ok(path)
}

/// Path to separate dedicated split email passwords SQLite database
pub fn get_email_passwords_db_path() -> Result<PathBuf, String> {
    let mut path = crate::modules::account::get_data_dir()?;
    path.push("email_passwords.db");
    Ok(path)
}

/// Open connection to email vault SQLite database with WAL and pragmas
pub fn connect_vault_db() -> Result<Connection, String> {
    let path = get_email_vault_db_path()?;
    if let Some(parent) = path.parent() {
        // Justification: best-effort directory creation; later file ops fail loudly if the directory is actually needed
        crate::error::record_ignored(fs::create_dir_all(parent), "create_dir_all");
    }
    let conn = Connection::open(&path)
        .map_err(|e| format!("Failed to open email vault database: {}", e))?;

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

    init_vault_tables(&conn)?;
    Ok(conn)
}

/// Open connection to separate passwords SQLite database
pub fn connect_passwords_db() -> Result<Connection, String> {
    let path = get_email_passwords_db_path()?;
    if let Some(parent) = path.parent() {
        // Justification: best-effort directory creation; later file ops fail loudly if the directory is actually needed
        crate::error::record_ignored(fs::create_dir_all(parent), "create_dir_all");
    }
    let conn = Connection::open(&path)
        .map_err(|e| format!("Failed to open email passwords database: {}", e))?;

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

    init_passwords_table(&conn)?;
    Ok(conn)
}

/// Initialize SQLite schema for accounts, recipients, settings, and logs
pub fn init_vault_tables(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS email_accounts (
            id TEXT PRIMARY KEY,
            alias TEXT NOT NULL,
            email TEXT NOT NULL,
            smtp_host TEXT NOT NULL,
            smtp_port INTEGER NOT NULL DEFAULT 587,
            imap_host TEXT NOT NULL,
            imap_port INTEGER NOT NULL DEFAULT 993,
            encryption_type TEXT NOT NULL DEFAULT 'TLS',
            is_default INTEGER NOT NULL DEFAULT 0,
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("Failed to create email_accounts table: {}", e))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS notify_recipients (
            id TEXT PRIMARY KEY,
            email TEXT NOT NULL,
            group_name TEXT NOT NULL DEFAULT 'default',
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at INTEGER NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("Failed to create notify_recipients table: {}", e))?;

    // Deduplicate any historical duplicate recipients and enforce unique index on normalized email
    // Justification: best-effort deduplication before unique index creation
    crate::error::record_ignored(
        conn.execute(
            "DELETE FROM notify_recipients WHERE rowid NOT IN (
                SELECT MIN(rowid) FROM notify_recipients GROUP BY LOWER(TRIM(email))
            )",
            [],
        ),
        "dedupe notify_recipients on init",
    );
    // Justification: best-effort unique index creation
    crate::error::record_ignored(
        conn.execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_notify_recipients_email_lower ON notify_recipients (LOWER(TRIM(email)))",
            [],
        ),
        "create idx_notify_recipients_email_lower",
    );

    conn.execute(
        "CREATE TABLE IF NOT EXISTS email_notification_settings (
            id TEXT PRIMARY KEY DEFAULT 'global',
            is_enabled INTEGER NOT NULL DEFAULT 0,
            polling_interval_minutes INTEGER NOT NULL DEFAULT 3,
            inbox_check_interval_minutes INTEGER NOT NULL DEFAULT 1,
            baseline_polling_interval_minutes INTEGER NOT NULL DEFAULT 4,
            active_awaiting_interval_seconds INTEGER NOT NULL DEFAULT 10,
            notify_on_quota_drop INTEGER NOT NULL DEFAULT 1,
            quota_drop_threshold_percent INTEGER NOT NULL DEFAULT 25,
            notify_on_workspace_switch INTEGER NOT NULL DEFAULT 1,
            notify_on_idle_workspace INTEGER NOT NULL DEFAULT 1,
            notify_on_system_update INTEGER NOT NULL DEFAULT 1,
            allow_remote_prompt_execution INTEGER NOT NULL DEFAULT 1,
            allow_remote_cli_execution INTEGER NOT NULL DEFAULT 1,
            allow_remote_instance_rotation INTEGER NOT NULL DEFAULT 1,
            local_machine_name TEXT NOT NULL DEFAULT '',
            local_machine_ip TEXT NOT NULL DEFAULT '',
            updated_at INTEGER NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("Failed to create email_notification_settings table: {}", e))?;

    let existing_cols: Vec<String> = conn
        .prepare("PRAGMA table_info(email_notification_settings)")
        .and_then(|mut s| {
            let rows = s.query_map([], |r| r.get::<_, String>(1))?;
            Ok(rows.flatten().collect())
        })
        .unwrap_or_default();

    if !existing_cols
        .iter()
        .any(|c| c == "baseline_polling_interval_minutes")
    {
        // Justification: best-effort DB migration column addition
        crate::error::record_ignored(
            conn.execute(
                "ALTER TABLE email_notification_settings ADD COLUMN baseline_polling_interval_minutes INTEGER NOT NULL DEFAULT 4",
                [],
            ),
            "db execute",
        );
    }
    if !existing_cols
        .iter()
        .any(|c| c == "active_awaiting_interval_seconds")
    {
        // Justification: best-effort DB migration column addition
        crate::error::record_ignored(
            conn.execute(
                "ALTER TABLE email_notification_settings ADD COLUMN active_awaiting_interval_seconds INTEGER NOT NULL DEFAULT 10",
                [],
            ),
            "db execute",
        );
    }
    if !existing_cols.iter().any(|c| c == "notify_on_system_update") {
        // Justification: best-effort DB migration column addition
        crate::error::record_ignored(
            conn.execute(
                "ALTER TABLE email_notification_settings ADD COLUMN notify_on_system_update INTEGER NOT NULL DEFAULT 1",
                [],
            ),
            "db execute",
        );
    }

    conn.execute(
        "CREATE TABLE IF NOT EXISTS email_inbound_audit_log (
            id TEXT PRIMARY KEY,
            message_id TEXT NOT NULL,
            sender_email TEXT NOT NULL,
            subject TEXT NOT NULL,
            action_type TEXT NOT NULL,
            action_payload TEXT NOT NULL,
            execution_status TEXT NOT NULL,
            execution_result TEXT NOT NULL,
            received_at INTEGER NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("Failed to create email_inbound_audit_log table: {}", e))?;

    Ok(())
}

/// Initialize separate SQLite schema for password vault
pub fn init_passwords_table(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS email_credentials (
            account_id TEXT PRIMARY KEY,
            auth_type TEXT NOT NULL DEFAULT 'password',
            encrypted_secret TEXT NOT NULL,
            rsa_public_fingerprint TEXT NOT NULL,
            ssh_rsa_public_key TEXT NOT NULL,
            salt TEXT NOT NULL,
            updated_at INTEGER NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("Failed to create email_credentials table: {}", e))?;

    Ok(())
}
