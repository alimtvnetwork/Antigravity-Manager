# Subtask 02: Account Switching Process Termination & Stop Instance Repair

## Scope
- Files: `src-tauri/src/modules/instance/lifecycle.rs`, `src-tauri/src/modules/instance/process.rs`, `src-tauri/src/commands/instances.rs`

## Implementation Steps
1. In `src-tauri/src/modules/instance/lifecycle.rs` & `process.rs`:
   - Implement `kill_instance_process_tree(pid: u32) -> Result<(), AppError>` using OS-specific tree kill (`taskkill /F /T /PID` on Windows).
   - In `switch_account_for_instance()`: check if instance is running. If running or PID active, invoke `kill_instance_process_tree()` and verify exit before mutating credentials.
   - In `stop_instance()`: invoke `kill_instance_process_tree()`, wait for process exit, and set `is_running = false`, `pid = None` in the database.
2. In `src-tauri/src/commands/instances.rs`:
   - Route `stop_instance` command through the robust lifecycle kill pipeline.

## Verification
- Code compilation check and unit testing with mock PID.
