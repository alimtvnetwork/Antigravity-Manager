# 116 — macOS Installation, Gatekeeper Quarantine, and IDE Switching Architecture Specification

## 1. Domain & Scope
This specification governs cross-platform parity between Windows and macOS across two critical operational domains:
1. **Automated & Manual Package Installation**: Resolving Gatekeeper quarantine, code-signing, dynamic bundle discovery, destination permissions, and CLI symlinking in `install.sh`.
2. **IDE Discovery, First-Time Information Gathering & Process Switching**: Ensuring Antigravity IDE detection via process table, standard applications directories, and Spotlight (`mdfind`), fixing argument ordering (`--args` preceding `--new-window`), supporting both `.app` bundles and launcher shell scripts, and ensuring comprehensive stack trace capture.

---

## 2. Architectural Components & Protocols

### 2.1 Installer Script (`install.sh`)
- **Error Trapping**: Implement a POSIX `ERR` trap capturing line number, failed statement, and stack trace:
  ```bash
  report_error_stack() {
      local exit_code=$?
      local line_no=$1
      local cmd="$2"
      error "Command '$cmd' failed at line $line_no with exit code $exit_code"
  }
  trap 'report_error_stack $LINENO "$BASH_COMMAND"' ERR
  ```
- **Pre-Mount Quarantine Removal**:
  Run `xattr -cr "$DOWNLOAD_PATH"` on the DMG file before mounting.
- **Dynamic Bundle Discovery**:
  Search for any `.app` directory in the mounted volume:
  ```bash
  local found_app
  found_app=$(find "$mount_point" -maxdepth 2 -name "*.app" -type d | head -n1)
  ```
- **User-Level Destination Fallback**:
  Attempt copy to `/Applications/`. If unwritable:
  ```bash
  local dest_dir="/Applications"
  if [[ ! -w "$dest_dir" ]] && ! mkdir -p "$dest_dir" 2>/dev/null; then
      dest_dir="${HOME}/Applications"
      mkdir -p "$dest_dir"
  fi
  ```
- **Quarantine Removal & Ad-Hoc Code Signing**:
  ```bash
  xattr -r -d com.apple.quarantine "${dest_dir}/${app_name}" 2>/dev/null || true
  xattr -cr "${dest_dir}/${app_name}" 2>/dev/null || true
  if command -v codesign &>/dev/null; then
      codesign --force --deep --sign - "${dest_dir}/${app_name}" 2>/dev/null || true
  fi
  ```
- **CLI Symlinking**:
  Locate `agm` inside `.app/Contents/MacOS/agm` (or `agm-alim`) and create symlink in `${HOME}/.local/bin/agm`.

---

### 2.2 IDE Discovery Engine (`src-tauri/src/modules/process.rs`)
- **Spotlight `mdfind` Integration**:
  When standard directory scans fail on macOS, execute `mdfind` to locate `Antigravity.app` or `Antigravity IDE.app`:
  ```rust
  #[cfg(target_os = "macos")]
  fn find_via_spotlight(app_name: &str) -> Option<std::path::PathBuf> {
      let query = format!("kMDItemFSName == '{}.app'", app_name);
      let output = std::process::Command::new("mdfind").arg(&query).output().ok()?;
      if output.status.success() {
          let stdout = String::from_utf8_lossy(&output.stdout);
          for line in stdout.lines() {
              let p = std::path::PathBuf::from(line.trim());
              if p.exists() {
                  return Some(p);
              }
          }
      }
      None
  }
  ```
- **Arguments Construction for `/usr/bin/open`**:
  Ensure all application arguments strictly follow `--args`:
  ```rust
  let mut cmd = Command::new("open");
  cmd.arg("-n").arg("-a").arg(&app_path);
  cmd.arg("--args");
  if let Some(user_args) = args {
      for a in user_args { cmd.arg(a); }
  }
  if is_new_window {
      cmd.arg("--new-window");
  }
  ```

---

### 2.3 Instance Launching Parity (`src-tauri/src/modules/instance.rs`)
- **Launcher Script vs `.app` Bundle Branching**:
  Check if `exe_str` ends with `.app` or is a directory:
  - If `.app`: launch via `open -n -a "$exe_str" --args ...`
  - If executable file/script (e.g. `antigravity-{instance_id}`): execute directly via `Command::new(&exe_str)` with proper working directory and argument propagation.
- **Initialization Parity**:
  Ensure `.gemini/antigravity-ide`, `.gemini/antigravity`, `ide-install-wizard-shown: true`, and `update_instance_app_storage` execute on macOS exactly as on Windows/Linux.

---

## 3. Stack Trace & Diagnostic Telemetry
Whenever any step fails in IDE discovery, switching, or launching:
1. Capture `std::backtrace::Backtrace::capture()`.
2. Format into `AppError::IdeNotFound` or `AppError::InstanceSwitchFailed`.
3. Log structured logs: `[IDE Discovery]`, `[INSTANCE_SWITCH:ERROR]`.
