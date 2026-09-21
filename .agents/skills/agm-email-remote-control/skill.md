---
name: agm-email-remote-control
description: Specialized skill for developing, maintaining, and debugging the inbound IMAP remote execution engine, outbound SMTP notifications, command parsing, and email receipts in Antigravity-Manager.
---

# AGM Inbound Email Remote Execution & SMTP Notifications

This skill guides engineering work on the email orchestration system in Antigravity-Manager. The engine polls incoming emails over IMAP, extracts structured commands, executes actions (prompt injection, CLI execution, instance rotation), and dispatches formatted HTML receipts via SMTP.

## Key Source Files

- `src-tauri/src/modules/email_inbound.rs` — IMAP polling loop, MIME parsing, action extraction (`InboundAction`), and remote execution bridge.
- `src-tauri/src/modules/email_sender.rs` — Outbound SMTP client, TLS stream negotiation, and HTML receipt generation.
- `src-tauri/src/modules/email_vault_db.rs` — SQLite storage for email accounts, polling intervals, and incoming execution audit trails.
- `src-tauri/src/modules/email_watcher.rs` — Background polling scheduler and error handling.
- `src-tauri/src/commands/email.rs` — Tauri IPC commands for testing email connections and querying inbound audit logs.

## Command Parsing Architecture (`InboundAction`)

Incoming email subjects and bodies are parsed into specific variants of `InboundAction`:
- `PromptInjection`: Dispatches a prompt to an active project workspace.
- `NamedPromptExecution`: Queries `repo.db` for matching prompt templates and triggers execution.
- `CliExecution`: Runs authorized terminal commands on specified targets, capturing standard output and error streams.
- `InstanceCreate`: Creates a new isolated Antigravity IDE profile.
- `AccountRotate`: Triggers proactive Google account rotation in `TokenManager`.
- `StatusQuery`: Queries live proxy telemetry, active account status, and quota percentages.

## Security & Operational Guardrails

### 1. Sender Allowlisting & Authorization
- Only pre-configured authorized sender email addresses stored in `email_vault.db` are permitted to execute commands.
- Unrecognized or forged sender addresses are discarded immediately and logged with `InboundAction::Ignored`.

### 2. Password Vault Isolation
- SMTP and IMAP passwords must never be stored in plain text.
- Passwords reside in the encrypted `email_passwords.db` managed by `security_db.rs`.

### 3. Execution Receipt Delivery
- After command processing, `email_sender.rs` formats a complete HTML report containing execution status, output snippets, duration, and timestamp, returning it to the sender.
