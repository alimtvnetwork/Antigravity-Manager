# Subtask 01: App Close Full Process Termination Fix

## Context
When closing Antigravity Manager via window X, Alt+F4, or UI buttons, the app must completely terminate from Windows Task Manager.

## Files to Modify
- `src-tauri/src/modules/process/lifecycle.rs` (or `process.rs`)
- `src-tauri/src/lib/setup_app.rs`
- `src-tauri/src/lib/run.rs`
- `src-tauri/src/commands/cmd_window.rs`

## Acceptance Criteria
1. Closing the main window initiates `exit_app_fully`.
2. `api.prevent_close()` and `api.prevent_exit()` are no longer called when user closes the app.
3. Background services (cloudflared, admin server, proxy, tokens) gracefully clean up within 500ms.
4. Process calls `std::process::exit(0)` ensuring zero residual processes in Windows Task Manager.
