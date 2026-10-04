---
plan: 119-macos-installation-gatekeeper-quarantine-and-ide-switching
subtask: "004"
title: Backend macOS instance launching and IDE switching parity
domain: backend/instance
target_files:
  - src-tauri/src/modules/instance.rs
status: pending
---

# 004 — Backend macOS Instance Launching and IDE Switching Parity

## Scope
1. In `src-tauri/src/modules/instance.rs`:
   - In `launch_instance_inner_with_extra_workspaces`:
     - Distinguish between application bundles (`.app` directories) and standalone binaries or launcher shell scripts (`bin/antigravity-<id>`).
     - For `.app` bundles: execute via `open -n -a` with `format_macos_open_args`.
     - For launcher shell scripts: execute directly via `Command::new(&exe_str)` with `0o755` permissions, completely avoiding `open -a` rejection.
     - Implement complete environment parity with Windows and Linux:
       * Initialize `.gemini/antigravity-ide` and `.gemini/antigravity` in instance home.
       * Update `app_storage.json` setting `ide-install-wizard-shown: true` and bound account email.
       * Export environment variables (`HOME`, `SSH_*`, OAuth tokens).
       * Capture and log `Backtrace` on process launch errors.
