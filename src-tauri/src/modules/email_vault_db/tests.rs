use base64::prelude::*;
use rusqlite::{params, Connection, OptionalExtension};

use super::*;

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    pub(crate) fn test_encryption_roundtrip() {
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
    pub(crate) fn test_vault_table_initialization() {
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
    pub(crate) fn test_recipient_add_skips_duplicates() {
        let conn = Connection::open_in_memory().unwrap();
        init_vault_tables(&conn).unwrap();

        let first = add_notify_recipient_with_conn(
            &conn,
            NotifyRecipientInput {
                email: "  User@Example.com ".to_string(),
                group_name: Some("alerts".to_string()),
                is_active: Some(true),
            },
        )
        .unwrap();
        assert!(first.is_added());

        // Same address, different case and surrounding spaces: skipped.
        let second = add_notify_recipient_with_conn(
            &conn,
            NotifyRecipientInput {
                email: "user@example.com".to_string(),
                group_name: Some("other".to_string()),
                is_active: Some(false),
            },
        )
        .unwrap();
        assert!(!second.is_added());
        // Existing row is returned unchanged (no group or active-flag overwrite).
        assert_eq!(second.recipient().group_name, "alerts");
        assert!(second.recipient().is_active);

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM notify_recipients", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    pub(crate) fn test_merge_duplicate_recipients() {
        let conn = Connection::open_in_memory().unwrap();
        // Build the table without init-time dedupe so duplicates can exist.
        conn.execute(
            "CREATE TABLE notify_recipients (
                id TEXT PRIMARY KEY,
                email TEXT NOT NULL,
                group_name TEXT NOT NULL DEFAULT 'default',
                is_active INTEGER NOT NULL DEFAULT 1,
                created_at INTEGER NOT NULL
            )",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO notify_recipients (id, email, group_name, is_active, created_at) VALUES ('a', 'User@Example.com', 'g1', 0, 100)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO notify_recipients (id, email, group_name, is_active, created_at) VALUES ('b', 'user@example.com', 'g2', 1, 200)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO notify_recipients (id, email, group_name, is_active, created_at) VALUES ('c', 'solo@example.com', 'g1', 1, 300)",
            [],
        )
        .unwrap();

        let mut conn = conn;
        let groups = find_duplicate_recipients_with_conn(&conn).unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].row_count, 2);
        assert_eq!(groups[0].keeper_id, "a");

        let deleted = merge_duplicate_recipients_with_conn(&mut conn).unwrap();
        assert_eq!(deleted, 1);

        // Oldest row kept, promoted to active because a duplicate was active.
        let keeper: (String, i64) = conn
            .query_row(
                "SELECT id, is_active FROM notify_recipients WHERE LOWER(TRIM(email)) = 'user@example.com'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(keeper.0, "a");
        assert_eq!(keeper.1, 1);

        let total: i64 = conn
            .query_row("SELECT COUNT(*) FROM notify_recipients", [], |r| r.get(0))
            .unwrap();
        assert_eq!(total, 2);
    }

    #[test]
    pub(crate) fn test_passwords_table_initialization() {
        let conn = Connection::open_in_memory().unwrap();
        let init_res = init_passwords_table(&conn);
        assert!(init_res.is_ok());
    }
}
