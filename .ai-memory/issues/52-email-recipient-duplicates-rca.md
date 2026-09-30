# Issue 52: Notification recipient emails are duplicated on import

Status: open (planned, not fixed)
Raised: 2026-09-30
Spec: [02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md](../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md)
Plan: [88](../plans/pending/88-email-recipient-dedupe-supabase-push-settings-and-vault-scripts.md)

## 1. Symptom
Importing or adding an email that is already a notification recipient adds it again.

## 2. Trigger
Email import, CLI add, GUI add, or bundle import.

## 3. Root cause
No normalized uniqueness on the recipients table and every path inserts independently (`email_vault_db.rs::add_notify_recipient`, `email_io.rs`, `commands/email.rs`, `agm.rs`, `EmailNotificationSettings.tsx`). Spec 74 deduplicated display only.

## 4. Why it escaped
Dedupe was verified in the UI list, not at the storage layer.

## 5. Fix (planned)
One upsert-or-skip helper on `LOWER(TRIM(email))`, a unique index migration, every path routed through it, import summary with skipped count, optional `agm email dedupe`. See plan 88 subtasks 001-003.

## 6. Prevention
Unique index makes duplicates impossible regardless of caller.

## 7. Regression check
`cargo test modules::email_vault_db`: same email in different case and with spaces yields one row.
