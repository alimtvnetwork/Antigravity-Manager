# Consolidated Plan 59: Instance PID Tracking, Quota Polling Ladder, Email Task UI & Universal JSON Settings

> **Execution Lifecycle:**
> - Started: User request for per-instance PID tracking in SQLite DB to eliminate collateral kills during profile switching, CLI fast-forward flag (`--fast-forward`), dynamic credit threshold polling ladder (<20% @ 3m, <=12% @ 1m) with auto-switch, Developer Task Quick Dispatch UI in Email Notification Recipients card, and universal JSON settings import/export with multi-pass base64 credential encryption.
> - Completed in: Continuous loop.
> - Total Subtasks Completed: 5/5
> - Verification Gates: CI/CD local runner passed (38/38 gates in 8.63s), rustfmt invariant preserved on `supabase_command_queue.rs:98`.

---

## 1. Executive Summary

This implementation delivers robust multi-instance process isolation, credit quota monitoring, automated failover, and email developer controls:

1. **Instance Process PID Tracking & Selective Process Termination (`instance.rs`, `instances.db`):**
   - Added `pid: Option<u32>` to `InstanceConfig` and initialized SQLite `instances.db` table `instance_processes`.
   - On spawn via `launch_instance`, captures `child.id()` and persists root PID into SQLite and memory registry.
   - On profile switch or close via `close_instance`, resolves target PID and child tree, terminating only matching processes (`taskkill /PID <pid> /T /F` on Windows; `kill -15` / `kill -9` on Unix).
   - Completely eliminated collateral kills of sibling Antigravity editor instances running under different `--user-data-dir` paths.
   - Removed destructive host-wide `switchAccount` call from `smartRotateProfileAccount` in `useInstanceStore.ts`.

2. **Terminal CLI Headless Fast-Forward:**
   - Added `--fast-forward` / `-ff` / `fast-forward` subcommands to `cli.rs`.
   - Automatically resolves next best healthy candidate profile and triggers background rotation headlessly via Tokio async runtime.

3. **Dynamic Credit Quota Polling Ladder & Auto Fast-Forward (`auto_switcher.rs`):**
   - Normal Tier (credits >= 20%): Standard configured check interval (15–30m default).
   - Caution Tier (credits < 20%): Accelerated polling down to 3 minutes (180s default, configurable).
   - Critical Tier (credits <= 12%): Emergency polling down to 1 minute (60s default, configurable).
   - Auto Fast-Forward Trigger: At critical quota (<= 12%), automatically executes fast-forward rotation to the highest-credit candidate, respecting unleased Supabase cluster nodes.
   - Added UI interval sliders and toggle in `AutoSwitcherSettings.tsx`.

4. **Email Developer Task Quick Dispatch & Universal JSON Settings (`EmailNotificationSettings.tsx`, `email_io.rs`):**
   - Redesigned `[Send] Notification Recipients` card in `EmailNotificationSettings.tsx`:
     - Added Developer Task Quick Dispatch section with dropdown supporting `prompt:`, `powershell:`, `cmd:`, `gitmap:`, and `status:` tasks.
     - Added target recipient selector (broadcast or individual recipient), custom subject line, and payload editor.
     - Direct "Dispatch Task" action button with live loading spinner calling backend IPC command `dispatch_custom_email_task`.
     - Added Adaptive Monitoring & Retry Interval sliders (5–10 min initial wait, 5–10 sec fast-poll) with "Save Monitoring Intervals" button.
   - Universal JSON settings import/export in `email_io.rs`:
     - Implemented `base64_encode_multi(secret, 3)` and `base64_decode_multi` to obfuscate and protect passwords/tokens across import/export cycles.
     - Added `ExportedCredential` to `EmailExportBundle` for lossless export and re-import into `email_vault_db`.

5. **CI/CD Quality Gates & Unit Tests:**
   - Preserved `supabase_command_queue.rs:98` single-line formatting invariant for `cargo fmt`.
   - Added unit test in `instance.rs` verifying SQLite `instance_processes` PID persistence, query, and status update.
   - Added unit test in `email_io.rs` verifying 3-pass and 1-pass base64 roundtrip and invalid input handling.
   - Added unit test in `auto_switcher.rs` verifying dynamic polling ladder step-downs across normal, caution, and critical quotas.
   - Verified local CI runner passing 38 of 38 quality gates.

---

## 2. Completed Subtasks Log

### Subtask 01: Instance Process PID Tracking & Selective Process Termination
- **Target Files:** `src-tauri/src/models/instance.rs`, `src-tauri/src/modules/instance.rs`
- **Delivered:** `pid: Option<u32>` in `InstanceConfig`, SQLite DB `instance_processes` table creation, `record_instance_pid`, `get_instance_saved_pid`, `mark_instance_stopped`, and selective `taskkill /PID <pid> /T /F`.

### Subtask 02: Robust Instance Switching & Terminal CLI Fast-Forward Support
- **Target Files:** `src/stores/useInstanceStore.ts`, `src-tauri/src/modules/cli.rs`
- **Delivered:** Isolated profile switching without collateral kills, added `--fast-forward` / `-ff` CLI command.

### Subtask 03: Multi-Tier Credit Threshold Polling Ladder & Cluster Auto-Switch
- **Target Files:** `src-tauri/src/models/config.rs`, `src-tauri/src/modules/auto_switcher.rs`, `src/types/config.ts`, `src/services/instanceService.ts`, `src/components/settings/AutoSwitcherSettings.tsx`
- **Delivered:** `calculate_next_interval_seconds` dynamic ladder, auto fast-forward trigger on <= 12% quota, caution & critical sliders in UI.

### Subtask 04: Email Recipients UI Enhancement, Task Dispatch Dropdown & Universal JSON Settings Import/Export
- **Target Files:** `src-tauri/src/modules/email_io.rs`, `src-tauri/src/commands/email.rs`, `src-tauri/src/lib.rs`, `src/services/emailService.ts`, `src/components/settings/EmailNotificationSettings.tsx`
- **Delivered:** Multi-pass base64 secret encryption in JSON exports, `dispatch_custom_email_task` backend IPC command, expanded `Notification Recipients` card with Developer Task Quick Dispatch, adaptive monitoring sliders, and save action.

### Subtask 05: Rustfmt Formatting Parity & Edge-Case Test Suite Verification
- **Target Files:** `src-tauri/src/modules/supabase_command_queue.rs`, `src-tauri/src/modules/instance.rs`, `src-tauri/src/modules/email_io.rs`, `src-tauri/src/modules/auto_switcher.rs`
- **Delivered:** Fixed single-line rustfmt check, added unit tests across all new modules, verified green across 38 CI quality gates.
