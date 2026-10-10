use base64::prelude::*;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use super::*;

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
        // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
        crate::error::record_ignored(
            vault_conn.execute("UPDATE email_accounts SET is_default = 0", []),
            "db execute",
        );
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
        save_account_secret(&account_id, pwd)?;
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
        // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
        crate::error::record_ignored(
            pass_conn.execute(
                "DELETE FROM email_credentials WHERE account_id = ?",
                params![account_id],
            ),
            "db execute",
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

/// Get designated default email account (or first active account if none marked default)
pub fn get_default_account() -> Result<Option<EmailAccount>, String> {
    let accounts = list_email_accounts()?;
    let def = accounts
        .iter()
        .find(|a| a.is_default && a.is_active)
        .or_else(|| accounts.iter().find(|a| a.is_active))
        .cloned();
    Ok(def)
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

/// Save or update raw secret in separate split passwords database for account_id
pub fn save_account_secret(account_id: &str, plain_secret: &str) -> Result<(), String> {
    let has_content = !plain_secret.trim().is_empty();
    if has_content {
        let pass_conn = connect_passwords_db()?;
        let now = Utc::now().timestamp();
        let mut salt_bytes = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut salt_bytes);
        let salt = BASE64_STANDARD.encode(salt_bytes);

        let (enc_secret, fingerprint, ssh_pub) = encrypt_secret(plain_secret, &salt)?;

        pass_conn
            .execute(
                "INSERT INTO email_credentials 
                 (account_id, auth_type, encrypted_secret, rsa_public_fingerprint, ssh_rsa_public_key, salt, updated_at)
                 VALUES (?, 'password', ?, ?, ?, ?, ?)
                 ON CONFLICT(account_id) DO UPDATE SET
                    encrypted_secret = excluded.encrypted_secret,
                    rsa_public_fingerprint = excluded.rsa_public_fingerprint,
                    ssh_rsa_public_key = excluded.ssh_rsa_public_key,
                    salt = excluded.salt,
                    updated_at = excluded.updated_at",
                params![account_id, enc_secret, fingerprint, ssh_pub, salt, now],
            )
            .map_err(|e| format!("Failed to save credential in split passwords database: {}", e))?;
    }
    Ok(())
}
