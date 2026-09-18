//! Split Email Vault Database Module
//! Dedicated SQLite Database for email accounts, separate credentials vault,
//! notification recipients, watcher settings, and inbound audit logs.

#![allow(dead_code)]

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::prelude::*;
use chrono::Utc;
use rand::RngCore;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

/// Represents an email account (public configuration)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAccount {
    pub id: String,
    pub alias: String,
    pub email: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub imap_host: String,
    pub imap_port: u16,
    pub encryption_type: String, // "TLS", "STARTTLS", "SSL", "NONE"
    pub is_default: bool,
    pub is_active: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Input payload for creating or updating an email account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAccountInput {
    pub id: Option<String>,
    pub alias: String,
    pub email: String,
    pub password: Option<String>,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub imap_host: String,
    pub imap_port: u16,
    pub encryption_type: String,
    pub is_default: bool,
    pub is_active: bool,
}

/// Represents a secure credential record in the dedicated split passwords database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailCredential {
    pub account_id: String,
    pub auth_type: String,
    pub encrypted_secret: String,
    pub rsa_public_fingerprint: String,
    pub ssh_rsa_public_key: String,
    pub salt: String,
    pub updated_at: i64,
}

/// Represents a notification recipient (individual or group)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyRecipient {
    pub id: String,
    pub email: String,
    pub group_name: String,
    pub is_active: bool,
    pub created_at: i64,
}

/// Input payload for creating a notification recipient
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyRecipientInput {
    pub email: String,
    pub group_name: Option<String>,
    pub is_active: Option<bool>,
}

/// Email watcher & notification settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailNotificationSettings {
    pub id: String,
    pub is_enabled: bool,
    pub polling_interval_minutes: u32,
    pub inbox_check_interval_minutes: u32,
    pub notify_on_quota_drop: bool,
    pub quota_drop_threshold_percent: u32,
    pub notify_on_workspace_switch: bool,
    pub notify_on_idle_workspace: bool,
    pub allow_remote_prompt_execution: bool,
    pub allow_remote_cli_execution: bool,
    pub allow_remote_instance_rotation: bool,
    pub local_machine_name: String,
    pub local_machine_ip: String,
    pub updated_at: i64,
}

impl Default for EmailNotificationSettings {
    fn default() -> Self {
        Self {
            id: "global".to_string(),
            is_enabled: false,
            polling_interval_minutes: 3,
            inbox_check_interval_minutes: 1,
            notify_on_quota_drop: true,
            quota_drop_threshold_percent: 15,
            notify_on_workspace_switch: true,
            notify_on_idle_workspace: true,
            allow_remote_prompt_execution: true,
            allow_remote_cli_execution: true,
            allow_remote_instance_rotation: true,
            local_machine_name: String::new(),
            local_machine_ip: String::new(),
            updated_at: Utc::now().timestamp(),
        }
    }
}

/// Represents an inbound audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailInboundAuditLog {
    pub id: String,
    pub message_id: String,
    pub sender_email: String,
    pub subject: String,
    pub action_type: String,
    pub action_payload: String,
    pub execution_status: String,
    pub execution_result: String,
    pub received_at: i64,
}

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
        let _ = fs::create_dir_all(parent);
    }
    let conn = Connection::open(&path)
        .map_err(|e| format!("Failed to open email vault database: {}", e))?;

    let _ = conn.pragma_update(None, "journal_mode", "WAL");
    let _ = conn.pragma_update(None, "busy_timeout", 5000);
    let _ = conn.pragma_update(None, "synchronous", "NORMAL");

    init_vault_tables(&conn)?;
    Ok(conn)
}

/// Open connection to separate passwords SQLite database
pub fn connect_passwords_db() -> Result<Connection, String> {
    let path = get_email_passwords_db_path()?;
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let conn = Connection::open(&path)
        .map_err(|e| format!("Failed to open email passwords database: {}", e))?;

    let _ = conn.pragma_update(None, "journal_mode", "WAL");
    let _ = conn.pragma_update(None, "busy_timeout", 5000);
    let _ = conn.pragma_update(None, "synchronous", "NORMAL");

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

    conn.execute(
        "CREATE TABLE IF NOT EXISTS email_notification_settings (
            id TEXT PRIMARY KEY DEFAULT 'global',
            is_enabled INTEGER NOT NULL DEFAULT 0,
            polling_interval_minutes INTEGER NOT NULL DEFAULT 3,
            inbox_check_interval_minutes INTEGER NOT NULL DEFAULT 1,
            notify_on_quota_drop INTEGER NOT NULL DEFAULT 1,
            quota_drop_threshold_percent INTEGER NOT NULL DEFAULT 15,
            notify_on_workspace_switch INTEGER NOT NULL DEFAULT 1,
            notify_on_idle_workspace INTEGER NOT NULL DEFAULT 1,
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

// ---------------------------------------------------------------------------
// Cryptographic Secret Encryption & SSH RSA Key Derivation
// ---------------------------------------------------------------------------

/// Derive encryption key from machine identity and salt
fn derive_aes_key(salt: &str) -> [u8; 32] {
    let machine_id = machine_uid::get().unwrap_or_else(|_| "antigravity-default-seed".to_string());
    let mut hasher = Sha256::new();
    hasher.update(machine_id.as_bytes());
    hasher.update(b":vault-kdf:");
    hasher.update(salt.as_bytes());
    let hash = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&hash[0..32]);
    key
}

/// Compute SSH RSA public key token and fingerprint
pub fn compute_ssh_rsa_identity(salt: &str, secret: &str) -> (String, String) {
    let mut hasher = Sha256::new();
    hasher.update(b"ssh-rsa-vault-token-v1:");
    hasher.update(salt.as_bytes());
    hasher.update(secret.as_bytes());
    let hash = hasher.finalize();

    let fingerprint = format!("SHA256:{}", BASE64_STANDARD.encode(hash));
    let pub_key = format!(
        "ssh-rsa AAAAB3NzaC1yc2E{} antigravity@node",
        BASE64_STANDARD.encode(hash)
    );
    (fingerprint, pub_key)
}

/// Encrypt secret using AES-256-GCM and generate SSH RSA token
pub fn encrypt_secret(secret: &str, salt: &str) -> Result<(String, String, String), String> {
    let key_bytes = derive_aes_key(salt);
    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| format!("Failed to init cipher: {}", e))?;

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, secret.as_bytes())
        .map_err(|e| format!("Failed to encrypt secret: {}", e))?;

    let mut combined = Vec::with_capacity(12 + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    let enc_b64 = BASE64_STANDARD.encode(&combined);
    let (fingerprint, pub_key) = compute_ssh_rsa_identity(salt, secret);

    Ok((enc_b64, fingerprint, pub_key))
}

/// Decrypt secret using AES-256-GCM
pub fn decrypt_secret(encrypted_b64: &str, salt: &str) -> Result<String, String> {
    let data = BASE64_STANDARD
        .decode(encrypted_b64)
        .map_err(|e| format!("Invalid base64 payload: {}", e))?;

    if data.len() < 12 {
        return Err("Payload too short for nonce".to_string());
    }

    let (nonce_bytes, ciphertext) = data.split_at(12);
    let key_bytes = derive_aes_key(salt);
    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| format!("Failed to init cipher: {}", e))?;

    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    String::from_utf8(plaintext).map_err(|e| format!("Invalid utf8 secret: {}", e))
}

// ---------------------------------------------------------------------------
// Account CRUD
// ---------------------------------------------------------------------------

/// List all email accounts
pub fn list_email_accounts() -> Result<Vec<EmailAccount>, String> {
    let conn = connect_vault_db()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, alias, email, smtp_host, smtp_port, imap_host, imap_port, 
                    encryption_type, is_default, is_active, created_at, updated_at 
             FROM email_accounts ORDER BY is_default DESC, created_at ASC",
        )
        .map_err(|e| format!("Failed to prepare list accounts: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            let def_int: i32 = row.get(8)?;
            let act_int: i32 = row.get(9)?;
            Ok(EmailAccount {
                id: row.get(0)?,
                alias: row.get(1)?,
                email: row.get(2)?,
                smtp_host: row.get(3)?,
                smtp_port: row.get::<_, u16>(4)?,
                imap_host: row.get(5)?,
                imap_port: row.get::<_, u16>(6)?,
                encryption_type: row.get(7)?,
                is_default: def_int > 0,
                is_active: act_int > 0,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })
        .map_err(|e| format!("Failed to query email accounts: {}", e))?
        .flatten()
        .collect();

    Ok(rows)
}

/// Save or update an email account and its credentials in separate split database
pub fn upsert_email_account(input: EmailAccountInput) -> Result<EmailAccount, String> {
    let vault_conn = connect_vault_db()?;
    let now = Utc::now().timestamp();
    let account_id = input.id.unwrap_or_else(|| Uuid::new_v4().to_string());

    if input.is_default {
        let _ = vault_conn.execute("UPDATE email_accounts SET is_default = 0", []);
    }

    let def_int = if input.is_default { 1 } else { 0 };
    let act_int = if input.is_active { 1 } else { 0 };

    vault_conn.execute(
        "INSERT INTO email_accounts 
         (id, alias, email, smtp_host, smtp_port, imap_host, imap_port, encryption_type, is_default, is_active, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET
            alias = excluded.alias,
            email = excluded.email,
            smtp_host = excluded.smtp_host,
            smtp_port = excluded.smtp_port,
            imap_host = excluded.imap_host,
            imap_port = excluded.imap_port,
            encryption_type = excluded.encryption_type,
            is_default = excluded.is_default,
            is_active = excluded.is_active,
            updated_at = excluded.updated_at",
        params![
            &account_id,
            &input.alias,
            &input.email,
            &input.smtp_host,
            input.smtp_port,
            &input.imap_host,
            input.imap_port,
            &input.encryption_type,
            def_int,
            act_int,
            now,
            now,
        ],
    )
    .map_err(|e| format!("Failed to upsert email account: {}", e))?;

    // Store password in separate split database email_passwords.db
    if let Some(ref pwd) = input.password {
        let has_content = !pwd.trim().is_empty();
        if has_content {
            let pass_conn = connect_passwords_db()?;
            let mut salt_bytes = [0u8; 16];
            rand::thread_rng().fill_bytes(&mut salt_bytes);
            let salt = BASE64_STANDARD.encode(salt_bytes);

            let (enc_secret, fingerprint, ssh_pub) = encrypt_secret(pwd, &salt)?;

            pass_conn.execute(
                "INSERT INTO email_credentials 
                 (account_id, auth_type, encrypted_secret, rsa_public_fingerprint, ssh_rsa_public_key, salt, updated_at)
                 VALUES (?, 'password', ?, ?, ?, ?, ?)
                 ON CONFLICT(account_id) DO UPDATE SET
                    encrypted_secret = excluded.encrypted_secret,
                    rsa_public_fingerprint = excluded.rsa_public_fingerprint,
                    ssh_rsa_public_key = excluded.ssh_rsa_public_key,
                    salt = excluded.salt,
                    updated_at = excluded.updated_at",
                params![&account_id, enc_secret, fingerprint, ssh_pub, salt, now],
            )
            .map_err(|e| format!("Failed to save credential in split passwords database: {}", e))?;
        }
    }

    Ok(EmailAccount {
        id: account_id,
        alias: input.alias,
        email: input.email,
        smtp_host: input.smtp_host,
        smtp_port: input.smtp_port,
        imap_host: input.imap_host,
        imap_port: input.imap_port,
        encryption_type: input.encryption_type,
        is_default: input.is_default,
        is_active: input.is_active,
        created_at: now,
        updated_at: now,
    })
}

/// Delete an email account and cascade credentials from separate passwords database
pub fn delete_email_account(account_id: &str) -> Result<(), String> {
    let vault_conn = connect_vault_db()?;
    vault_conn
        .execute(
            "DELETE FROM email_accounts WHERE id = ?",
            params![account_id],
        )
        .map_err(|e| format!("Failed to delete email account: {}", e))?;

    if let Ok(pass_conn) = connect_passwords_db() {
        let _ = pass_conn.execute(
            "DELETE FROM email_credentials WHERE account_id = ?",
            params![account_id],
        );
    }
    Ok(())
}

/// Set designated default email account
pub fn set_default_email_account(account_id: &str) -> Result<(), String> {
    let conn = connect_vault_db()?;
    conn.execute("UPDATE email_accounts SET is_default = 0", [])
        .map_err(|e| format!("Failed to reset default flags: {}", e))?;

    conn.execute(
        "UPDATE email_accounts SET is_default = 1 WHERE id = ?",
        params![account_id],
    )
    .map_err(|e| format!("Failed to set default account: {}", e))?;

    Ok(())
}

/// Retrieve decrypted password from separate split passwords database
pub fn get_account_secret(account_id: &str) -> Result<String, String> {
    let conn = connect_passwords_db()?;
    let mut stmt = conn
        .prepare("SELECT encrypted_secret, salt FROM email_credentials WHERE account_id = ?")
        .map_err(|e| format!("Failed to prepare credential lookup: {}", e))?;

    let row = stmt
        .query_row(params![account_id], |r| {
            let enc: String = r.get(0)?;
            let salt: String = r.get(1)?;
            Ok((enc, salt))
        })
        .optional()
        .map_err(|e| format!("Failed to query credentials: {}", e))?;

    let cred =
        row.ok_or_else(|| format!("No credential record found for account '{}'", account_id))?;
    decrypt_secret(&cred.0, &cred.1)
}

// ---------------------------------------------------------------------------
// Notification Recipients CRUD
// ---------------------------------------------------------------------------

/// List notification recipients
pub fn list_notify_recipients() -> Result<Vec<NotifyRecipient>, String> {
    let conn = connect_vault_db()?;
    let mut stmt = conn
        .prepare("SELECT id, email, group_name, is_active, created_at FROM notify_recipients ORDER BY created_at ASC")
        .map_err(|e| format!("Failed to prepare list recipients: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            let act_int: i32 = row.get(3)?;
            Ok(NotifyRecipient {
                id: row.get(0)?,
                email: row.get(1)?,
                group_name: row.get(2)?,
                is_active: act_int > 0,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| format!("Failed to query notify recipients: {}", e))?
        .flatten()
        .collect();

    Ok(rows)
}

/// Add notification recipient
pub fn add_notify_recipient(input: NotifyRecipientInput) -> Result<NotifyRecipient, String> {
    let conn = connect_vault_db()?;
    let id = Uuid::new_v4().to_string();
    let group = input.group_name.unwrap_or_else(|| "default".to_string());
    let is_active = input.is_active.unwrap_or(true);
    let act_int = if is_active { 1 } else { 0 };
    let now = Utc::now().timestamp();

    conn.execute(
        "INSERT INTO notify_recipients (id, email, group_name, is_active, created_at) VALUES (?, ?, ?, ?, ?)",
        params![&id, &input.email, &group, act_int, now],
    )
    .map_err(|e| format!("Failed to add notify recipient: {}", e))?;

    Ok(NotifyRecipient {
        id,
        email: input.email,
        group_name: group,
        is_active,
        created_at: now,
    })
}

/// Delete notification recipient
pub fn delete_notify_recipient(id: &str) -> Result<(), String> {
    let conn = connect_vault_db()?;
    conn.execute("DELETE FROM notify_recipients WHERE id = ?", params![id])
        .map_err(|e| format!("Failed to delete notify recipient: {}", e))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Notification Settings CRUD
// ---------------------------------------------------------------------------

/// Load notification settings
pub fn get_notification_settings() -> Result<EmailNotificationSettings, String> {
    let conn = connect_vault_db()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, is_enabled, polling_interval_minutes, inbox_check_interval_minutes,
                    notify_on_quota_drop, quota_drop_threshold_percent, notify_on_workspace_switch,
                    notify_on_idle_workspace, allow_remote_prompt_execution, allow_remote_cli_execution,
                    allow_remote_instance_rotation, local_machine_name, local_machine_ip, updated_at
             FROM email_notification_settings WHERE id = 'global'",
        )
        .map_err(|e| format!("Failed to prepare settings query: {}", e))?;

    let row = stmt
        .query_row([], |r| {
            let is_en: i32 = r.get(1)?;
            let p_int: u32 = r.get(2)?;
            let in_int: u32 = r.get(3)?;
            let n_quota: i32 = r.get(4)?;
            let q_drop: u32 = r.get(5)?;
            let n_ws: i32 = r.get(6)?;
            let n_idle: i32 = r.get(7)?;
            let a_prompt: i32 = r.get(8)?;
            let a_cli: i32 = r.get(9)?;
            let a_inst: i32 = r.get(10)?;
            Ok(EmailNotificationSettings {
                id: r.get(0)?,
                is_enabled: is_en > 0,
                polling_interval_minutes: p_int,
                inbox_check_interval_minutes: in_int,
                notify_on_quota_drop: n_quota > 0,
                quota_drop_threshold_percent: q_drop,
                notify_on_workspace_switch: n_ws > 0,
                notify_on_idle_workspace: n_idle > 0,
                allow_remote_prompt_execution: a_prompt > 0,
                allow_remote_cli_execution: a_cli > 0,
                allow_remote_instance_rotation: a_inst > 0,
                local_machine_name: r.get(11)?,
                local_machine_ip: r.get(12)?,
                updated_at: r.get(13)?,
            })
        })
        .optional()
        .map_err(|e| format!("Failed to query notification settings: {}", e))?;

    Ok(row.unwrap_or_default())
}

/// Save notification settings
pub fn save_notification_settings(settings: EmailNotificationSettings) -> Result<(), String> {
    let conn = connect_vault_db()?;
    let now = Utc::now().timestamp();

    let is_en = if settings.is_enabled { 1 } else { 0 };
    let n_quota = if settings.notify_on_quota_drop { 1 } else { 0 };
    let n_ws = if settings.notify_on_workspace_switch {
        1
    } else {
        0
    };
    let n_idle = if settings.notify_on_idle_workspace {
        1
    } else {
        0
    };
    let a_prompt = if settings.allow_remote_prompt_execution {
        1
    } else {
        0
    };
    let a_cli = if settings.allow_remote_cli_execution {
        1
    } else {
        0
    };
    let a_inst = if settings.allow_remote_instance_rotation {
        1
    } else {
        0
    };

    conn.execute(
        "INSERT INTO email_notification_settings
         (id, is_enabled, polling_interval_minutes, inbox_check_interval_minutes,
          notify_on_quota_drop, quota_drop_threshold_percent, notify_on_workspace_switch,
          notify_on_idle_workspace, allow_remote_prompt_execution, allow_remote_cli_execution,
          allow_remote_instance_rotation, local_machine_name, local_machine_ip, updated_at)
         VALUES ('global', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET
            is_enabled = excluded.is_enabled,
            polling_interval_minutes = excluded.polling_interval_minutes,
            inbox_check_interval_minutes = excluded.inbox_check_interval_minutes,
            notify_on_quota_drop = excluded.notify_on_quota_drop,
            quota_drop_threshold_percent = excluded.quota_drop_threshold_percent,
            notify_on_workspace_switch = excluded.notify_on_workspace_switch,
            notify_on_idle_workspace = excluded.notify_on_idle_workspace,
            allow_remote_prompt_execution = excluded.allow_remote_prompt_execution,
            allow_remote_cli_execution = excluded.allow_remote_cli_execution,
            allow_remote_instance_rotation = excluded.allow_remote_instance_rotation,
            local_machine_name = excluded.local_machine_name,
            local_machine_ip = excluded.local_machine_ip,
            updated_at = excluded.updated_at",
        params![
            is_en,
            settings.polling_interval_minutes,
            settings.inbox_check_interval_minutes,
            n_quota,
            settings.quota_drop_threshold_percent,
            n_ws,
            n_idle,
            a_prompt,
            a_cli,
            a_inst,
            &settings.local_machine_name,
            &settings.local_machine_ip,
            now,
        ],
    )
    .map_err(|e| format!("Failed to save notification settings: {}", e))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Inbound Audit Log & Replay Guard
// ---------------------------------------------------------------------------

/// Record an inbound command in audit log
pub fn record_inbound_audit_log(entry: EmailInboundAuditLog) -> Result<(), String> {
    let conn = connect_vault_db()?;
    conn.execute(
        "INSERT INTO email_inbound_audit_log 
         (id, message_id, sender_email, subject, action_type, action_payload, execution_status, execution_result, received_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            &entry.id,
            &entry.message_id,
            &entry.sender_email,
            &entry.subject,
            &entry.action_type,
            &entry.action_payload,
            &entry.execution_status,
            &entry.execution_result,
            entry.received_at,
        ],
    )
    .map_err(|e| format!("Failed to record audit log: {}", e))?;
    Ok(())
}

/// Check if message_id has already been processed to prevent replay
pub fn is_message_already_processed(message_id: &str) -> Result<bool, String> {
    let conn = connect_vault_db()?;
    let mut stmt = conn
        .prepare("SELECT COUNT(1) FROM email_inbound_audit_log WHERE message_id = ?")
        .map_err(|e| format!("Failed to check message replay: {}", e))?;

    let count: i64 = stmt
        .query_row(params![message_id], |r| r.get(0))
        .unwrap_or(0);

    let has_seen = count > 0;
    Ok(has_seen)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let salt = "test_salt_123456";
        let secret = "MySecretMailPassword!@#";
        let (encrypted, fingerprint, ssh_key) = encrypt_secret(secret, salt).unwrap();
        assert!(fingerprint.starts_with("SHA256:"));
        assert!(ssh_key.starts_with("ssh-rsa "));
        assert_ne!(encrypted, secret);

        let decrypted = decrypt_secret(&encrypted, salt).unwrap();
        assert_eq!(decrypted, secret);
    }

    #[test]
    fn test_vault_table_initialization() {
        let conn = Connection::open_in_memory().unwrap();
        let init_res = init_vault_tables(&conn);
        assert!(init_res.is_ok());

        let insert_res = conn.execute(
            "INSERT INTO email_accounts (id, alias, email, smtp_host, smtp_port, imap_host, imap_port, encryption_type, is_default, is_active, created_at, updated_at)
             VALUES ('acc-1', 'Main', 'main@example.com', 'smtp.example.com', 587, 'imap.example.com', 993, 'TLS', 1, 1, 100, 100)",
            [],
        );
        assert!(insert_res.is_ok());
    }

    #[test]
    fn test_passwords_table_initialization() {
        let conn = Connection::open_in_memory().unwrap();
        let init_res = init_passwords_table(&conn);
        assert!(init_res.is_ok());
    }
}
