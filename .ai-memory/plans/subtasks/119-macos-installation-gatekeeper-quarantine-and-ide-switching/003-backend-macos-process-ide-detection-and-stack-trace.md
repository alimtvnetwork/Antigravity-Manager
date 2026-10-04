---
plan: 119-macos-installation-gatekeeper-quarantine-and-ide-switching
subtask: "003"
title: Backend macOS process execution argument ordering and stack trace capture
domain: backend/process
target_files:
  - src-tauri/src/modules/process.rs
status: pending
---

# 003 — Backend macOS Process Execution Argument Ordering and Stack Trace Capture

## Scope
1. In `src-tauri/src/modules/process.rs`:
   - Centralize `/usr/bin/open` argument formatting in `format_macos_open_args`.
   - Ensure all application arguments (including `--new-window` and `--user-data-dir`) strictly follow `--args` to prevent `open: unrecognized option` syntax errors.
   - Expand `get_macos_candidate_paths` to check `Contents/MacOS/` binaries and user `$HOME/Applications/`.
   - Integrate macOS Spotlight `mdfind` query fallback to locate Antigravity when installed in non-standard directories.
   - Capture Rust backtraces (`std::backtrace::Backtrace::capture()`) whenever process detection or execution fails.
