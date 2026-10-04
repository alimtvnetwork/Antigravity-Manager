---
name: agm-inbound-remote-control
description: Specialized skill for managing Antigravity-Manager remote execution daemons (Telegram Bot, IMAP Email), bidirectional command dispatching, sliding debounces, and multi-channel telemetry notifications.
---

# AGM Inbound Remote Control & Telemetry Hub Architecture

This skill provides comprehensive architectural guidance, command parsing rules, debounce mechanisms, and notification formatting standards for the Telegram Inbound, Email Remote Control, and Notification Hub subsystems in Antigravity-Manager.

---

## 1. Subsystem Architecture Overview

Antigravity-Manager enables secure out-of-band management of remote development nodes via bidirectional Telegram bot polling and IMAP email commands:

```
                               Remote User Interaction
                      +-------------------+-------------------+
                      |                                       |
                      v                                       v
            [Telegram Bot Messages]                  [Inbound IMAP Emails]
                      |                                       |
                      v                                       v
         telegram_inbound.rs                         email_inbound.rs
  - Polling loop (getUpdates)                - IMAP UNSEEN poller
  - Auth: allowed_chat_id                    - Sender Allowlist check
  - Command Registry (/status, /switch...)   - 10s Sliding Debounce Window
                      |                                       |
                      +-------------------+-------------------+
                                          |
                                          v
                              Command Execution Engine
                               src-tauri/src/modules/
             - Switch Profile & Rotate Credentials (auto_switcher.rs)
             - Inspect Workspace Prompts & Projects (repo_db.rs)
             - Retrieve Machine & Quota Telemetry (account.rs)
                                          |
                                          v
                               Unified Notification Hub
                        src-tauri/src/modules/notification_hub.rs
                                          |
                      +-------------------+-------------------+
                      |                                       |
                      v                                       v
           [Telegram Markdown Cards]                 [SMTP Outbound Receipts]
     - Account switch card                     - Two-Phase: [ACK] & [RESULT]
     - Low quota warnings (<= 25%)             - Standardized Subject Header:
     - Expand prompt preview (/expand)           [AGM vX.Y.Z | <host> | <ip>] ...
```

---

## 2. Key Files & Core Responsibilities

| File Path | Core Responsibilities |
|---|---|
| `src-tauri/src/modules/telegram_inbound.rs` | Long-polling daemon via `getUpdates`, chat ID authentication, command dispatcher, `/expand` prompt inspection, and observe status cards. |
| `src-tauri/src/modules/email_inbound.rs` | IMAP mail poller, sender address verification, pipe-delimited syntax parsing, and sliding 10-second debounce caching. |
| `src-tauri/src/modules/email_sender.rs` | Outbound SMTP dispatch via `lettre`, TLS configuration, standardized subject headers (`[AGM vX.Y.Z]`), and HTML formatting. |
| `src-tauri/src/modules/notification_hub.rs` | Central notification router broadcasting account switches, low quota warnings, and system update telemetry simultaneously to Email and Telegram. |

---

## 3. Telegram Bot Remote Commands

All incoming messages are validated against the configured `allowed_chat_id`. Valid commands include:

| Command | Arguments | Action |
|---|---|---|
| `/status` | None | Displays comprehensive telemetry: active account, rolling 5h & weekly quotas, proxy status, running instances. |
| `/rotate` | None | Triggers immediate smart account rotation based on candidate scoring. |
| `/switch` | `<email>` | Forces switch to a specific target account by email address. |
| `/prompts`| None | Lists running workspace prompts across all active IDE instances. |
| `/expand` | `<prompt_id>` | Displays the complete, untruncated prompt instruction text for a given ID. |
| `/projects`| None | Lists registered workspace projects and their last activity timestamps. |
| `/nodes`  | None | Reports cluster status and node aliases in multi-machine environments. |

---

## 4. Email Inbound Syntax & Sliding Debounce Protection

### Pipe-Delimited Command Syntax:
Emails sent to the monitored inbox are parsed using the format:
```
CMD | TARGET | PAYLOAD
```
Examples:
- `PROMPT | my-app | Fix CSS centering in navbar`
- `SWITCH | default | next`
- `STATUS | all | telemetry`

### Sliding 10-Second Debounce Protection (`check_debounce_rate_limit`):
Email clients frequently re-sync or re-send messages during network reconnections. To prevent duplicate execution:
- Computes a debounce key: `hash(sender + command + target)`.
- If an identical key is received within a **10-second sliding window**, the message is acknowledged as a duplicate and execution is skipped.

### Two-Phase Receipts:
1. **`[ACK]` Receipt**: Sent immediately upon receiving and validating the command.
2. **`[RESULT]` Receipt**: Sent upon completion with execution output or error details.

---

## 5. Standardized Email Subject Line Format

All automated emails dispatched by Antigravity-Manager **must** follow the canonical subject format:

```
[AGM vX.Y.Z | <machine_name> | <ip_address>] <Subject Description>
```
Example:
`[AGM v4.101.1 | Node-Primary | 192.168.1.50] Account Switched: user1@gmail.com -> user2@gmail.com`
