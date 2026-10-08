# Subtask 01: Smart Process Cache, Closed-PID Re-Scan & agy Dispatch

- **Subtask ID**: `148-01`
- **Parent Task**: `148-ide-prompt-dispatch-process-cache-and-bracket-cleanup`
- **Target Files**:
  - `src-tauri/src/modules/instance.rs`
  - `src-tauri/src/modules/repo_db.rs`
  - `src-tauri/src/modules/process.rs`
  - `src/components/instances/PromptTreeViewModal.tsx`
- **Maintainer / Attribution**: Strictly `@aukgit` (`(Thanks to @aukgit)`)
- **Status**: READY_FOR_EXECUTION

---

## 1. Objective

Implement the user-mandated 3-step smart process caching and closed-PID rescan lifecycle in `src-tauri/src/modules/instance.rs`, fix the `close_instance` argument refresh bug, inject `--user-data-dir` into `spawn_prompt_via_agy` in `src-tauri/src/modules/repo_db.rs`, implement the cross-platform system clipboard helper, and eliminate the frontend double-launch race condition in `PromptTreeViewModal.tsx`.

---

## 2. Context & Root Cause Analysis

1. **Unwanted Repeated IDE Restarts**:
   - When a user sends or enqueues a prompt, AGM previously failed to verify whether the target instance process was already alive.
   - When an instance process had terminated or reloaded under a new PID, AGM immediately initiated a new launch instead of rescanning the OS process table to detect reloaded PIDs.
2. **`close_instance` Sysinfo Bug**:
   - `sysinfo::System::refresh_processes` failed to refresh command-line arguments. Consequently, `proc.cmd()` returned empty strings, preventing `args_str.contains(clean_data)` from matching the instance data directory and causing termination requests to fail silently.
3. **`spawn_prompt_via_agy` Missing `--user-data-dir`**:
   - Non-default profiles did not pass `--user-data-dir <dir>` when spawning `agy`, causing the CLI tool to connect to the default configuration path instead of the active instance's IPC socket.
4. **Frontend Double-Launch Race**:
   - `handleResendPrompt` in `PromptTreeViewModal.tsx` invoked both `sendPromptNow` (which performs smart launch on the backend) and `focusOrLaunchInstance`, launching two redundant windows.

---

## 3. Detailed Implementation Plan

### Step 1: Implement `scan_and_cache_all_running_instances()` in `src-tauri/src/modules/instance.rs`
- Query the OS process table with full command arguments refreshed (`.with_cmd(sysinfo::UpdateKind::Always)`).
- Iterate through all registered instances in `instances.json` and count how many instances are running (`total_running_instances`).
- For each running instance, extract all live PIDs and designate the primary PID.
- Store results in `INSTANCE_PROCESS_CACHE`:
  ```rust
  pub fn scan_and_cache_all_running_instances() -> (usize, std::collections::HashMap<String, InstanceProcessCacheItem>)
  ```
- Expose `get_instance_running_process_count() -> usize` backed by this scan.

### Step 2: Implement Closed-PID Re-Scan & Smart Ensure Protocol
- In `is_instance_process_running_smart(instance_id: &str) -> (bool, Option<u32>, Vec<u32>)`:
  - **Check Cache**: If entry exists, check if `primary_pid` is alive via `is_pid_alive_os`.
  - **Closed-PID Detection**: If cached PID is closed (`is_pid_alive_os` returns `false`), do **not** immediately launch.
  - **OS Rescan**: Immediately rescan the OS process table for any processes whose command arguments match the target instance's `--user-data-dir`.
  - If a matching PID is found on rescan:
    - Update `INSTANCE_PROCESS_CACHE` with the newly discovered PID.
    - Set `is_running = true`.
    - Return `(true, Some(new_pid), new_pids)`.
  - If no matching PID is found after OS rescan:
    - Invalidate cache entry.
    - Return `(false, None, Vec::new())`.
- In `ensure_instance_running_smart(instance_id: &str, workspace_path: Option<&str>)`:
  - Call `is_instance_process_running_smart`.
  - If `is_running` is true: focus existing window and return `Ok((true, primary_pid))` without reopening.
  - If `is_running` is false: only then invoke `launch_instance_with_workspaces` (or `launch_instance`), verify the newly spawned PID, cache it, and return `Ok((true, new_pid))`.

### Step 3: Fix Sysinfo Command Argument Refresh in `close_instance`
- In `close_instance(instance_id: &str)` in `src-tauri/src/modules/instance.rs`:
  - Replace `system.refresh_processes(sysinfo::ProcessesToUpdate::All)` with:
    ```rust
    system.refresh_processes_specifics(
        sysinfo::ProcessesToUpdate::All,
        sysinfo::ProcessRefreshKind::new()
            .with_cmd(sysinfo::UpdateKind::Always)
            .with_exe(sysinfo::UpdateKind::Always),
    );
    ```
  - Ensures `proc.cmd()` is populated with `--user-data-dir` arguments so `close_instance` accurately targets and terminates the intended instance processes.

### Step 4: Inject `--user-data-dir` into `spawn_prompt_via_agy`
- In `src-tauri/src/modules/repo_db.rs` within `spawn_prompt_via_agy`:
  - Check if `prompt.instance_id` is non-default:
    ```rust
    if prompt.instance_id != "default" && !prompt.instance_id.is_empty() {
        if let Ok(registry) = crate::modules::instance::load_registry() {
            if let Some(inst) = registry.instances.iter().find(|i| i.id == prompt.instance_id) {
                cmd.arg("--user-data-dir").arg(&inst.data_dir);
            }
        }
    }
    ```
  - Ensures `agy` commands connect directly to the active instance's IPC domain socket.

### Step 5: Implement Cross-Platform `copy_to_system_clipboard`
- In `src-tauri/src/modules/instance.rs`:
  ```rust
  pub fn copy_to_system_clipboard(prompt_content: &str) -> Result<(), String> {
      #[cfg(target_os = "windows")]
      {
          use std::os::windows::process::CommandExt;
          use std::io::Write;
          let mut child = std::process::Command::new("clip")
              .creation_flags(0x08000000)
              .stdin(std::process::Stdio::piped())
              .spawn()
              .map_err(|e| e.to_string())?;
          if let Some(ref mut stdin) = child.stdin {
              let _ = stdin.write_all(prompt_content.as_bytes());
          }
          let _ = child.wait();
      }
      #[cfg(target_os = "macos")]
      {
          use std::io::Write;
          let mut child = std::process::Command::new("pbcopy")
              .stdin(std::process::Stdio::piped())
              .spawn()
              .map_err(|e| e.to_string())?;
          if let Some(ref mut stdin) = child.stdin {
              let _ = stdin.write_all(prompt_content.as_bytes());
          }
          let _ = child.wait();
      }
      #[cfg(target_os = "linux")]
      {
          use std::io::Write;
          let copied = if let Ok(mut child) = std::process::Command::new("wl-copy")
              .stdin(std::process::Stdio::piped())
              .spawn()
          {
              if let Some(ref mut stdin) = child.stdin {
                  let _ = stdin.write_all(prompt_content.as_bytes());
              }
              child.wait().is_ok()
          } else {
              false
          };

          if !copied {
              if let Ok(mut child) = std::process::Command::new("xclip")
                  .args(["-selection", "clipboard"])
                  .stdin(std::process::Stdio::piped())
                  .spawn()
              {
                  if let Some(ref mut stdin) = child.stdin {
                      let _ = stdin.write_all(prompt_content.as_bytes());
                  }
                  let _ = child.wait();
              }
          }
      }
      Ok(())
  }
  ```
- Integrate `copy_to_system_clipboard` into `send_prompt_now_for_instance`.

### Step 6: Connect 3-Step Sequence to `send_prompt_now` & `enqueue_prompt`
- In `send_prompt_now_for_instance`:
  - Step 1: Call `scan_and_cache_all_running_instances()`.
  - Step 2: Call `ensure_instance_running_smart(instance_id, Some(repo_path))`.
  - Step 3: Call `copy_to_system_clipboard(prompt_content)`.
  - Step 4: Dispatch via `spawn_prompt_via_agy(&active_prompt)`.
- In `enqueue_prompt_for_instance`:
  - Step 1: Call `scan_and_cache_all_running_instances()`.
  - Step 2: Save to SQLite `active_prompts` with status `'queued'`.
  - Step 3: Write `.antigravity_resume_task.json` with status `'queued'`.

### Step 7: Eliminate Frontend Double-Launch in `PromptTreeViewModal.tsx`
- In `src/components/instances/PromptTreeViewModal.tsx`:
  - In `handleResendPrompt`, remove the redundant second invocation:
    ```typescript
    // REMOVE THIS DUPLICATE CALL:
    // await focusOrLaunchInstance(targetInstId, repoPath);
    ```
  - Retain `sendPromptNow`, clipboard synchronization, and user feedback message.

---

## 4. Verification & Quality Gates

- [ ] **Positive Booleans**: Verify all predicates use `is_running`, `is_alive`, `is_cached`, `has_valid_pid`.
- [ ] **Path Hygiene**: Confirm zero absolute paths or file protocol URIs in modified files.
- [ ] **Rust Formatting**: `cargo fmt -- --check` passes cleanly.
- [ ] **Rust Clippy**: `cargo clippy --all-targets --all-features` passes with zero warnings.
- [ ] **Frontend Build**: `npm run build` succeeds without type errors.
