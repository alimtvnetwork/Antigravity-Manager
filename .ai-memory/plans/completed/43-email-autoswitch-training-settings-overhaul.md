# Consolidated Completed Plan: Email Verification, Auto-Switcher Hardening, Training API, and Settings UI/UX Overhaul

- **Task Reference:** 43-email-autoswitch-training-settings-overhaul
- **Spec Reference:** [02-spec/21-app/43-email-autoswitch-training-settings-overhaul.md](../../../02-spec/21-app/43-email-autoswitch-training-settings-overhaul.md)
- **Status:** COMPLETED
- **Steps Budget Taken:** 14 turns / 300 steps

---

## 1. Initial Prompt & Context

The user required four primary deliverables:
1. Confirm that inbound email command execution and threaded replies are working end-to-end (`VM3 | 1 | help` syntax, ACK and Result receipt delivery without RFC 5321 syntax error 501, verified via IMAP).
2. Root cause analysis and hardening for the auto-switcher (fixing self-exclusion logic, flash model name matching, and synchronizing global account IDs with instance profile swaps).
3. Implementation of a machine training and telemetry REST API engine (`/api/v1/training`), allowing external callers to seek/read telemetry, submit reinforcement learning feedback, and modify machine configurations, guarded by a settings toggle.
4. Settings UI/UX overhaul and tab/button minimization matching visual screenshot `assets/screenshots/settings-ui-overhaul-01.png`:
   - "Proxy Settings" -> "Proxy"
   - "Email & Alerts" -> "Email-Alerts"
   - "Supabase Sync" -> "Supabase"
   - "Backup & Restore" -> "Backup"
   - "Save Settings" -> "Save"
   - Remove standalone "Debug" tab from the settings tab row and consolidate into top header Bug icon and hamburger "More" dropdown.

---

## 2. Consolidated Subtasks & Technical Execution

### Subtask 01: Inbound Email Parsing, Threaded Replies, and Live E2E Verification
- **Traceability ID:** Task-01
- **Target Files:** `src-tauri/src/modules/email_inbound.rs`, `src-tauri/src/modules/email_sender.rs`, `src-tauri/src/bin/agm.rs`
- **Root Cause & Fixes:**
  - In `src-tauri/src/modules/email_sender.rs`, added `clean_recipient_email` to extract the bare email address from `Full Name <email@example.com>`, preventing RFC 5321 501 syntax error on SMTP `RCPT TO:<...>`.
  - Added RFC 2047 subject decoding and full support for 3-part subject syntax `<node> | <instance> | <command>` (e.g. `VM3 | 1 | help`).
  - Added IMAP tagged response reader `read_imap_tagged_response` to avoid partial reads and buffer desynchronization.
  - Added `In-Reply-To` and `References` headers for threaded message chaining.
- **Verification:**
  - Ran `agm test-email execute "VM3 | 1 | help"`. Phase 1 ACK receipt and Phase 2 Result receipt delivered to `devorg.bd@gmail.com` via SMTP port 465 SSL cleanly without error.

### Subtask 02: Auto-Switcher Candidate Scoring, Rotation & Cooldown Verification
- **Traceability ID:** Task-02
- **Target Files:** `src-tauri/src/modules/auto_switcher.rs`, `src-tauri/src/modules/instance.rs`, `src-tauri/src/bin/agm.rs`
- **Root Cause & Fixes:**
  - In `auto_switcher.rs:669` and `820`, `excluded_accounts` explicitly removed the current bound account from exclusion, causing rotation from a low-quota account back to the same account. Fixed by strictly keeping `bound_acc_id` inside `excluded_accounts`.
  - Flexible model matching: adjusted `contains("flash") && contains("gemini")` to catch actual model names like `gemini-3.8-flash-high`.
  - Synchronized global active account state in `instance::switch_account_to_instance` via `crate::modules::account::set_current_account_id(&account.id)`.
- **Verification:**
  - Ran `agm test-switcher 90.0` with `riseup.asia.team@gmail.com` (2% quota): successfully triggered critical quota rotation (<15%) to `shohagbazar004@gmail.com`.
  - Ran `agm test-switcher 80.0` with `shohagbazar004@gmail.com` (66% quota): successfully triggered standard low-quota rotation to `goqubadudexe77@gmail.com`.
  - Global `account::get_current_account()` and instance registry verified in sync.

### Subtask 03: Machine Training & Telemetry REST API Engine with Settings Toggle
- **Traceability ID:** Task-03
- **Target Files:** `src-tauri/src/modules/training_api.rs`, `src-tauri/src/proxy/server.rs`, `src-tauri/src/models/config.rs`, `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs`
- **Implementation:**
  - Added `src-tauri/src/modules/training_api.rs`:
    - `GET /api/v1/training/telemetry`: Node name (`VM3`), local IP, app version, accounts summary, healthy/depleted counts, instances, auto-switcher status.
    - `POST /api/v1/training/learn`: Ingests learning signals into SQLite `training_vault.db` under table `training_logs`, returning persistent log UUIDs and optionally adapting auto-switcher routing.
    - `POST /api/v1/training/machines`: Remotely modifies machine states, binds instances, adjusts thresholds, and switches accounts.
  - Added `training_api_enabled: bool` to `AppConfig` with 403 Forbidden protection when disabled.
  - Mounted routes on proxy server (port 8045) on both `/api/v1/training/*` and `/training/*`.
  - Added Tauri IPC commands `get_training_api_status`, `set_training_api_status`, `get_training_telemetry`.
- **Verification:**
  - Ran `agm test-training`: verified telemetry gathering, SQLite learning feedback persistence, dynamic threshold modification, and settings toggle.

### Subtask 04: Settings Menu UI/UX Overhaul & Tab/Button Minimization
- **Traceability ID:** Task-04
- **Target Files:** `src/pages/Settings.tsx`, `src/types/config.ts`, `src/locales/en.json`
- **Implementation:**
  - Renamed tabs:
    - "Proxy Settings" -> "Proxy"
    - "Email & Alerts" -> "Email-Alerts"
    - "Supabase Sync" -> "Supabase"
  - Shortened action buttons:
    - "Backup & Restore" -> "Backup"
    - "Save Settings" -> "Save"
  - Removed standalone "Debug" tab from the primary tab bar.
  - Added hamburger "More" dropdown menu (`Menu` icon) providing access to `Advanced`, `Debug`, and `About` with active indicators.
  - Added informative banner in the Debug view guiding users to the top header Bug icon for floating console overlay access.
  - Added "Machine Training REST API" toggle switch in Settings bound to `training_api_enabled`.
- **Verification:**
  - `npm run build` (`tsc && vite build`) passed with exit code 0.

---

## 3. Verification & Quality Gates Summary

- **Cargo Format:** `cargo fmt -- --check` passed cleanly (exit code 0).
- **Clippy Linter:** `cargo clippy --bin agm --lib` passed with 0 errors.
- **Frontend Build:** `npm run build` passed with 0 errors in 20.33s.
- **Live E2E Suites:**
  - `agm test-email execute "VM3 | 1 | help"` -> All ACK and Result receipts delivered.
  - `agm test-switcher` -> Rotated critical and low quota accounts without re-selection loops.
  - `agm test-training` -> Telemetry, learning ingestion, and machine modifications verified.
