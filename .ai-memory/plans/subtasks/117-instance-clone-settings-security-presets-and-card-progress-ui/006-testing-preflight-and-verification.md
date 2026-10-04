---
plan: 117-instance-clone-settings-security-presets-and-card-progress-ui
subtask: "006"
title: Testing, pre-flight validation, and verification
domain: test/preflight
target_files:
  - src-tauri/tests/instance_cloning_and_sync_test.rs
status: pending
---

# 006 — Testing, Pre-flight Validation, and Verification

## Scope
1. Update `src-tauri/tests/instance_cloning_and_sync_test.rs` to assert:
   - `resolve_instance_id` resolves `"default"`, sequence numbers, and UUIDs.
   - Cloning copies settings, keybindings, snippets, security presets, and workspaces.
2. Run pre-flight checks:
   - `cd src-tauri && cargo fmt -- --check`
   - `npm run build`
3. Verify zero absolute paths and perform atomic GitMap commit via `gitmap cpf`.
