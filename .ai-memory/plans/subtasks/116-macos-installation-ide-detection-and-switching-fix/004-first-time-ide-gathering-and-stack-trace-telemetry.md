---
plan: 116-macos-installation-ide-detection-and-switching-fix
subtask: "004"
title: Implement first-time IDE information gathering and stack trace capture across macOS operations
domain: backend/rust
target_files:
  - src-tauri/src/modules/process.rs
  - src-tauri/src/lib.rs
status: pending
---

# 004 — Implement First-Time IDE Information Gathering and Stack Trace Capture Across macOS Operations

## Scope
1. Implement `discover_and_persist_initial_ide_info()`:
   - On application startup or first run, inspect running processes, standard directories, and Spotlight.
   - If a valid Antigravity IDE or Classic installation is found and `antigravity_executable` is empty in `gui_config.json`, auto-populate and save it.
   - Log complete diagnostics and capture backtraces with `std::backtrace::Backtrace::capture()`.
2. Integrate into `src-tauri/src/lib.rs` startup flow.
