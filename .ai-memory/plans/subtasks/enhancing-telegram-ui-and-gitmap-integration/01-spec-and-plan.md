# Subtask 01: Architecture Specification & Master Plan

> **Subtask Path:** `.ai-memory/plans/subtasks/enhancing-telegram-ui-and-gitmap-integration/01-spec-and-plan.md`  
> **Architecture Spec:** `02-spec/21-app/enhancing-telegram-ui-and-gitmap-integration/01-architecture-spec.md`  
> **Status:** COMPLETED  

---

## 1. Objective

Produce the complete architectural specification and disjoint execution plans for **Task 03: Enhancing Telegram UI and GitMap Integration**, covering:
1. Fixing the literal `&nbsp;&nbsp;` HTML entity rendering bug in `src-tauri/src/modules/notification_hub/telegram_alerts.rs` and attaching rich inline keyboard buttons to account switch and post-switch liveness alerts.
2. Enhancing the Telegram Bot UI (`src-tauri/src/modules/telegram_inbound/`) with a categorized click-to-copy `/help` manual, dedicated `/balance` (`/credits`, `/quota`) command, multipart CSV/JSON file export (`sendDocument`), and streamlined machine/instance prompt targeting.
3. Integrating GitMap multi-machine/multi-instance SSH control (`src-tauri/src/modules/ssh_manager/`, `src-tauri/src/bin/agm/`), automatic `gitmap` detection and PowerShell installer, and RSA-2048 envelope password encryption (`src-tauri/src/utils/crypto.rs` & `src-tauri/src/modules/ssh_manager/keys.rs`).

---

## 2. Owned Files

- `02-spec/21-app/enhancing-telegram-ui-and-gitmap-integration/01-architecture-spec.md`
- `.ai-memory/plans/subtasks/enhancing-telegram-ui-and-gitmap-integration/01-spec-and-plan.md`
- `.ai-memory/plans/subtasks/enhancing-telegram-ui-and-gitmap-integration/02-telegram-bot-and-csv-export.md`
- `.ai-memory/plans/subtasks/enhancing-telegram-ui-and-gitmap-integration/03-gitmap-ssh-rsa-and-cli.md`

---

## 3. Execution Decomposition & Agent Boundaries

### Agent 02 — Telegram Alert HTML Fix, `/balance`, Rich Keyboards & CSV/JSON Multipart Export
- **Subtask File:** `.ai-memory/plans/subtasks/enhancing-telegram-ui-and-gitmap-integration/02-telegram-bot-and-csv-export.md`
- **Owned Files:**
  - `src-tauri/src/modules/notification_hub/telegram_alerts.rs`
  - `src-tauri/src/modules/notification_hub/telemetry.rs`
  - `src-tauri/src/modules/telegram_inbound/formatting.rs`
  - `src-tauri/src/modules/telegram_inbound/media.rs`
  - `src-tauri/src/modules/telegram_inbound/reports_a.rs`
  - `src-tauri/src/modules/telegram_inbound/reports_b.rs`
  - `src-tauri/src/modules/telegram_inbound/processing.rs`
  - `src-tauri/src/modules/telegram_inbound/daemon.rs`
  - `src-tauri/src/modules/telegram_inbound/send_flow.rs`
  - `src-tauri/src/modules/telegram_inbound/tests.rs`

### Agent 03 — GitMap Multi-Machine SSH Control, RSA-2048 Password Vault & CLI Parity
- **Subtask File:** `.ai-memory/plans/subtasks/enhancing-telegram-ui-and-gitmap-integration/03-gitmap-ssh-rsa-and-cli.md`
- **Owned Files:**
  - `src-tauri/src/utils/crypto.rs`
  - `src-tauri/src/modules/ssh_manager/types.rs`
  - `src-tauri/src/modules/ssh_manager/keys.rs`
  - `src-tauri/src/modules/ssh_manager/connections.rs`
  - `src-tauri/src/modules/ssh_manager/sync.rs`
  - `src-tauri/src/modules/ssh_manager/exec.rs`
  - `src-tauri/src/modules/ssh_manager/bundles.rs`
  - `src-tauri/src/modules/ssh_manager/tests.rs`
  - `src-tauri/src/commands/fleet.rs`
  - `src-tauri/src/bin/agm/ssh_cmds.rs`
  - `src-tauri/src/bin/agm/ssh_handlers.rs`
  - `src-tauri/src/bin/agm/quota_cmds.rs`
  - `src-tauri/src/bin/agm/main.rs`
  - `src-tauri/src/lib.rs`

---

## 4. Acceptance Criteria

- [x] `02-spec/21-app/enhancing-telegram-ui-and-gitmap-integration/01-architecture-spec.md` includes `## User Request (Verbatim)` and `## Visual Evidence` embedding both `../../../assets/screenshots/enhancing-telegram-ui-and-gitmap-integration-01.png` and `../../../assets/screenshots/enhancing-telegram-ui-and-gitmap-integration-02.png`.
- [x] Subtask files define clear, non-overlapping file ownership boundaries and concrete implementation instructions using strictly relative repository paths.
