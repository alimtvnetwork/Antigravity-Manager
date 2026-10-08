# Subtask 01: Backend Smart Instance Process Cache & PID Liveness

## Objective
Implement smart instance process caching and PID verification in `src-tauri/src/modules/instance.rs` and `src-tauri/src/modules/process.rs` to prevent unwanted instance restarts and track running processes accurately.

## Actions
1. Define `InstanceProcessCacheEntry` and `GlobalInstanceProcessTracker` in `src-tauri/src/modules/instance.rs`.
2. Implement `get_instance_running_process_count() -> usize`: counts how many distinct Antigravity IDE instances are currently active on the host machine.
3. Implement `is_instance_process_running_cached(instance_id: &str) -> bool`:
   - Checks if cached PID is alive in OS process table.
   - If cached PID died, invalidates cache and rescans processes for `instance_id`.
4. Refactor `focus_or_launch_instance_with_workspace`:
   - If `!pids.is_empty()` (instance already running): focus window, and IF focus fails, DO NOT launch a new instance! Return `Ok(true)` because the instance is already running.
   - Only call `launch_instance_with_workspaces` if `pids.is_empty()`.
5. Implement `ensure_instance_running_smart(instance_id: &str, workspace_path: Option<&str>)`:
   - Verifies if running via cache.
   - If running, reuses it without re-launching.
   - If not running, launches and caches the newly created PID.
