---
name: email-management
description: Autonomously manage email dispatch, split security vault databases, mailbox failover swapping pools, multi-trigger background sensors, and bidirectional remote mailbox control.
---

# Email Management & Mailbox Remote Automation

> **Scope:** Multi-Account Email Dispatch, Split Vault DB, Failover Swapping, and Inbound Mailbox Control
> **Specification Reference:** `02-spec/21-app/16-email-dispatch-mailbox-remote-management-and-split-security-db.md`

## Core Architectural Pillars

### 1. Split Database & Asymmetric RSA Key Storage

- **Physical Database Isolation:**
  - `email_vault.db`: Stores non-sensitive metadata including account configurations (aliases, SMTP/IMAP hosts, ports, encryption types, default/active flags), notification recipients, notification settings, and inbound command audit logs.
  - `email_passwords.db`: Completely isolated split database storing credential records (`account_id`, `auth_type`, `encrypted_secret`, `rsa_public_fingerprint`, `ssh_rsa_public_key`, `salt`, `updated_at`). Plaintext passwords are never stored in the vault.
- **SSH RSA Representation:**
  - Generates OpenSSH formatted public key identities (`ssh-rsa AAAAB3NzaC1yc2E... antigravity@node`) alongside SHA-256 fingerprints to satisfy audit requirements and prevent key reversal.
- **Pragmas:** Both databases operate with SQLite WAL mode, `busy_timeout = 5000`, `synchronous = NORMAL`, and foreign keys enabled.

### 2. Two-Way Import & Export Engine

- **JSON Format:** Complete serialization of accounts, recipient groups, and watcher sensor thresholds.
- **CSV Format:** Clean two-section CSV structure (`# SECTION: ACCOUNTS` and `# SECTION: RECIPIENTS`) with standard RFC 4180 quotation handling.
- **Excel Spreadsheet Format:** Native Microsoft Excel XML Spreadsheet 2003 (`.xml` / `.xlsx`) supporting `<Worksheet ss:Name="Mailboxes">` and `<Worksheet ss:Name="Recipients">`.
- **SQLite Database Backup/Restore:** Direct atomic file copying of `email_vault.db` for zero-data-loss system migration.

### 3. Outbound Mailer & Mailbox Pool Swapping

- **Default Sender Prioritization:** Messages are dispatched primarily through the designated default mailbox account.
- **Automated Failover Swapping:** If the default mailbox encounters an SMTP transport failure, timeout, or authentication issue, the sender automatically cycles through all remaining active mailboxes in the pool until delivery succeeds.
- **Anti-Spam Formatting:** Headers are RFC 2047 utf-8 encoded, and message bodies provide responsive HTML styling with clean typography and plain-text multipart fallbacks.

### 4. Background Watcher Daemon & Sensors

- **Configurable Polling Loop:** Default 3 minutes (configurable from 1 to 5 minutes).
- **Machine Telemetry:** Detects and embeds node hostname and local network IP (discovered via socket routing) in all alerts and receipts.
- **Sensor Triggers:**
  - **Quota Drop Alert:** Dispatches warning when active profile quota drops below 15%.
  - **Workspace Switch Notice:** Dispatches notification prior to automated profile rotation.
  - **Idle Workspace Sensor:** Inspects `repo_db::list_running_projects()`. If projects are running but no prompts are executing in the queue, dispatches an idle alert prompting the user to reply with instructions.

### 5. Inbound Mailbox Reader & Bidirectional Remote Execution

- **IMAP Poller:** Checks the last 5 unread instructions via IMAP on a configurable interval (default: 1 minute).
- **Reply Prefix Handling:** Automatically strips `Re: ` and `Fwd: ` prefixes from email client replies, falling back to body matching if needed.
- **Supported Remote Commands:**
  - `Project: <project-name>`: Injects user reply prompt into the running project workspace in Antigravity.
  - `exec: <ip>`: Verifies that `<ip>` matches the machine's local IP address before executing approved shell/GitMap instructions, formatting stdout/stderr into an HTML log.
  - `instance: new`: Dynamically spawns an isolated IDE instance profile.
  - `rotate: accounts`: Triggers instant profile rotation to the next highest-quota account.
  - `status`: Replies with running projects and prompt queue telemetry.
  - `help`: Replies with a complete HTML command cheat sheet.
- **Bidirectional Acknowledgment Receipts:** Every inbound command automatically emails back a styled HTML confirmation receipt detailing execution status and output.

### 6. Error Management & UI Guidelines

- **AppError Envelope:** All backend operations return `AppResult<T>` with `AppError::Email(String)` and error code `E8002`.
- **Global Error Modal:** Frontend service methods wrap all invocations in try/catch blocks that invoke `useErrorStore.getState().captureError(e, ...)` so errors can be inspected and copied.
- **Boolean Guidelines:** Strict implicit evaluation (`if is_active`), positive prefixes, and zero mixed polarity (`&& !`).
