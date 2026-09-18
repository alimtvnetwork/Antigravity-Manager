# Subtask 06: Tauri IPC Commands & Error Handling Architecture

> **Parent Plan:** `.ai-memory/plans/pending/25-email-management-split-security-db-and-remote-control.md`
> **Specification:** `02-spec/21-app/16-email-dispatch-mailbox-remote-management-and-split-security-db.md`
> **Status:** Completed
> **Files:** `src-tauri/src/commands/email.rs`, `src-tauri/src/ipc.rs`

---

## Objective

Expose the complete suite of Tauri IPC commands for email management, connection testing, import/export, and watcher telemetry, fully integrated with `AppError` and the global error modal.

## Requirements

1. **IPC Commands:**
   - `get_email_settings` -> `Result<EmailNotificationSettings, AppError>`
   - `save_email_settings(settings: EmailNotificationSettings)` -> `Result<(), AppError>`
   - `list_email_accounts` -> `Result<Vec<EmailAccount>, AppError>`
   - `add_email_account(account: EmailAccountInput)` -> `Result<EmailAccount, AppError>`
   - `update_email_account(account: EmailAccountInput)` -> `Result<EmailAccount, AppError>`
   - `delete_email_account(id: String)` -> `Result<(), AppError>`
   - `set_default_email_account(id: String)` -> `Result<(), AppError>`
   - `list_notify_recipients` -> `Result<Vec<NotifyRecipient>, AppError>`
   - `add_notify_recipient(recipient: NotifyRecipientInput)` -> `Result<NotifyRecipient, AppError>`
   - `delete_notify_recipient(id: String)` -> `Result<(), AppError>`
   - `test_smtp_connection(account_id: String)` -> `Result<String, AppError>`
   - `test_imap_connection(account_id: String)` -> `Result<String, AppError>`
   - `export_email_data(format: String)` -> `Result<String, AppError>` (JSON, CSV, XLSX)
   - `import_email_data(format: String, payload: String)` -> `Result<ImportSummary, AppError>`
   - `get_email_watcher_status` -> `Result<WatcherStatus, AppError>`
   - `trigger_manual_email_check` -> `Result<String, AppError>`
2. **Error Management:**
   - Standard `AppError` wrapping with error codes `E8001` to `E8015`.
   - Preserve error messages, causes, and backtraces for frontend error modal.
