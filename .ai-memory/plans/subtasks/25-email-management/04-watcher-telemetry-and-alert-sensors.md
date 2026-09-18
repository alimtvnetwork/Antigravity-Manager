# Subtask 04: Background Watcher & Multi-Trigger Telemetry Sensor

> **Parent Plan:** `.ai-memory/plans/pending/25-email-management-split-security-db-and-remote-control.md`
> **Specification:** `02-spec/21-app/16-email-dispatch-mailbox-remote-management-and-split-security-db.md`
> **Status:** Completed
> **Files:** `src-tauri/src/modules/email_watcher.rs`

---

## Objective

Implement a background daemon in Rust that samples machine telemetry, account quotas, and active prompts on a configurable loop (default: 3 minutes) and triggers email notifications.

## Requirements

1. **Telemetry Capture:**
   - Query local machine hostname (`gethostname` or system env).
   - Query local network IP address (via standard socket connect / network interface enumeration).
   - Query GitMap CLI or Git identity when available.
2. **Sensor Triggers:**
   - **Trigger 1 (Quota Drop):** Inspect active profile quota (via `account.rs` / `auto_switcher.rs`). If quota drops below threshold (e.g. <15%), dispatch warning email to `notify_recipients`.
   - **Trigger 2 (Workspace Switch):** Send email notification when an auto-switch event occurs.
   - **Trigger 3 (Idle Running Projects):** Query `repo_db.rs` for running projects. If running projects exist but have zero active prompts in the prompt queue, send email asking: *"There is no prompt. Would you like to send something to this project? Available projects: [...]"*.
3. **Safety:**
   - Deduplicate alerts so spamming does not occur within cooldown intervals.
   - Thread-safe background execution with graceful stop mechanism.
