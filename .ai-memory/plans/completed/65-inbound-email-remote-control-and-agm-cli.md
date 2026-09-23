# Plan 65: Inbound Email Remote Control, Plaintext Receipts, and Native AGM Terminal CLI

Spec Reference: [02-spec/21-app/19-inbound-email-remote-control-and-agm-cli.md](../../../02-spec/21-app/19-inbound-email-remote-control-and-agm-cli.md)
Status: COMPLETED
Completion Date: 2026-09-23
Execution Loops: 2

## 1. Architectural Context & How the Task Started

This task was initiated from user requirements and screenshot `https://prnt.sc/UX8ZI8H9Qm2h` requesting:
1. A unified pipe-delimited inbound email command grammar:
   `sub: [worker-name|ip] | [ins-{instance}] | (prompt|gitmap|cmd|update|ls|help|gitmap macro|gitmap update|agm update|agm status|agm instances|agm ls|agm ff/smart-switch|agy prompts ls|gitmap prompts ls) [ | proj-{project name} ]`
2. Two-phase plaintext notification receipts (Phase 1 Immediate ACK + Phase 2 Result) eliminating bloated HTML email templates.
3. A 10-second sliding debounce rate-limiting stack restricting rapid duplicate requests to at most 2 emails.
4. Sender authorization ACL rejecting unauthorized email senders without outbound replies.
5. The native `agm` terminal command-line interface (CLI) with `status`, `instances`/`ls`, `update`, `install` (PATH + PowerShell `$PROFILE`), `ff`/`smart-switch`, and `ssh` remote login / VM auto-update.
6. Consolidated Telegram settings in the Email & Alerts view with setup documentation and automated PowerShell script.
7. Local-only E2E permutation test suite dynamically reading SQLite DB credentials (zero secrets in git).

## 2. Completed Deliverables

### Task-01: Canonical Spec & Telegram Automation
- Authored canonical Spec 19: `02-spec/21-app/19-inbound-email-remote-control-and-agm-cli.md`.
- Authored Telegram Bot Setup Guide: `02-spec/21-app/telegram-bot-setup-guide.md`.
- Authored Telegram Setup Utility: `03-ai-scripts/setup-telegram-bot.ps1`.
- Registered entries in `02-spec/21-app/01-index.md`.

### Task-02: Native AGM Terminal CLI Engine (`agm`)
- Created native standalone executable `src-tauri/src/bin/agm.rs` and registered `[[bin]] name = "agm"` in `src-tauri/Cargo.toml`.
- Implemented subcommands:
  - `agm status`: Displays node telemetry, active account, proxy port, and IP.
  - `agm instances` / `agm ls`: Lists sandbox profiles, active paths, and running PIDs.
  - `agm ff` / `agm smart-switch`: Triggers manual account rotation.
  - `agm update`: Checks GitHub releases and downloads updates.
  - `agm install`: Installs into `%LOCALAPPDATA%\agm-cli\` and registers in PATH and PowerShell `$PROFILE`.
  - `agm ssh <[user@]host> [-p port] [--password <pwd>] [--update]`: SSH login with interactive password fallback (no terminal echo) and remote VM auto-update.
- Verified compilation and execution of all subcommands.

### Task-03: Inbound Email Pipe Grammar & 2-Phase Plaintext Receipts
- Updated `src-tauri/src/modules/email_inbound.rs`:
  - Implemented 2-to-4 segment pipe-delimited subject grammar parser.
  - Added partial IP octet matching (e.g. `12` matching local IP `192.168.1.12`).
  - Added structured body instruction extraction (`prompt-name:`, `prompt instruction:`).
  - Implemented Phase 1 Immediate ACK receipt (`[AGM ACK] COMMAND ACKNOWLEDGED AND RUNNING`).
  - Implemented Phase 2 Completion Result receipt (`[AGM Result] EXECUTION COMPLETED`).
  - Implemented 10-second sliding debounce rate-limiting stack.
  - Implemented sender ACL authorization check against `notify_recipients`.
- Updated `src-tauri/src/modules/email_sender.rs`:
  - Added dynamic content-type detection (`text/plain; charset=UTF-8` vs `text/html`).

### Task-04: Telegram Settings Consolidation in Email & Alerts UI
- Consolidated Telegram Bot Token (show/hide), Chat ID, polling interval, and connection test directly into `src/components/settings/EmailNotificationSettings.tsx`.
- Linked setup guide modal and instructions.
- Verified TypeScript (`npx tsc --noEmit`) and Vite build (`npm run build`).

### Task-05: Local-Only E2E Test Suite
- Authored `03-ai-scripts/40-test-email-permutations-e2e.py`.
- Grounded dynamically in local `~/.antigravity_tools/email_vault.db` (zero hardcoded secrets).
- Verified:
  - 9 Rust unit tests in `modules::email_inbound::tests` (100% pass).
  - All 19 subject command permutations (100% pass).
  - 2-Phase Plaintext Receipts formatting (100% pass).
  - 10-Second Debounce stack & ACL verification (100% pass).
  - Native AGM CLI commands (`--version`, `help`, `status`, `instances`) (100% pass).

### Task-06: Release Ceremony (`v4.62.0`)
- Bumped version to `v4.62.0` across all manifests and documentation.
- Updated `CHANGELOG.md` and `CHANGELOG_EN.md` with comprehensive release notes.
