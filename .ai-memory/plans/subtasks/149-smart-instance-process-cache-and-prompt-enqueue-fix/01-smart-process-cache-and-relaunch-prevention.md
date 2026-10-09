# Subtask Plan: Smart Process Cache Unification & Zero-Relaunch Prevention

> **Subtask ID:** `01-smart-process-cache-and-relaunch-prevention`  
> **Parent Slug:** `149-smart-instance-process-cache-and-prompt-enqueue-fix`  
> **Target Files:** `src-tauri/src/modules/instance.rs`, `src-tauri/src/modules/process.rs`, `src-tauri/src/commands/instance.rs`  
> **Status:** `[READY]`  
> **Author:** @aukgit  
> **Protocol:** `execute-parent-task-with-n-steps-v6`  

---

## 1. Context & Objectives

Users report that clicking "Send Now" or "Enqueue" for an instance profile frequently causes Antigravity Manager (AGM) to terminate the already running Antigravity IDE and launch a duplicate or blank window.

This subtask addresses the process-management root causes:
1. **Unify Dual Caches**: Merge `SMART_PROCESS_CACHE` and `INSTANCE_PROCESS_CACHE` into a single authoritative, thread-safe cache (`INSTANCE_PROCESS_CACHE`).
2. **Canonical Instance ID Resolution**: Ensure `is_instance_process_running_smart` calls `resolve_instance_id` to reliably resolve sequence numbers (`"1"`, `"#1"`), aliases (`"default"`, `"active"`), and case variations before cache lookups or registry checks.
3. **Closed-PID Double-Check & Re-scan**: When a cached PID exits, do not prematurely declare the instance dead; perform an authoritative OS re-scan for surviving child/main Antigravity processes matching the instance's data directory.
4. **Decouple Window Focus from Liveness**: Never conclude a process is dead if `focus_instance_workspace_window` fails. If the process is alive, return success immediately and NEVER invoke `launch_instance` or `close_instance`.

---

## 2. Step-by-Step Implementation Plan

### Step 1: Unify Process Caches in `src-tauri/src/modules/instance.rs`

#### Current Problem
- `SMART_PROCESS_CACHE` (line 95) uses `InstanceProcessRecord`.
- `INSTANCE_PROCESS_CACHE` (line 3240) uses `InstanceProcessCacheItem`.
- Subsystems update one while reading from the other, causing stale cache divergence.

#### Action
1. Merge the data structures into a unified `InstanceProcessRecord`:
   ```rust
   #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
   pub struct InstanceProcessRecord {
       pub instance_id: String,
       pub pids: Vec<u32>,
       pub primary_pid: Option<u32>,
       pub data_dir: String,
       pub launched_at: i64,
       pub last_verified_at: i64,
       pub is_alive: bool,
       pub command_line: Option<String>,
   }
   ```
2. Retain a single global cache:
   ```rust
   pub static INSTANCE_PROCESS_CACHE: std::sync::LazyLock<
       std::sync::Arc<std::sync::RwLock<std::collections::HashMap<String, InstanceProcessRecord>>>,
   > = std::sync::LazyLock::new(|| {
       std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()))
   });
   ```
3. Update `record_instance_pid`, `mark_instance_stopped`, and `invalidate_instance_process_cache` to write to this unified cache.
4. Provide helper functions:
   - `get_cached_instance_process(instance_id: &str) -> Option<InstanceProcessRecord>`
   - `invalidate_instance_process_cache(instance_id: &str)`
   - `update_cached_instance_process(record: InstanceProcessRecord)`

---

### Step 2: Integrate `resolve_instance_id` into `is_instance_process_running_smart`

#### Current Problem
- `is_instance_process_running_smart(instance_id: &str)` performs direct string equality against `registry.instances` without calling `resolve_instance_id`.
- Queries passing sequence numbers like `"1"`, prefixed IDs like `"ins-1"`, or alias `"default"` fail to match and falsely report the instance as offline.

#### Action
1. At the very top of `is_instance_process_running_smart`:
   ```rust
   let canonical_id = resolve_instance_id(instance_id).unwrap_or_else(|_| instance_id.to_string());
   ```
2. Check the unified cache using `canonical_id`:
   - If cached entry exists and primary PID is alive (`is_pid_alive_os(pid)`), return `(true, Some(pid), entry.pids)`.
3. When searching `registry.instances`:
   - Search by `i.id == canonical_id || (canonical_id == "default" && i.is_default)`.
4. Index both `canonical_id` and the input `instance_id` into the cache for fast future lookups.

---

### Step 3: Implement Closed-PID Double-Check & Re-Scan

#### Current Problem
- When Electron launches, the initial bootstrap PID may exit while child or secondary windows continue running.
- In `is_instance_process_running_smart`, if the primary PID dies, it checks `find_pids_for_data_dir`, which may use a stale 10-second cache (`PROCESS_SCAN_CACHE`) and fail to detect the live process.

#### Action
1. In `is_instance_process_running_smart`, if the cached PID is not alive:
   - Invalidate the stale cached entry.
   - Force a fresh process scan by bypassing or refreshing `PROCESS_SCAN_CACHE`:
     ```rust
     force_refresh_process_cache();
     ```
2. Scan running OS processes:
   - Match by `data_dir` path (with canonicalized/normalized slash separators).
   - Check if any Antigravity process matches the custom cloned executable (e.g. `antigravity-{canonical_id}`).
   - Check SQLite `instance_processes` table for recently recorded PIDs.
3. If any surviving PID is found alive:
   - Re-populate `INSTANCE_PROCESS_CACHE` with the surviving PIDs.
   - Update SQLite `instance_processes` with the new active PID.
   - Return `(true, Some(surviving_pid), pids)`.
4. Only return `(false, None, Vec::new())` if zero processes match across the entire system.

---

### Step 4: Zero-Relaunch Enforcement in `ensure_instance_running_smart` & `focus_or_launch`

#### Current Problem
- `focus_or_launch_instance_with_workspace` and `focus_or_launch_workspace` call `ensure_instance_running_smart`.
- If window focus fails, or if the process check failed, it executed `launch_instance_with_workspaces`, which called `close_instance(instance_id)`.
- This forcibly terminated the user's running IDE.

#### Action
1. Rewrite `ensure_instance_running_smart`:
   ```rust
   pub fn ensure_instance_running_smart(
       instance_id: &str,
       workspace_path: Option<&str>,
   ) -> Result<(bool, Option<u32>), crate::error::AppError> {
       let canonical_id = resolve_instance_id(instance_id).unwrap_or_else(|_| instance_id.to_string());
       let (is_running, primary_pid, pids) = is_instance_process_running_smart(&canonical_id);

       if is_running {
           crate::modules::logger::log_info(&format!(
               "[SmartProcessCache] Instance '{}' ({}) is verified running (PID: {:?}). Zero relaunch guarantee active.",
               instance_id, canonical_id, primary_pid
           ));

           // Attempt non-invasive window focus
           if let Some(ws) = workspace_path {
               let clean_ws = ws.trim_end_matches(['/', '\\']);
               let repo_name = std::path::Path::new(clean_ws)
                   .file_name()
                   .and_then(|n| n.to_str())
                   .unwrap_or(clean_ws);
               let _ = crate::modules::process::focus_instance_workspace_window(&pids, repo_name);
           } else {
               let _ = crate::modules::process::focus_instance_pids(&pids);
           }

           // INVARIANT: Process is alive! Return immediately. NEVER terminate or cold-launch.
           return Ok((true, primary_pid));
       }

       // Only if confirmed dead across entire OS:
       crate::modules::logger::log_info(&format!(
           "[SmartProcessCache] Instance '{}' ({}) confirmed offline across OS. Initiating cold launch...",
           instance_id, canonical_id
       ));

       if let Some(ws) = workspace_path {
           let ws_vec = vec![ws.to_string()];
           launch_instance_with_workspaces(&canonical_id, Some(&ws_vec), true)?;
       } else {
           launch_instance(&canonical_id)?;
       }

       std::thread::sleep(std::time::Duration::from_millis(800));
       invalidate_instance_process_cache(&canonical_id);
       force_refresh_process_cache();
       let (is_now_running, new_pid, _) = is_instance_process_running_smart(&canonical_id);

       Ok((is_now_running, new_pid))
   }
   ```
2. Update `focus_or_launch_instance_with_workspace` and `focus_or_launch_workspace` to guarantee that they return `Ok(true)` as long as the process is alive, regardless of window focus return code.

---

### Step 5: Multi-Instance Process Count Synchronization

#### Current Problem
- `get_running_instances_process_count` and `scan_and_cache_all_running_instances` did not synchronize with the unified cache, leading to inaccurate badge counts on the UI.

#### Action
1. In `scan_and_cache_all_running_instances`:
   - Enumerate all instances from registry.
   - For each instance, resolve canonical ID and check live PIDs.
   - Insert records into `INSTANCE_PROCESS_CACHE`.
   - Return the count of instances that have $\ge 1$ verified live PID.
2. In `src-tauri/src/commands/instance.rs`:
   - Ensure `get_running_instances_process_count()` calls `scan_and_cache_all_running_instances()`.

---

## 3. Verification & Safety Checklist

- [ ] `resolve_instance_id` correctly resolves `"1"`, `"#1"`, `"ins-1"`, `"default"`, and full IDs.
- [ ] `SMART_PROCESS_CACHE` references removed; unified `INSTANCE_PROCESS_CACHE` in use across all functions.
- [ ] Process liveness check runs in $< 1\text{ms}$ on cache hit.
- [ ] If an instance IDE window is open, calling `focus_or_launch_instance` or `focus_or_launch_workspace` NEVER kills or restarts the window.
- [ ] Pre-flight checks pass:
  - `cd src-tauri && cargo fmt -- --check`
  - `cd src-tauri && cargo clippy --all-targets --all-features`
