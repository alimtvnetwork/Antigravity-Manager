---
plan: 119-macos-installation-gatekeeper-quarantine-and-ide-switching
subtask: "005"
title: First-time IDE discovery, background integration, and diagnostic persistence
domain: backend/telemetry
target_files:
  - src-tauri/src/lib.rs
  - src-tauri/src/modules/process.rs
status: pending
---

# 005 — First-Time IDE Discovery, Background Integration, and Diagnostic Persistence

## Scope
1. In `src-tauri/src/modules/process.rs`:
   - In `discover_and_persist_initial_ide_info()`:
     - Check if `antigravity_executable` in `gui_config.json` is already valid and existing.
     - If unconfigured, run `detect_antigravity_with_diagnostics(None)`.
     - Automatically persist discovered path to `gui_config.json` via `crate::modules::config::save_app_config`.
     - On failure, capture `std::backtrace::Backtrace::capture()`, log comprehensive diagnostic checklist of inspected paths.
2. In `src-tauri/src/lib.rs`:
   - Call `crate::modules::process::discover_and_persist_initial_ide_info()` on startup (in spawned background thread) to ensure the first-time setup gathers IDE information automatically.
