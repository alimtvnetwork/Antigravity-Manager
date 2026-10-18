# Subtask 02: Process Termination & Stop Instance Fix

- **Slug:** `instance-management-and-ui-ux-synchronization`
- **Subtask ID:** `02-process-termination-stop-instance`
- **Parent Plan:** `.ai-memory/plans/instance-management-and-ui-ux-synchronization.md`
- **Status:** `PENDING`

## Objective
Ensure that clicking the "Stop Instance" button reliably kills the running instance process tree across platforms (Windows, macOS, Linux).

## Scope of Changes
- Trace `stop_instance` Tauri IPC command and underlying `process_manager` / `instance_manager` implementations.
- Implement robust conscious process termination ladder (`WM_CLOSE` / `SIGTERM` -> `taskkill /T /F /PID` or Unix `kill -9` -> verification of PID absence).
- Handle edge cases where recorded PID in SQLite diverges from actual running process or child process hierarchy.
- Ensure database state transitions to `stopped` with `pid = None` immediately upon termination.

## Verification
- Running instance terminates within 2 seconds of invoking stop.
- No zombie/orphaned processes left behind.
- Rust unit tests and preflight checks pass cleanly.
