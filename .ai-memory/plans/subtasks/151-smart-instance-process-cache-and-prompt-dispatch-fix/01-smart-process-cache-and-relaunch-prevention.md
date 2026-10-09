# Subtask Implementation Plan: Smart Process Cache & Relaunch Prevention

> **Subtask ID:** `01-smart-process-cache-and-relaunch-prevention`  
> **Parent Slug:** `151-smart-instance-process-cache-and-prompt-dispatch-fix`  
> **Target Files:** `src-tauri/src/modules/instance.rs`, `src-tauri/src/commands/instance.rs`, `src/components/instances/PromptTreeViewModal.tsx`  
> **Status:** `[READY]`  
> **Author:** @aukgit  
> **Protocol:** `execute-parent-task-with-n-steps-v6`  

---

## 1. Context & Objectives

Users report that clicking "Send Now", pressing hotkey 'N', or clicking "Enqueue" frequently causes Antigravity Manager (AGM) to terminate their already running Antigravity IDE and launch a blank, duplicate window. This destroys in-flight terminal work, unsaved buffers, and causes prompt delivery failure.

### Core Objectives:
1. **Multi-PID Vitality Check & Promotion**: Refactor `check_cached_pid_alive` so that when the primary launcher PID terminates, surviving child PIDs in `entry.pids` are scanned, and the first living survivor is promoted to `primary_pid`. Only invalidate the cache if all PIDs in `entry.pids` are confirmed dead.
2. **Windows 8.3 & Path Normalization**: Enhance `find_pids_for_data_dir` to expand 8.3 short paths (e.g. `C:\Users\ADMINI~1\...` to `C:\Users\Administrator\...`), perform case-insensitive slash-agnostic comparisons, and trace process lineage for Electron child workers that omit `--user-data-dir`.
3. **Closed-PID OS Re-Scan**: In `is_instance_process_running_smart`, when the cache indicates dead PIDs, re-scan the OS process table immediately. If any surviving IDE process is detected, update the cache and return running status without triggering a cold launch.
4. **Zero-Relaunch Guarantee**: Completely decouple window focusing from process launching in `ensure_instance_running_smart`. If an instance is running, invoke `focus_running_instance` and return immediately. Under no condition shall `close_instance` (`taskkill`) be called during prompt dispatch.
5. **Frontend Dispatch De-Duplication**: In `PromptTreeViewModal.tsx`, remove redundant `focusInstanceWorkspace` invocations following `sendPromptNow`.

---

## 2. Step-by-Step Implementation Details

### Step 1: Multi-PID Vitality Check & Promotion in `check_cached_pid_alive`

**File:** `src-tauri/src/modules/instance.rs` (lines 3280–3294)

#### Current Fragility:
```rust
fn check_cached_pid_alive(
    instance_id: &str,
    canonical_id: &str,
) -> Option<(bool, Option<u32>, Vec<u32>)> {
    let entry = get_cached_instance_process(canonical_id)
        .or_else(|| get_cached_instance_process(instance_id))?;
    if let Some(pid) = entry.primary_pid {
        let is_alive = is_pid_alive_targeted(pid) && saved_pid_matches(pid);
        if is_alive {
            return Some((true, Some(pid), entry.pids));
        }
    }
    invalidate_instance_process_cache(instance_id);
    None
}
```

#### Proposed Implementation:
```rust
fn check_cached_pid_alive(
    instance_id: &str,
    canonical_id: &str,
) -> Option<(bool, Option<u32>, Vec<u32>)> {
    let mut entry = get_cached_instance_process(canonical_id)
        .or_else(|| get_cached_instance_process(instance_id))?;

    // 1. Verify primary PID
    if let Some(primary) = entry.primary_pid {
        if is_pid_alive_targeted(primary) && saved_pid_matches(primary) {
            return Some((true, Some(primary), entry.pids));
        }
    }

    // 2. Multi-PID Vitality Fallback: Check all surviving PIDs
    let surviving_pids: Vec<u32> = entry
        .pids
        .iter()
        .copied()
        .filter(|&pid| pid > 0 && is_pid_alive_targeted(pid) && saved_pid_matches(pid))
        .collect();

    if !surviving_pids.is_empty() {
        let promoted_pid = surviving_pids[0];
        crate::modules::logger::log_info(&format!(
            "[SmartProcessCache] Primary PID {:?} exited for instance '{}'. Promoted surviving child PID {} from {} candidates.",
            entry.primary_pid, canonical_id, promoted_pid, surviving_pids.len()
        ));

        // Promote new primary PID and update cache
        entry.primary_pid = Some(promoted_pid);
        entry.pid = promoted_pid;
        entry.pids = surviving_pids.clone();
        entry.last_verified_at = chrono::Utc::now().timestamp();
        entry.is_alive = true;

        if let Ok(mut cache) = INSTANCE_PROCESS_CACHE.write() {
            cache.insert(canonical_id.to_string(), entry.clone());
            if instance_id != canonical_id {
                cache.insert(instance_id.to_string(), entry);
            }
        }
        let _ = record_instance_pid(canonical_id, promoted_pid, &entry.data_dir);
        return Some((true, Some(promoted_pid), surviving_pids));
    }

    // 3. Only invalidate cache when all candidate PIDs are dead
    invalidate_instance_process_cache(canonical_id);
    if instance_id != canonical_id {
        invalidate_instance_process_cache(instance_id);
    }
    None
}
```

---

### Step 2: Windows 8.3 & Path Normalization in `find_pids_for_data_dir`

**File:** `src-tauri/src/modules/instance.rs` (lines 771–850)

#### Key Improvements:
1. Resolve Windows short path representations (e.g. `C:\Users\ADMINI~1\AppData` vs `C:\Users\Administrator\AppData`) using `dunce::canonicalize` or case-insensitive string expansion.
2. Build dual candidate match sets:
   - Forward-slash normalized (`c:/users/administrator/appdata/local/...`)
   - Backslash normalized (`c:\users\administrator\appdata\local\...`)
   - 8.3 alias short form if available.
3. Track Electron child process hierarchy: when a root process matches the data directory, record its PID in an `instance_root_pids` set so that child helper/renderer processes descending from it are properly attributed.

```rust
pub fn find_pids_for_data_dir(data_dir: &str, is_default: bool) -> Vec<u32> {
    let processes = get_cached_antigravity_processes();

    let norm_slash = data_dir.to_lowercase().replace('\\', "/");
    let clean_slash = norm_slash.trim_end_matches('/').to_string();
    let norm_bslash = data_dir.to_lowercase().replace('/', "\\");
    let clean_bslash = norm_bslash.trim_end_matches('\\').to_string();

    let canonical_expanded = std::fs::canonicalize(data_dir).ok().map(|p| {
        let s = p.to_string_lossy().to_lowercase().replace('\\', "/");
        s.trim_start_matches("//?/")
            .trim_start_matches(r"\\?\")
            .trim_end_matches('/')
            .to_string()
    });

    let mut matched_pids = Vec::new();
    let mut parent_map: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
    let mut instance_root_pids: std::collections::HashSet<u32> = std::collections::HashSet::new();
    let mut default_candidate_pids: Vec<u32> = Vec::new();

    for proc in &processes {
        let pid_u32 = proc.pid;
        if let Some(parent) = proc.parent {
            parent_map.insert(pid_u32, parent);
        }

        let name = &proc.name;
        let exe = &proc.exe;
        let args_str = &proc.args_str;

        let has_target_match = (!clean_slash.is_empty() && args_str.contains(&clean_slash))
            || (!clean_bslash.is_empty() && args_str.contains(&clean_bslash))
            || canonical_expanded.as_ref().map_or(false, |c| !c.is_empty() && args_str.contains(c));

        // Cloned exe marker matching
        let matches_cloned_exe = if clean_slash.contains("/instances/") {
            let inst_id = clean_slash.split("/instances/").nth(1).and_then(|s| s.split('/').next());
            if let Some(id) = inst_id {
                let marker = format!("antigravity-{}", id.to_lowercase());
                exe.contains(&marker) || name.contains(&marker)
            } else {
                false
            }
        } else {
            false
        };

        if has_target_match || matches_cloned_exe {
            instance_root_pids.insert(pid_u32);
            matched_pids.push(pid_u32);
        }
    }

    // Lineage inheritance for children
    for proc in &processes {
        let pid_u32 = proc.pid;
        if matched_pids.contains(&pid_u32) {
            continue;
        }
        let mut curr = pid_u32;
        for _ in 0..6 {
            if instance_root_pids.contains(&curr) {
                matched_pids.push(pid_u32);
                break;
            }
            if let Some(&parent) = parent_map.get(&curr) {
                curr = parent;
            } else {
                break;
            }
        }
    }

    matched_pids
}
```

---

### Step 3: Closed-PID OS Table Re-Scan in `is_instance_process_running_smart`

**File:** `src-tauri/src/modules/instance.rs` (lines 3348–3375)

#### Key Improvements:
1. When `check_cached_pid_alive` returns `None`, force a refresh of the OS process table via `force_refresh_process_cache()`.
2. Re-scan via `scan_instance_os_pids(inst, &canonical_id)`.
3. If living PIDs are detected:
   - Cache them via `cache_live_instance_process`.
   - Return `(true, primary_pid, pids)` immediately.
4. Only return `(false, None, Vec::new())` if the OS process table confirms zero active processes.

---

### Step 4: Removal of Destructive `close_instance` from Prompt Dispatch

**File:** `src-tauri/src/modules/instance.rs` (lines 3390–3426)

#### Key Improvements:
1. In `ensure_instance_running_smart`:
   - If `is_running == true`: Call `focus_running_instance(&pids, workspace_path)` and return `Ok((true, primary_pid))`. Never call `launch_instance_with_workspaces`.
   - If `is_running == false`: Log that the instance is confirmed offline before initiating cold launch.
2. In `focus_or_launch_workspace`:
   - Enforce that focus is the primary action and cold launch is strictly an offline fallback.

---

### Step 5: Frontend Dispatch De-duplication

**File:** `src/components/instances/PromptTreeViewModal.tsx` (lines 1705–1725)

#### Current Problem:
```typescript
// 1. Dispatch prompt directly to running instance via sendPromptNow
await sendPromptNow(targetInstId, repoPath, promptContent, conv?.conversation_id);

// 2. Copy prompt content to clipboard
await navigator.clipboard.writeText(promptContent);

// 4. Redundant focus call causing race condition
if (repoPath) {
    const repoName = repoPath.split(/[/\\]/).filter(Boolean).pop() || repoPath;
    await focusInstanceWorkspace(targetInstId, repoPath, repoName);
}
```

#### Proposed Fix:
Remove the redundant `focusInstanceWorkspace` call. `sendPromptNow` handles both prompt persistence (`.antigravity_resume_task.json`) and non-destructive window focus in a single coordinated backend call.

---

## 3. Verification & Testing Checklist

- [ ] **Multi-PID Promotion Test**: Terminate the primary launcher PID while leaving a child worker process alive; verify `check_cached_pid_alive` promotes the child PID and maintains running status.
- [ ] **Zero-Relaunch Test**: Trigger "Send Now" while IDE is active; verify the active window is focused and no `taskkill` or duplicate launch occurs.
- [ ] **Windows 8.3 Path Test**: Configure an instance data directory with short path components; verify `find_pids_for_data_dir` resolves the process.
- [ ] **Host Shielding**: Verify test runs never terminate developer IDE instances.

---

## 4. Risk Assessment & Rollback

| Risk | Impact | Mitigation |
| :--- | :--- | :--- |
| Child PID misattribution | Could falsely keep instance marked running if PID reused | `saved_pid_matches(pid)` ensures the process executable matches Antigravity |
| Stale process table snapshot | Misses newly spawned window | `force_refresh_process_cache()` called before OS table scan |
| Rollback Plan | Revert edits in `src-tauri/src/modules/instance.rs` and `PromptTreeViewModal.tsx` via Git |
