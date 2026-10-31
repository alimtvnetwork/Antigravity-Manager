# Subtask 02: Hardened Process Termination & Win32 Fallback Ladder

**Parent Plan:** `.ai-memory/plans/instance-management-and-ui-ux-synchronization.md`  
**Status:** `IN PROGRESS`  
**Owner:** Worker 02 (Rust Backend)  
**Exclusive Files:**  
- `src-tauri/src/modules/instance/close.rs`  
- `src-tauri/src/modules/instance/create.rs`  
- `src-tauri/src/commands/instance.rs`  

## Objectives & Deliverables
1. Implement `force_terminate_pid_win32(pid)` in `close.rs` using direct Win32 `OpenProcess(PROCESS_TERMINATE, 0, pid)` and `TerminateProcess(handle, 1)`.
2. Fix contract violation in `close_instance`: do NOT prematurely abort with `return Err(...)` after 3000ms. Instead, run the Win32 `TerminateProcess` ladder on surviving PIDs so `close_instance_verified` can complete its verification poll loop.
3. In `stop_instance`, ensure that when verified closed, instance `pid` is cleared to `None` in registry / status and returns `Ok(())`.
