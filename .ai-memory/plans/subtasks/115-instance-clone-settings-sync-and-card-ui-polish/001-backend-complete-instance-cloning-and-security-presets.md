# Subtask 001: Backend Complete Instance Cloning and Security Presets

## Objective
Fix the instance cloning engine in `src-tauri/src/modules/instance.rs` so that duplicating an instance (from GUI, CLI, or tooling) reliably duplicates 100% of the source instance's:
1. `settings.json` (including all color themes, font settings, and Antigravity preferences)
2. `keybindings.json` and `snippets/`
3. Antigravity security presets (`security_presets.json`, `antigravity_policies.json`, and `.gemini/policies/`)
4. Opened projects & folders (`workspaceStorage/`)
5. Global storage states (`storage.json` recent paths, `state.vscdb` UI state while sanitizing only credentials)

## Target Files
- `src-tauri/src/modules/instance.rs`
- `src-tauri/src/modules/db.rs`

## Root Causes Addressed
1. `REQUIRED_IDE_REL_PATHS` lacked `security_presets.json` and `antigravity_policies.json`.
2. `GEMINI_CLONE_DIRS` lacked `policies` and `config`.
3. In `copy_dir_recursive`, unhandled error propagation with `?` caused an OS sharing violation (error 32) on an active process to immediately abort copying subsequent directories like `workspaceStorage/`.
4. `safe_clone_sqlite_db`: SQLite databases (`state.vscdb`, `workspaceStorage/.../state.vscdb`) locked with exclusive/WAL locks were failing `fs::copy`. Implement SQLite online backup or WAL-tolerant copy.
5. `copy_instance_settings` was never invoked from `copy_instance_with_options`. It must be called unconditionally so deep-merge configuration is written to all target settings files.
6. In `db::sanitize_session`, `antigravityUnifiedStateSync.enterprisePreferences` was wiped. Only wipe auth tokens (`oauthToken` and `userStatus`).

## Acceptance Criteria
- [x] Duplicating an instance copies `settings.json`, `keybindings.json`, `security_presets.json`, `antigravity_policies.json`, and `workspaceStorage/`.
- [x] Cloning works cleanly even if the source instance IDE is actively running with open file locks.
- [x] Theme, security policies, and workspace history are retained in the cloned instance.
