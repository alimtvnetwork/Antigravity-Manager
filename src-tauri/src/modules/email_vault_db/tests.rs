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
    pub(crate) fn test_passwords_table_initialization() {
        let conn = Connection::open_in_memory().unwrap();
        let init_res = init_passwords_table(&conn);
        assert!(init_res.is_ok());
    }
}
