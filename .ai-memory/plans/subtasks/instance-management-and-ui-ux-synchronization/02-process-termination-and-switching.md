# Subtask 02: Process Termination, Account Switch Sync & Audit Database Instrumentation

## Task Metadata
- **Code**: `Task-02`
- **Owner**: Worker 02
- **Status**: PENDING
- **Owned Files**:
  - `src-tauri/src/modules/audit_action.rs`
  - `src-tauri/src/modules/instance/close.rs`
  - `src-tauri/src/modules/instance/switch.rs`
  - `src-tauri/src/modules/instance/launch.rs`
  - `src-tauri/src/modules/integration/system_impl.rs`

## Requirements
1. **Extend `AuditAction` Enum**:
   - In `src-tauri/src/modules/audit_action.rs`:
     - Add `LaunchInstance = 6`, `StopInstance = 7`, `KillInstance = 8`, `CreateInstance = 9`, `CloneInstance = 10`, `DeleteInstance = 11`, `InstanceError = 12`.
     - Update `code()`, `pascal()`, `title()`, `from_code()`, and `from_legacy()`.
2. **Conscious Process Termination in `close.rs`**:
   - Normalize paths in `matches_instance`: handle 8.3 short paths, `ADMINI~1`, lowercase, and slash direction (`/` vs `\`).
   - If `pids.is_empty()` but `config.pid` or cached PID is alive, do not silently exit; verify and kill the root PID tree via `taskkill /F /T /PID <pid>`.
   - Prevent premature timeout: ensure kernel has time to release file locks.
3. **Account Switching Synchronization**:
   - In `src-tauri/src/modules/instance/switch.rs`: Ensure verified termination before rewriting credentials in `state.vscdb`.
   - In `src-tauri/src/modules/integration/system_impl.rs`: Replace bare `record_ignored` on `close_instance_verified` with verified closure before credential application.
4. **Audit Logging for Instance Lifecycle**:
   - Instrument `launch.rs` to record `AuditAction::LaunchInstance`.
   - Instrument `close.rs` / `create.rs` to record `AuditAction::StopInstance` / `KillInstance`.
