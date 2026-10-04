---
plan: 116-macos-installation-ide-detection-and-switching-fix
subtask: "003"
title: Ensure macOS instance launching parity for shell scripts vs bundles and add missing app storage setup
domain: backend/rust
target_files:
  - src-tauri/src/modules/instance.rs
status: pending
---

# 003 — Ensure macOS Instance Launching Parity for Shell Scripts vs Bundles and Add Missing App Storage Setup

## Scope
1. In `src-tauri/src/modules/instance.rs`:
   - In `launch_instance_inner_with_extra_workspaces` on macOS:
     - Differentiate between `.app` directory bundles and executable shell scripts / Mach-O binaries.
     - Never pass shell scripts to `open -a`; execute them directly with `Command::new`.
     - Implement the missing initialization: `.gemini` home folders, `ide-install-wizard-shown: true`, and `update_instance_app_storage`.
   - In `clone_instance_executable`:
     - Ensure generated launcher scripts have valid shebangs, execution permissions, and pass arguments correctly.
