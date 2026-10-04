---
plan: 117-instance-clone-settings-security-presets-and-card-progress-ui
subtask: "002"
title: Backend complete user settings synchronization and security presets replication
domain: backend/settings
target_files:
  - src-tauri/src/modules/instance.rs
status: pending
---

# 002 — Backend Complete User Settings Synchronization and Security Presets Replication

## Scope
1. In `src-tauri/src/modules/instance.rs`:
   - Expand `copy_instance_settings` to deep-merge ALL keys from `settings.json` between source and target, not merely `workbench.*theme*` and `antigravity.*`. Preserve existing target identity/window title while copying user preferences, keybindings, and extensions settings.
   - Ensure `copy_instance_settings` also copies `keybindings.json`, `snippets/`, `security_presets.json`, and `antigravity_policies.json`.
   - In `copy_gemini_trees`:
     - If the source instance home does not yet contain `.gemini/policies` or `.gemini/config`, fall back to reading from `%USERPROFILE%/.gemini/` or default installation directories.
     - Ensure security presets and policy templates are copied into target data dir and target home roaming directories unconditionally.
