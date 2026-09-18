# Subtask 05: Inbound IMAP Mailbox Poller & Bidirectional Remote Execution Bridge

> **Parent Plan:** `.ai-memory/plans/pending/25-email-management-split-security-db-and-remote-control.md`
> **Specification:** `02-spec/21-app/16-email-dispatch-mailbox-remote-management-and-split-security-db.md`
> **Status:** Completed
> **Files:** `src-tauri/src/modules/email_inbound.rs`

---

## Objective

Build an inbound email poller checking the last 5 unread instructions via IMAP on a configurable interval (default: 1 min), with deterministic command parsing and remote action dispatch.

## Requirements

1. **IMAP Poller Engine:**
   - Connect to IMAP host/port with TLS/SSL.
   - Fetch unread messages (cap to last 5).
   - Record received messages in `email_inbound_audit_log` to prevent double-execution.
2. **Deterministic Command Parser:**
   - **Prompt Injection:**
     - Subject: `Project: <project-name>` or `project-prompt: <name>` (case-insensitive prefix match).
     - Body: The multi-line prompt text.
     - Action: Injects prompt into matching active running project in `repo_db.rs` / Antigravity workspace.
   - **Command Execution:**
     - Subject: `exec: <ip>` or `command: <ip>`.
     - Action: If local machine IP matches, execute approved command (e.g., `gitmap` status/command or CLI), capture stdout/stderr, and email back HTML report.
   - **Instance Creation:**
     - Subject: `instance: new` or `instance: create`.
     - Action: Triggers new IDE instance profile clone/launch.
   - **Account Rotation:**
     - Subject: `rotate: accounts` or `account: rotate`.
     - Action: Triggers instant account rotation to the next highest-quota profile.
   - **Help / Cheat Sheet:**
     - Subject: `help`.
     - Action: Emails back rich HTML cheat sheet with examples for all supported email commands.
