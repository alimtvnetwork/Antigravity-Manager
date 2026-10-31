# Subtask 03: Account Switch Synchronization & Pre-Switch Eviction

## Objectives
1. In `src-tauri/src/modules/instance/switch.rs`, enforce pre-switch PID eviction:
   - Check if the target instance is actively running or has an active PID.
   - Forcefully terminate the running PID before credential injection.
   - Enforce 200ms kernel settle delay to avoid Windows NTFS file locks.
   - Clear stale Chromium lockfiles (`lockfile`, `code.lock`, `DevToolsActivePort`, `Singleton*`).
2. Atomically inject account credentials into `state.vscdb` and update `storage.json` with retry logic.
3. Relaunch instance and poll for the newly materialized PID, recording it in `instance_processes` and `registry.json`.

## Owned Files
- `src-tauri/src/modules/instance/switch.rs`
- `src-tauri/src/modules/instance/credentials.rs`

## Verification
- Switch account test executes without `database locked` errors and binds new account.
