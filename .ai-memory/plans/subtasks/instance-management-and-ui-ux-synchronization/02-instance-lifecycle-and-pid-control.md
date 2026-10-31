---
id: task-02-instance-lifecycle-and-pid-control
title: Backend Instance Lifecycle and PID Control Hardening
status: QUEUED
dependencies: []
target_components:
  - src-tauri/src/modules/instance/close.rs
  - src-tauri/src/modules/instance/create.rs
  - src-tauri/src/modules/instance/switch.rs
  - src-tauri/src/commands/instance.rs
---

# Subtask Plan: Task-02 — Backend Instance Lifecycle and PID Control Hardening

Spec Reference: [02-spec/21-app/instance-management-and-ui-ux-synchronization/02-component-spec.md](../../../../02-spec/21-app/instance-management-and-ui-ux-synchronization/02-component-spec.md)

## 1. Context & Objectives

1. **Eliminate the 3-Second Hard Abort Bug**: In `src-tauri/src/modules/instance/close.rs`, prevent premature `return Err(...)` when Electron/Chromium helper subprocesses exceed 3,000ms during `close_instance`.
2. **Implement Windows Process Tree Kill Fallback Ladder**: Provide multi-stage progressive termination (`taskkill /F /T` -> PowerShell `Stop-Process -Force` / `sysinfo` kill -> verification loop) with bounded timeout and diagnostic journal logging.
3. **Guarantee Post-Kill Cleanup Invariants**: Ensure stale lockfiles (`lockfile`, `code.lock`, `Singleton*`, `DevToolsActivePort`) are purged, SQLite `is_active = 0` is set, `inst.pid = None` is updated, and caches are invalidated unconditionally.
4. **Enforce 6-Stage Deterministic Sequencing in `switch_account_to_instance`**:
   - Stage 1: Snapshot active arguments, workspaces, running prompts, and account status.
   - Stage 2: Terminate target instance PIDs with verified dead confirmation.
   - Stage 3: Purge stale lockfiles and enforce mandatory 200ms OS settle delay.
   - Stage 4: Inject credentials into `state.vscdb`, `storage.json`, and set keyring bypass markers.
   - Stage 5: Spawn IDE process and background CDP dialog dismisser.
   - Stage 6: Poll & sync live PID into DB (`is_active = 1`), update registry, and emit `instances://refreshed` & `accounts://refreshed` to Tauri UI.
5. **Instant Frontend Event Synchronization**: Update `stop_instance` in `src-tauri/src/commands/instance.rs` to broadcast `instances://refreshed` and `accounts://refreshed`.

---

## 2. Target Files & Exact Modification Steps

### Step 1: Refactor `src-tauri/src/modules/instance/close.rs`

#### A. Refactor `close_instance` (around lines 500–566)
- **Problem**: When `start_wait.elapsed() > max_graceful (3000ms)`, it returns `Err(format!("Instance '{instance_id}' PID(s) {still:?} stayed alive after taskkill"))`, prematurely aborting before cleaning lockfiles, updating `is_active = 0`, or clearing caches.
- **Modification**:
  1. Split the wait loop into two phases:
     - Phase 1 (Graceful/Initial Kill verification, 1,500ms): Poll every 50ms. If all processes exit, break immediately.
     - Phase 2 (Force Kill Fallback, up to 2,500ms total): For any remaining alive PIDs:
       - Windows: Trigger PowerShell `Stop-Process -Id <pid> -Force` or sysinfo `proc.kill()`.
       - Unix: Trigger `kill -9 <pid>`.
  2. If PIDs survive after the full timeout:
     - Check if primary executable is dead.
     - Log surviving helper PIDs as warnings to `[CLOSE_JOURNAL]` instead of returning `Err`.
  3. **Unconditional Execution of Cleanup**:
     ```rust
     let target_data_path = PathBuf::from(&config.data_dir);
     let dt_port = target_data_path.join("DevToolsActivePort");
     if dt_port.exists() {
         crate::error::record_ignored(fs::remove_file(&dt_port), "remove file");
     }
     clear_stale_instance_lockfiles(&target_data_path);
     crate::error::record_ignored(mark_instance_stopped(instance_id), "mark instance stopped");
     invalidate_instance_process_cache(instance_id);
     crate::modules::repo_db::invalidate_prompt_tree_cache(Some(instance_id));
     std::thread::sleep(std::time::Duration::from_millis(150));
     Ok(())
     ```

#### B. Refactor `close_instance_verified` (around lines 201–263)
- Ensure errors from individual `close_instance` passes do not terminate the retry sequence prematurely.
- Log pass attempts and remaining PIDs cleanly in `[CLOSE_JOURNAL]`.
- Return `Ok(true)` if confirmed dead, `Ok(false)` if stragglers remain after 2 passes, invalidating caches in both outcomes.

---

### Step 2: Refactor `src-tauri/src/modules/instance/create.rs`

#### Refactor `stop_instance` (lines 325–334)
- **Problem**: Calls `close_instance_verified` and returns `Err` if not all processes exit, without ensuring lockfiles are cleared and `mark_instance_stopped` is applied.
- **Modification**:
  ```rust
  pub fn stop_instance(instance_id: &str) -> Result<(), String> {
      let resolved = resolve_instance_id(instance_id).unwrap_or_else(|_| instance_id.to_string());
      let closed = close_instance_verified(&resolved, std::time::Duration::from_millis(5000))
          .map_err(|e| format!("Error closing instance '{resolved}': {e}"))?;

      // Unconditional post-stop cleanup
      if let Ok(reg) = load_registry() {
          if let Some(inst) = reg.instances.iter().find(|i| i.id == resolved) {
              let data_path = std::path::PathBuf::from(&inst.data_dir);
              clear_stale_instance_lockfiles(&data_path);
          }
      }
      crate::error::record_ignored(mark_instance_stopped(&resolved), "mark instance stopped");
      invalidate_instance_process_cache(&resolved);
      crate::modules::repo_db::invalidate_prompt_tree_cache(Some(&resolved));

      if !closed {
          let remaining = get_instance_saved_pid(&resolved);
          if let Some(pid) = remaining {
              if is_pid_alive_targeted(pid) {
                  return Err(format!("Instance '{resolved}' primary PID {pid} stayed alive after stop"));
              }
          }
          crate::modules::logger::log_warn(&format!(
              "[Instance] Stop completed for '{}' with helper stragglers; locks and DB cleaned",
              resolved
          ));
      }

      Ok(())
  }
  ```

---

### Step 3: Refactor `src-tauri/src/modules/instance/switch.rs`

#### Refactor `switch_account_to_instance` (lines 224–435 & lines 570–605)
- **Implement 6-Stage Deterministic Sequence**:
  - **Stage 1**: Snapshot running exe path, CLI args, workspace folders, and re-enqueue active prompts.
  - **Stage 2**: Terminate target instance PIDs via `close_instance_verified(&instance.id, Duration::from_millis(5000))`. Verify primary PID is dead.
  - **Stage 3**: Explicitly invoke `clear_stale_instance_lockfiles(&PathBuf::from(&instance.data_dir))` and sleep for 200ms (`std::thread::sleep(Duration::from_millis(200))`).
  - **Stage 4**: Inject credentials via `inject_account_credentials`, bind account in registry, and asynchronously trigger Supabase lease acquisition.
  - **Stage 5**: Launch IDE with original arguments and workspace folders, and spawn async CDP security notice dismisser.
  - **Stage 6**: Sync Live PID & Broadcast Events:
    ```rust
    // Poll process table for newly spawned PID (up to 3 seconds)
    let fresh_pid = {
        let mut detected = None;
        let start = std::time::Instant::now();
        while start.elapsed() < std::time::Duration::from_millis(3000) {
            let pids = find_pids_for_data_dir(&instance.data_dir, is_default_inst);
            if let Some(&first) = pids.first() {
                if is_pid_alive_targeted(first) {
                    detected = Some(first);
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        detected
    };

    if let Some(pid) = fresh_pid {
        crate::error::record_ignored(
            record_instance_pid(&instance.id, pid, &instance.data_dir),
            "record live pid after switch",
        );
    }

    if let Some(handle) = crate::modules::log_bridge::get_app_handle() {
        use tauri::Emitter;
        crate::error::record_ignored(handle.emit("instances://refreshed", ()), "emit instances://refreshed");
        crate::error::record_ignored(handle.emit("accounts://refreshed", ()), "emit accounts://refreshed");
    }
    ```

---

### Step 4: Refactor `src-tauri/src/commands/instance.rs`

#### Refactor `stop_instance` (lines 53–56)
- Update command to broadcast `instances://refreshed` and `accounts://refreshed` via `crate::modules::log_bridge::get_app_handle()` upon successful shutdown.
- Repeat event emission in `close_instance` command (lines 207–210).

---

## 3. Verification Commands & Testing Strategy

```powershell
# 1. Format check
cd src-tauri; cargo fmt -- --check

# 2. Clippy gate
cd src-tauri; cargo clippy --all-targets --all-features

# 3. Targeted test execution
cd src-tauri; cargo test --lib modules::instance::close
cd src-tauri; cargo test --lib modules::instance::create
cd src-tauri; cargo test --lib modules::instance::switch
```

---

## 4. Acceptance Criteria & Quality Gates

- [ ] **No Premature Abort**: `close_instance` handles stubborn processes without bailing out after 3,000ms.
- [ ] **Windows Fallback Working**: When `taskkill` is slow or fails, PowerShell `Stop-Process -Force` / `sysinfo` kill is attempted.
- [ ] **Lockfile Hygiene**: `lockfile`, `code.lock`, `DevToolsActivePort`, and `Singleton*` are deleted from `data_dir` upon stop.
- [ ] **Database Synced**: `is_active` is updated to `0` in SQLite `instance_processes` table and registry `pid` is `null` when an instance is stopped.
- [ ] **6-Stage Sequencing**: `switch_account_to_instance` runs stages 1 through 6 deterministically, settling file locks before injecting auth tokens and syncing live PID upon launch.
- [ ] **UI Event Emission**: `instances://refreshed` and `accounts://refreshed` are emitted on instance stop and account switch.
