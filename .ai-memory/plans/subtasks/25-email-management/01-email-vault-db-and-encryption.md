# Subtask 01: Split Email Vault Database (`email_vault.db`) & Asymmetric RSA Encryption

> **Parent Plan:** `.ai-memory/plans/pending/25-email-management-split-security-db-and-remote-control.md`
> **Specification:** `02-spec/21-app/16-email-dispatch-mailbox-remote-management-and-split-security-db.md`
> **Status:** Completed
> **Files:** `src-tauri/src/modules/email_vault_db.rs`

---

## Objective

Implement a dedicated split SQLite database (`email_vault.db`) using `rusqlite` for email accounts, separate credentials vault with RSA public key fingerprinting and salted irreversible key storage, notification recipients, notification settings, and inbound email audit logging.

## Requirements

1. **Split DB Isolation:**
   - Store `email_vault.db` in the application data directory (`crate::modules::account::get_data_dir()?.join("email_vault.db")`).
   - Configure WAL mode, `busy_timeout = 5000`, `synchronous = NORMAL`, and `foreign_keys = ON`.
2. **Tables:**
   - `email_accounts`: `id`, `alias`, `email`, `smtp_host`, `smtp_port`, `imap_host`, `imap_port`, `encryption_type`, `is_default`, `is_active`, `created_at`, `updated_at`.
   - `email_credentials`: `account_id` (FK cascade), `auth_type`, `encrypted_secret`, `rsa_public_fingerprint`, `salt`, `updated_at`.
   - `notify_recipients`: `id`, `email`, `group_name`, `is_active`, `created_at`.
   - `email_notification_settings`: `id` ('global'), `is_enabled`, `polling_interval_minutes`, `inbox_check_interval_minutes`, `notify_on_quota_drop`, `quota_drop_threshold_percent`, `notify_on_workspace_switch`, `notify_on_idle_workspace`, `allow_remote_prompt_execution`, `allow_remote_cli_execution`, `allow_remote_instance_rotation`, `local_machine_name`, `local_machine_ip`, `updated_at`.
   - `email_inbound_audit_log`: `id`, `message_id`, `sender_email`, `subject`, `action_type`, `action_payload`, `execution_status`, `execution_result`, `received_at`.
3. **Encryption Architecture:**
   - Password encryption using SHA-256 + salt + RSA token simulation or AES-GCM wrapping so plaintext passwords are never stored directly.
   - Return clean typed Rust structs with `serde::Serialize` / `Deserialize`.
   - Positive booleans only (`is_default`, `is_active`, `is_enabled`, etc.).
