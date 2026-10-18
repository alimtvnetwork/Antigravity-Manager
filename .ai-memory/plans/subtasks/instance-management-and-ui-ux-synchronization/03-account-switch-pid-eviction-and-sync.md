# Subtask 03: Account Switch Pre-Switch Process Eviction & Synchronization

**Parent Plan:** `.ai-memory/plans/instance-management-and-ui-ux-synchronization.md`  
**Status:** `IN PROGRESS`  
**Owner:** Worker 02 (Rust Backend)  
**Exclusive Files:**  
- `src-tauri/src/modules/instance/switch.rs`  
- `src-tauri/src/modules/integration/system_impl.rs`  

## Objectives & Deliverables
1. In `switch.rs`:
   - Before rewriting credentials in `state.vscdb`, unconditionally scan for living PIDs matching `--user-data-dir` or `instance.pid`.
   - Terminate all detected PIDs using `close_instance_verified`.
   - If PIDs remain alive, execute direct Win32 `TerminateProcess` ladder until `is_pid_alive_targeted(pid)` returns false.
   - Clean stale Chromium lockfiles (`lockfile`, `code.lock`, `DevToolsActivePort`).
   - If termination fails after hard deadline, abort cleanly *before* touching credentials.
2. In `system_impl.rs`:
   - Replace `record_ignored` on `close_instance_verified`. Ensure the process is truly confirmed dead before writing credentials.
