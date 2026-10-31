# Subtask 02: Deterministic Process Termination & Stop Instance Fix

## Objectives
1. Implement the 5-stage progressive termination ladder in `src-tauri/src/modules/instance/close.rs`:
   - Stage 1: Lineage identification (root PID + child PIDs).
   - Stage 2: `taskkill /F /T /PID <root_pid>` with `CREATE_NO_WINDOW (0x08000000)`.
   - Stage 3: Fast poll verification (50ms interval, 1500ms max).
   - Stage 4: Force escalation fallback via direct Win32 `TerminateProcess` and PowerShell `Stop-Process -Force`.
   - Stage 5: Mandatory 200ms kernel settle delay and unconditional lockfile cleanup.
2. In `src-tauri/src/modules/instance/create.rs:stop_instance`, ensure `close_instance_verified` failures or stragglers are properly handled, SQLite `instance_processes` is updated to stopped state (`is_active = 0`, `pid = NULL`), caches are invalidated, and events are emitted.

## Owned Files
- `src-tauri/src/modules/instance/close.rs`
- `src-tauri/src/modules/instance/create.rs`

## Verification
- Rust clippy passes.
- E2E script confirms target PID is terminated and confirmed dead.
