# Subtask 01 Engineering Spec: Prompt Send & Enqueue RCA, Smart Process Cache, and Relaunch Elimination

**Subtask:** `01-prompt-send-enqueue-rca-and-process-cache.md`  
**Parent Task:** `147-smart-instance-process-cache-and-prompt-dispatch`  
**File Location:** `.ai-memory/plans/subtasks/147-smart-instance-process-cache-and-prompt-dispatch/01-prompt-send-enqueue-rca-and-process-cache.md`  
**Target Modules:**  
- `src-tauri/src/modules/instance.rs` (Smart Process Cache, PID liveness verification, decoupled focus/launch)  
- `src-tauri/src/commands/instance.rs` (Backend IPC `enqueue_prompt` command)  
- `src-tauri/src/lib.rs` (Tauri command registration)  
- `src-tauri/src/modules/repo_db.rs` (`enqueue_prompt_for_instance`)  
- `src/components/instances/PromptTreeViewModal.tsx` (`handleResendPrompt`, `handleEnqueuePrompt` error recovery and feedback)  
**Status:** READY FOR IMPLEMENTATION  

---

## 1. Problem Statement & Subtask Scope

When users dispatch prompts from the **Prompt Tree View Modal** in Antigravity-Manager:
1. **Unwanted IDE Relaunch & Process Destruction**: Calling `focus_or_launch_instance_with_workspace` erroneously treats a failed window focus (Win32 `SetForegroundWindow` failing due to OS foreground lock or minimized state) as proof of process termination. It then invokes `launch_instance_with_workspaces`, which proactively kills the running IDE process via `close_instance(instance_id)`.
2. **Missing `enqueue_prompt` IPC Command**: The frontend attempts to invoke `enqueue_prompt`, but the backend lacks this command in `commands/instance.rs` and `lib.rs`. The request fails silently and falls back to saving a disconnected `.antigravity_resume_task.json` file.
3. **Missing Process Cache**: Process discovery scans repeatedly query the entire operating system process table (`ProcessesToUpdate::All`), creating latency spikes and race conditions during prompt dispatch.

This subtask delivers:
- **`SMART_PROCESS_CACHE`** in `src-tauri/src/modules/instance.rs` with targeted <1ms liveness verification.
- **Decoupled Focus and Launch** in `focus_or_launch_instance_with_workspace` and `focus_or_launch_workspace`.
- **Backend IPC Command `enqueue_prompt`** in `commands/instance.rs` and `lib.rs`.
- **Frontend Action Hardening** in `PromptTreeViewModal.tsx`.

---

## 2. Technical Design & Architecture

### 2.1 Smart Process Cache (`SMART_PROCESS_CACHE`)

We introduce an in-memory, thread-safe cache in `src-tauri/src/modules/instance.rs`:

```rust
use once_cell::sync::Lazy;
use std::sync::RwLock;
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InstanceProcessRecord {
    pub instance_id: String,
    pub pid: u32,
    pub data_dir: String,
    pub launched_at: i64,
    pub last_verified_at: i64,
    pub is_alive: bool,
    pub command_line: Option<String>,
}

pub static SMART_PROCESS_CACHE: Lazy<std::sync::Arc<RwLock<HashMap<String, InstanceProcessRecord>>>> =
    Lazy::new(|| std::sync::Arc::new(RwLock::new(HashMap::new())));
```

### 2.2 Sub-Millisecond PID Liveness Check (`is_pid_alive_targeted`)

Instead of rebuilding the entire OS process table, we execute a targeted PID check:

```rust
/// Check if a specific PID is alive in the operating system in sub-millisecond time.
pub fn is_pid_alive_targeted(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }

    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::Foundation::CloseHandle;
        use windows_sys::Win32::System::Threading::{
            GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, STILL_ACTIVE,
        };
        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if !handle.is_null() {
                let mut exit_code: u32 = 0;
                let success = GetExitCodeProcess(handle, &mut exit_code);
                CloseHandle(handle);
                return success != 0 && exit_code == STILL_ACTIVE as u32;
            }
        }
    }

    // Cross-platform fallback via sysinfo targeted update
    let mut sys = sysinfo::System::new();
    let target = sysinfo::Pid::from_u32(pid);
    sys.refresh_processes_specifics(
        sysinfo::ProcessesToUpdate::Some(&[target]),
        sysinfo::ProcessRefreshKind::new(),
    );
    sys.process(target).is_some()
}
```

### 2.3 Three-Tier Verification Engine: `ensure_instance_running_for_dispatch`

```rust
/// Guarantee that the instance is running prior to prompt dispatch.
/// Tier 1: Check SMART_PROCESS_CACHE. If PID alive -> returns PID instantly (<1ms).
/// Tier 2: Check OS process table for data_dir match (find_pids_for_data_dir). If found -> update cache and return PID.
/// Tier 3: ONLY if confirmed completely dead across the OS -> cold launch the instance.
pub fn ensure_instance_running_for_dispatch(
    instance_id: &str,
    workspace_path: Option<&str>,
) -> Result<u32, crate::error::AppError> {
    let resolved_id = resolve_instance_id(instance_id).unwrap_or_else(|_| instance_id.to_string());
    let now = chrono::Utc::now().timestamp();

    // -------------------------------------------------------------
    // Tier 1: Fast Cached PID Verification (<1ms)
    // -------------------------------------------------------------
    {
        let cache = SMART_PROCESS_CACHE.read().unwrap();
        if let Some(record) = cache.get(&resolved_id) {
            if record.pid > 0 && is_pid_alive_targeted(record.pid) {
                // Verified alive! Record is fresh.
                crate::modules::logger::log_info(&format!(
                    "[SmartProcessCache] Tier 1 HIT: instance '{}' PID {} verified alive in <1ms",
                    resolved_id, record.pid
                ));
                return Ok(record.pid);
            }
        }
    }

    // -------------------------------------------------------------
    // Tier 2: Targeted System Re-Scan
    // -------------------------------------------------------------
    let registry = load_registry().map_err(crate::error::AppError::Config)?;
    let inst = registry
        .instances
        .iter()
        .find(|i| i.id == resolved_id || (resolved_id == "default" && i.is_default))
        .ok_or_else(|| {
            crate::error::AppError::Config(format!("Instance {} not found", resolved_id))
        })?;

    let is_default = inst.is_default || inst.id == "default";
    let pids = find_pids_for_data_dir(&inst.data_dir, is_default);

    if let Some(&primary_pid) = pids.first() {
        if primary_pid > 0 && is_pid_alive_targeted(primary_pid) {
            crate::modules::logger::log_info(&format!(
                "[SmartProcessCache] Tier 2 HIT: instance '{}' discovered living PID {} in OS process table; updating cache",
                resolved_id, primary_pid
            ));

            // Update SMART_PROCESS_CACHE
            {
                let mut cache = SMART_PROCESS_CACHE.write().unwrap();
                cache.insert(
                    resolved_id.clone(),
                    InstanceProcessRecord {
                        instance_id: resolved_id.clone(),
                        pid: primary_pid,
                        data_dir: inst.data_dir.clone(),
                        launched_at: now,
                        last_verified_at: now,
                        is_alive: true,
                        command_line: None,
                    },
                );
            }
            let _ = record_instance_pid(&resolved_id, primary_pid, &inst.data_dir);
            return Ok(primary_pid);
        }
    }

    // Also check saved PID from SQLite before concluding instance is dead
    if let Some(saved_pid) = get_instance_saved_pid(&resolved_id) {
        if saved_pid > 0 && is_pid_alive_targeted(saved_pid) {
            crate::modules::logger::log_info(&format!(
                "[SmartProcessCache] Tier 2B HIT: instance '{}' saved PID {} verified alive; updating cache",
                resolved_id, saved_pid
            ));
            let mut cache = SMART_PROCESS_CACHE.write().unwrap();
            cache.insert(
                resolved_id.clone(),
                InstanceProcessRecord {
                    instance_id: resolved_id.clone(),
                    pid: saved_pid,
                    data_dir: inst.data_dir.clone(),
                    launched_at: now,
                    last_verified_at: now,
                    is_alive: true,
                    command_line: None,
                },
            );
            return Ok(saved_pid);
        }
    }

    // -------------------------------------------------------------
    // Tier 3: Cold Launch Fallback (ONLY if confirmed dead across OS)
    // -------------------------------------------------------------
    crate::modules::logger::log_info(&format!(
        "[SmartProcessCache] Tier 3: instance '{}' confirmed completely dead across OS; cold launching targeted at workspace: {:?}",
        resolved_id, workspace_path
    ));

    if let Some(ws) = workspace_path {
        let ws_vec = vec![ws.to_string()];
        launch_instance_with_workspaces(&resolved_id, Some(&ws_vec), true)?;
    } else {
        launch_instance(&resolved_id)?;
    }

    // Retrieve freshly launched PID
    std::thread::sleep(std::time::Duration::from_millis(600));
    let fresh_pids = find_pids_for_data_dir(&inst.data_dir, is_default);
    let fresh_pid = fresh_pids.first().copied().unwrap_or(0);

    if fresh_pid > 0 {
        let mut cache = SMART_PROCESS_CACHE.write().unwrap();
        cache.insert(
            resolved_id.clone(),
            InstanceProcessRecord {
                instance_id: resolved_id.clone(),
                pid: fresh_pid,
                data_dir: inst.data_dir.clone(),
                launched_at: now,
                last_verified_at: now,
                is_alive: true,
                command_line: None,
            },
        );
    }

    Ok(fresh_pid)
}
```

### 2.4 Decoupling Window Focus in `focus_or_launch_instance_with_workspace`

Modify `src-tauri/src/modules/instance.rs:3060`:

```rust
pub fn focus_or_launch_instance_with_workspace(
    instance_id: &str,
    workspace_path: Option<&str>,
) -> Result<bool, crate::error::AppError> {
    // 1. Ensure instance is running using 3-Tier Verification
    let pid = ensure_instance_running_for_dispatch(instance_id, workspace_path)?;

    // 2. Attempt window focus safely
    let pids = vec![pid];
    let focused = if let Some(ws) = workspace_path {
        let clean_ws = ws.trim_end_matches(['/', '\\']);
        let repo_name = std::path::Path::new(clean_ws)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(clean_ws);
        crate::modules::process::focus_instance_workspace_window(&pids, repo_name)
    } else {
        false
    };

    let final_focused = if focused {
        true
    } else {
        crate::modules::process::focus_instance_pids(&pids)
    };

    // 3. SAFE DEGRADATION: If window focus could not be established by OS,
    // NEVER relaunch! NEVER call close_instance!
    if !final_focused {
        crate::modules::logger::log_info(&format!(
            "[Instance] Instance '{}' (PID {}) is running, but OS focus window could not be raised; keeping living process intact",
            instance_id, pid
        ));
    }

    Ok(final_focused)
}
```

Apply the identical decoupling to `focus_or_launch_workspace` (`src-tauri/src/modules/instance.rs:3125`).

---

## 3. Implementation Steps: Subtask 01

### Step 1: Implement `SMART_PROCESS_CACHE` and Liveness Helpers
- **File:** `src-tauri/src/modules/instance.rs`
- **Actions:**
  1. Define `InstanceProcessRecord` and `SMART_PROCESS_CACHE`.
  2. Implement `is_pid_alive_targeted(pid: u32) -> bool`.
  3. Implement `ensure_instance_running_for_dispatch(instance_id: &str, workspace_path: Option<&str>) -> Result<u32, crate::error::AppError>`.
  4. In `close_instance(instance_id: &str)`, remove the target instance from `SMART_PROCESS_CACHE`.
  5. In `record_instance_pid(instance_id: &str, pid: u32, data_dir: &str)`, update `SMART_PROCESS_CACHE`.

### Step 2: Refactor `focus_or_launch_instance_with_workspace` and `focus_or_launch_workspace`
- **File:** `src-tauri/src/modules/instance.rs`
- **Actions:**
  1. Replace lines 3060-3122 with the decoupled focus implementation.
  2. Replace lines 3125-3179 with the decoupled workspace focus implementation.
  3. Ensure that if `final_focused` is false, it returns `Ok(false)` without triggering `launch_instance_with_workspaces`.

### Step 3: Implement `enqueue_prompt_for_instance` in `repo_db.rs`
- **File:** `src-tauri/src/modules/repo_db.rs`
- **Actions:**
  1. Add public function:
     ```rust
     pub fn enqueue_prompt_for_instance(
         instance_id: &str,
         repo_path: &str,
         prompt_content: &str,
         conversation_id: Option<&str>,
         project_id: Option<&str>,
     ) -> Result<String, String> {
         let conn = connect_db()?;
         let now = Utc::now().timestamp();
         let norm_inst = crate::modules::instance::resolve_instance_id(instance_id)
             .unwrap_or_else(|_| instance_id.to_string());
         let clean_repo = repo_path.trim();
         let proj_id = project_id.unwrap_or(clean_repo);
         let prompt_id = format!("queued-{}-{}", norm_inst, uuid::Uuid::new_v4());

         conn.execute(
             "INSERT INTO active_prompts (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at)
              VALUES (?1, ?2, ?3, ?4, ?5, 'gemini-2.5-pro', ?6, 'queued', ?7, ?8)",
             rusqlite::params![
                 &prompt_id,
                 proj_id,
                 &norm_inst,
                 clean_repo,
                 prompt_content,
                 conversation_id,
                 now,
                 now
             ],
         ).map_err(|e| format!("Failed to enqueue prompt: {}", e))?;

         invalidate_prompt_tree_cache(Some(&norm_inst));
         Ok(prompt_id)
     }
     ```

### Step 4: Add and Register IPC Command `enqueue_prompt`
- **File:** `src-tauri/src/commands/instance.rs`
- **Actions:**
  1. Add `enqueue_prompt`:
     ```rust
     #[tauri::command]
     pub fn enqueue_prompt(
         instance_id: String,
         repo_path: String,
         prompt_content: String,
         conversation_id: Option<String>,
         project_id: Option<String>,
     ) -> Result<String, String> {
         crate::modules::repo_db::enqueue_prompt_for_instance(
             &instance_id,
             &repo_path,
             &prompt_content,
             conversation_id.as_deref(),
             project_id.as_deref(),
         )
     }
     ```
- **File:** `src-tauri/src/lib.rs`
- **Actions:**
  1. Register `commands::instance::enqueue_prompt` in `generate_handler!`.

### Step 5: Update `PromptTreeViewModal.tsx` Handlers
- **File:** `src/components/instances/PromptTreeViewModal.tsx`
- **Actions:**
  1. Update `handleResendPrompt`: Keep the IPC call clean, write `.antigravity_resume_task.json`, copy to clipboard, and call `focusOrLaunchInstance(targetInstId, repoPath)`.
  2. Update `handleEnqueuePrompt`: Now that `enqueue_prompt` is a native registered Tauri command, remove the noisy warning fallback, display positive feedback on success (`Prompt enqueued into FIFO scheduler queue!`), and trigger `refreshPromptTree()`.

---

## 4. Verification & Testing Protocol

1. **Focus Failure Test**: With Antigravity IDE running in the background, invoke `focus_or_launch_instance`. Verify in `agm.log` that:
   - Tier 1 HIT or Tier 2 HIT is recorded.
   - Zero calls to `close_instance()` occur.
   - PID remains identical before and after the call (`Get-Process Antigravity`).
2. **IPC Enqueue Test**: From the frontend UI or via devtools `invoke('enqueue_prompt', ...)`, enqueue a prompt. Verify in `repo_prompts.db`:
   ```sql
   SELECT id, status, instance_id, created_at FROM active_prompts WHERE status = 'queued';
   ```
   Confirm row is present with `status = 'queued'`.
3. **Queue Dispatch Verification**: Run `agm prompt queue` or wait for `check_and_dispatch_enqueued_prompts`. Verify the prompt is dispatched in strict FIFO sequence.
