# Subtask 01: AGM Update CLI Self-Delegation & File-Lock Crash Prevention

> **Status:** Pending Execution  
> **Owner:** `@aukgit`  
> **Parent Spec:** `02-spec/21-app/update-cli-command-and-pipeline-fixes/01-architecture-spec.md`  
> **Estimated Scope:** 5 files (`src-tauri/src/bin/agm/update_cmds.rs`, `src-tauri/src/modules/delegate_updater/process.rs`, `src-tauri/src/modules/delegate_updater/run.rs`, `src-tauri/src/modules/delegate_updater/prepare.rs`, `install.ps1`)  

---

## 1. Objectives & Context

Eliminate Windows executable file-locking deadlocks (`ERROR_ACCESS_DENIED`, os error 5 / 32) when running `agm update` from any terminal or background task by:
1. Enforcing universal self-delegation in `agm update`: when running on Windows outside `%TEMP%\agm-updater\`, immediately clone the active binary to `%TEMP%\agm-updater\agm-update-cli-<pid>.exe`, spawn the delegated copy in a detached console with `--wait-pid <pid>`, and terminate the calling process immediately.
2. Expanding the updater process taxonomy in `src-tauri/src/modules/delegate_updater/process.rs` and `install.ps1` to cover `antigravity-manager.exe` and `agm.exe` alongside legacy binary aliases.
3. Synchronizing `$lockCandidates` in `install.ps1` to rename `antigravity-manager.exe` and `agm.exe` to `.old` before executing installer binaries.

---

## 2. Target Files & Line-by-Line Changes

### 2.1 File: `src-tauri/src/bin/agm/update_cmds.rs`
- **Location:** Lines 380–416 (`cmd_update`).
- **Current Defect:**
  ```rust
  let ui_running = antigravity_tools_lib::modules::delegate_updater::is_ui_process_running_excluding(std::process::id());
  if ui_running && !is_json {
      // delegates only if UI is running
      antigravity_tools_lib::modules::delegate_updater::run(&delegate_args);
      return;
  }
  // If UI is not running, executes run_cli_update in-process!
  updated = antigravity_tools_lib::modules::delegate_updater::run_cli_update(&cli_update_args);
  ```
- **Required Modification:**
  Remove the `if ui_running` constraint on delegation. On Windows, check if the current binary is running from an active installation directory or if self-delegation is required:
  ```rust
  #[cfg(target_os = "windows")]
  {
      let current_exe = std::env::current_exe().unwrap_or_default();
      let is_running_from_temp = current_exe
          .to_string_lossy()
          .to_lowercase()
          .contains("agm-updater")
          || args.iter().any(|a| a == "--delegated-worker");

      if !is_running_from_temp && !is_check && !is_help {
          let my_pid = std::process::id();
          if !is_json {
              println!("[*] Self-delegating update task to isolated temp CLI copy...");
          }
          match antigravity_tools_lib::modules::delegate_updater::prepare_isolated_update_cli(my_pid) {
              Ok(temp_cli) => {
                  let mut delegate_args = vec![
                      "--relaunch".to_string(),
                      "--version".to_string(),
                      tag_name.clone(),
                  ];
                  if is_force || has_explicit_target {
                      delegate_args.push("--force".to_string());
                  }
                  match antigravity_tools_lib::modules::delegate_updater::spawn_delegated_update_cli(
                      &temp_cli,
                      my_pid,
                      &resolved_install_dir,
                      &resolved_target_exe,
                      true,
                      Some(&tag_name),
                  ) {
                      Ok(child_pid) => {
                          if !is_json {
                              println!(
                                  "[OK] Delegated to {:?} (PID {}). Exiting caller so update can proceed.",
                                  temp_cli, child_pid
                              );
                          }
                          std::process::exit(0);
                      }
                      Err(e) => {
                          eprintln!("[WARN] Could not spawn delegated copy ({}), continuing in-process...", e);
                      }
                  }
              }
              Err(e) => {
                  eprintln!("[WARN] Could not prepare isolated CLI copy ({}), continuing in-process...", e);
              }
          }
      }
  }
  ```

### 2.2 File: `src-tauri/src/modules/delegate_updater/process.rs`
- **Location:** Lines 55–80 (`is_ui_process_running_excluding`) and Lines 100–133 (`kill_other_ui_processes`).
- **Current Defect:**
  Only checks `["agm-alim.exe", "antigravity-tools.exe"]` and ignores `antigravity-manager.exe` and `agm.exe`.
- **Required Modification:**
  ```rust
  pub fn is_ui_process_running_excluding(exclude_pid: u32) -> bool {
      #[cfg(target_os = "windows")]
      {
          let process_names = [
              "antigravity-manager.exe",
              "agm.exe",
              "agm-alim.exe",
              "antigravity-tools.exe",
          ];
          for proc_name in process_names {
              let output = Command::new("tasklist.exe")
                  .args(["/FI", &format!("IMAGENAME eq {}", proc_name), "/NH"])
                  .output();
              if let Ok(out) = output {
                  let text = String::from_utf8_lossy(&out.stdout);
                  for line in text.lines() {
                      let trimmed = line.trim();
                      if trimmed.is_empty()
                          || trimmed.to_lowercase().starts_with("info:")
                          || trimmed.contains("No tasks")
                      {
                          continue;
                      }
                      if trimmed.contains(proc_name) {
                          if exclude_pid > 0 && trimmed.contains(&exclude_pid.to_string()) {
                              continue;
                          }
                          return true;
                      }
                  }
              }
          }
          false
      }
      #[cfg(not(target_os = "windows"))]
      {
          let targets = ["antigravity-manager", "agm", "agm-alim"];
          for tgt in targets {
              let output = Command::new("pgrep").args(["-f", tgt]).output();
              if let Ok(out) = output {
                  let text = String::from_utf8_lossy(&out.stdout);
                  for line in text.lines() {
                      if let Ok(pid) = line.trim().parse::<u32>() {
                          if pid != exclude_pid && pid != std::process::id() {
                              return true;
                          }
                      }
                  }
              }
          }
          false
      }
  }

  pub fn kill_other_ui_processes(exclude_pid: u32) {
      #[cfg(target_os = "windows")]
      {
          let filter = if exclude_pid > 0 {
              format!("PID ne {}", exclude_pid)
          } else {
              "PID gt 0".to_string()
          };
          let procs = [
              "antigravity-manager.exe",
              "agm.exe",
              "agm-alim.exe",
              "antigravity-tools.exe",
          ];
          for p in procs {
              // Justification: best-effort process termination; failure logged
              crate::error::record_ignored(
                  Command::new("taskkill.exe")
                      .args(["/F", "/IM", p, "/FI", &filter])
                      .output(),
                  "spawn taskkill.exe",
              );
          }
      }
      #[cfg(not(target_os = "windows"))]
      {
          let _ = exclude_pid;
          let targets = ["antigravity-manager", "agm", "agm-alim"];
          for tgt in targets {
              // Justification: best-effort process termination; failure logged
              crate::error::record_ignored(
                  Command::new("pkill").args(["-f", tgt]).output(),
                  "spawn pkill",
              );
          }
      }
  }
  ```

### 2.3 File: `install.ps1`
- **Location:** Lines 275–281 (`Close-ToolProcesses`) and Lines 1577–1585 (`$lockCandidates`).
- **Current Defect:** Missing `"antigravity-manager"` and `"agm"` in process list; missing `antigravity-manager.exe` and `agm.exe` in `$lockCandidates`.
- **Required Modification:**
  In `Close-ToolProcesses`:
  ```powershell
  $processNames = @(
      "antigravity-manager",
      "agm",
      "agm-alim",
      "antigravity-tools",
      "Anti-Gravity Tools",
      "Anti-Gravity Tools by Alim",
      "AGM by Alim"
  )
  ```
  In `$lockCandidates`:
  ```powershell
  $lockCandidates = @(
      (Join-Path $InstallDir $BinaryName),
      (Join-Path $InstallDir "antigravity-manager.exe"),
      (Join-Path $InstallDir "agm.exe"),
      (Join-Path $InstallDir "agm-alim.exe"),
      (Join-Path $InstallDir "AGM by Alim.exe"),
      (Join-Path $InstallDir "Anti-Gravity Tools by Alim.exe"),
      (Join-Path $InstallDir "antigravity-tools.exe")
  )
  ```

---

## 3. Input & Output Structures

### Input Parameters for `prepare_isolated_update_cli`:
- `caller_pid: u32` - Process ID of the active calling process to embed in the isolated temp executable name.

### Output Structure:
- `Result<PathBuf, String>` - Absolute path to `%TEMP%\agm-updater\agm-update-cli-<pid>.exe`.

### Execution Arguments for `spawn_delegated_update_cli`:
- `--delegated-worker`
- `--wait-pid <caller_pid>`
- `--install-dir <dir>`
- `--target-exe <exe>`
- `--relaunch` or `--no-relaunch`
- `--version <ver>` (optional)

---

## 4. Acceptance Criteria

1. **AC-01 (Caller Instant Termination):** When executing `agm update` from an installed location on Windows, the caller process terminates with code 0 within 500ms after launching the temp updater.
2. **AC-02 (File Handle Clearance):** After caller termination, `agm.exe` and `antigravity-manager.exe` in the installation directory have 0 active OS file handles, allowing unrestricted write/overwrite.
3. **AC-03 (Process Taxonomy Coverage):** Running instances of `antigravity-manager.exe` are correctly detected by `is_ui_process_running_excluding`, terminated before file extraction, and relaunched upon update completion.
4. **AC-04 (Installer Rename Protection):** `install.ps1` renames both `antigravity-manager.exe` and `agm.exe` to `.old` prior to unpacking, preventing NSIS extraction errors.
5. **AC-05 (Code Quality & Attribution):** All touched modules adhere strictly to positive boolean conventions (`is_running_from_temp`, `is_ui_process_running`) and maintain maintainer attribution strictly to `@aukgit`.

---

## 5. Verification Plan

1. **Compilation & Clippy Verification:**
   ```powershell
   cd src-tauri
   cargo clippy --bin agm --all-features
   ```
2. **Process Discovery Unit Test:**
   ```powershell
   cd src-tauri
   cargo test modules::delegate_updater::tests
   ```
3. **Simulated Update Delegation Run:**
   ```powershell
   # Run agm update with check flag to verify parsing and path resolution
   target\debug\agm.exe update --check
   ```
