---
plan: 116-macos-installation-ide-detection-and-switching-fix
subtask: "002"
title: Fix macOS process launching argument formatting and expand IDE filesystem & Spotlight detection
domain: backend/rust
target_files:
  - src-tauri/src/modules/process.rs
status: pending
---

# 002 — Fix macOS Process Launching Argument Formatting and Expand IDE Filesystem & Spotlight Detection

## Scope
1. In `src-tauri/src/modules/process.rs`:
   - Fix `start_antigravity_with_fallback_path`:
     - Add `-n` flag to `open` command.
     - Properly format `--args` so `--new-window` is never passed as a raw option to `open`.
   - In `audit_standard_locations`:
     - Add checks for `/Applications/{name}.app/Contents/MacOS/{name}`, `Contents/MacOS/Antigravity`, and `$HOME/Applications/`.
     - Implement Spotlight `mdfind` query for Antigravity app bundles.
   - Maintain unit tests for path detection and argument formatting.
