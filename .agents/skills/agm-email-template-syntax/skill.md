---
name: agm-email-template-syntax
description: Specialized skill for managing email multi-format export/import (JSON/YAML/CSV), native OS save dialog filters, embedded AI syntax, and inbound IMAP prompt interception in Antigravity-Manager.
---

# AGM Email Multi-Format IO & Inbound Prompt Interception Engine

This skill governs the email configuration import/export subsystem, native OS dialog filtering, embedded AI prompt syntax, and IMAP inbound prompt execution in Antigravity-Manager.

---

## 1. Visual Export Modal & Multi-Format Serializers

### 1.1 `MailboxExportModal.tsx`
- Replaced silent background downloads with an interactive visual modal.
- Offers three live syntax-highlighted format tabs: **JSON**, **YAML**, and **CSV**.
- Supports single-mailbox export (via table row actions) or full-pool export.
- Features one-click clipboard copying and native OS file downloads.

### 1.2 Serialization Engines (`src/utils/emailFormatters.ts`)
- **JSON**: Formatted with 2-space indentation.
- **YAML**: Clean nested key-value representation without unnecessary quotation marks.
- **CSV**: RFC 4180 compliant with standard headers (`email`, `smtp_host`, `smtp_port`, `imap_host`, `imap_port`, `username`, `is_default`, `use_tls`).
- **Native File Dialog Filters**:
  - `yaml` maps to `['yaml', 'yml']`.
  - `xlsx` maps to `['xlsx', 'xls']`.
  - Backend path validator `validate_user_json_path` in `src-tauri/src/commands/mod.rs` permits `.json`, `.csv`, `.yaml`, `.yml`, `.txt`, `.xlsx`, `.xls` while blocking directory traversal and system directories.

---

## 2. Inbound IMAP Execution & Prompt Interception

### 2.1 Shell Prefix Stripping
In `src-tauri/src/modules/email_inbound.rs`:
- Automatically strips common shell command prefixes:
  - `powershell:`, `ps:`, `ps `, `pwsh:`
  - `bash:`, `sh:`, `cmd:`
- On Windows, executes via `powershell.exe -NoProfile -NonInteractive -WindowStyle Hidden -ExecutionPolicy Bypass`.

### 2.2 Natural Language Prompt Interception
- When an inbound email subject or body contains prompt markers (`Project:`, `Prompt:`, `AI:`, `Instruction:`), the engine routes the payload to `InboundAction::NamedPromptExecution`.
- This injects the instruction into the active workspace prompt queue rather than executing it as raw shell commands, preventing PowerShell parse errors.

### 2.3 Two-Phase Receipts & Plaintext Delivery
- **Phase 1 (ACK)**: Sends immediate ASCII acknowledgment receipt confirming receipt and node assignment.
- **Phase 2 (Result)**: Executes task, captures stdout/stderr/exit code, and sends full result receipt.
- Delivery is 100% plaintext ASCII to avoid HTML email formatting artifacts and spam filters.
