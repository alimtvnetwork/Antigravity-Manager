# Specification 119: macOS Installation, Gatekeeper Quarantine Bypass, IDE Discovery Telemetry, and Process Launching Parity

## 1. Objectives & Non-Negotiables

1. **Zero "Move to Trash" Errors**: Eliminate macOS Gatekeeper / AMFI quarantine errors on unnotarized bundles via pre-mount attribute stripping, post-install recursive quarantine removal (`xattr -cr`), and local ad-hoc Mach-O codesigning (`codesign --force --deep --sign -`).
2. **Permission-Resilient Installation**: Provide seamless fallback from `/Applications` to `$HOME/Applications/` without requiring `sudo` or interactive stdin.
3. **Dynamic Bundle Discovery**: Scan mounted DMG volumes dynamically (`find "$mount_point" -maxdepth 2 -name "*.app" -type d`) rather than relying on hardcoded bundle names.
4. **macOS IDE Launch & Switch Parity**: Ensure `/usr/bin/open` strictly orders custom flags after `--args`, branch launcher between `.app` bundles (`open -n -a`) and shell scripts (`Command::new()`), and achieve 100% parity with Windows/Linux instance environments (`.gemini` directories, `app_storage.json`).
5. **Proactive First-Time IDE Discovery & Stack Trace Telemetry**: Discover IDE across running processes, standard paths, user applications, and Spotlight (`mdfind`) upon first startup, persist to `gui_config.json`, and capture Rust `Backtrace` on unexpected failures.
6. **POSIX Error Traps & Diagnostics**: Maintain robust POSIX `ERR` traps in `install.sh` and DMG packaging scripts to provide transparent error tracing.

---

## 2. Component Design & Technical Specifications

### A. DevOps & Installer (`install.sh`, `scripts/Fix_Damaged.command`, `scripts/package_dmg.sh`)
1. **Quarantine Stripping Sequence**:
   - `xattr -cr "$DOWNLOAD_PATH"` & `xattr -r -d com.apple.quarantine "$DOWNLOAD_PATH"`
   - `hdiutil attach "$DOWNLOAD_PATH" -nobrowse -noautoopen`
   - `source_app=$(find "$mount_point" -maxdepth 2 -name "*.app" -type d | head -n1)`
   - `dest_dir="/Applications"` (fallback to `"${HOME}/Applications"`)
   - `cp -R "$source_app" "$dest_dir/"`
   - `hdiutil detach "$mount_point" -force -quiet`
   - `xattr -cr "$target_app"` & `xattr -r -d com.apple.quarantine "$target_app"`
   - `codesign --force --deep --sign - "$target_app"`
2. **CLI Symlinking**:
   - Locate binary in `${target_app}/Contents/MacOS/` (`agm`, `agm-alim`, or main binary).
   - Create symlinks `${HOME}/.local/bin/agm` and `${HOME}/.local/bin/agm-alim`.
   - Ensure `${HOME}/.local/bin` is exported in `.zshrc`, `.bashrc`, and `.bash_profile`.
3. **Bilingual Fix Utility (`Fix_Damaged.command`)**:
   - Standalone shell script inside DMG with execution permissions (`chmod +x`).
   - Dynamically discovers `.app` locally or in `/Applications`, strips quarantine attributes, and applies ad-hoc codesigning.

### B. Backend macOS Process Management (`src-tauri/src/modules/process.rs`)
1. **Open Argument Construction**:
   ```rust
   pub(crate) fn format_macos_open_args(
       app_target: &str,
       custom_args: Option<&[String]>,
       new_window: bool,
   ) -> Vec<String> {
       let mut open_args = vec!["-n".to_string(), "-a".to_string(), app_target.to_string()];
       let mut app_args = Vec::new();
       if let Some(user_args) = custom_args {
           app_args.extend(user_args.iter().cloned());
       }
       if new_window {
           app_args.push("--new-window".to_string());
       }
       if !app_args.is_empty() {
           open_args.push("--args".to_string());
           open_args.extend(app_args);
       }
       open_args
   }
   ```
2. **First-Time IDE Discovery & Persistence**:
   - In `discover_and_persist_initial_ide_info()`:
     - Check existing `antigravity_executable` in `gui_config.json`.
     - If empty or invalid, probe running processes (`find_antigravity_process`).
     - If not running, probe standard paths (`get_macos_candidate_paths`) and Spotlight `mdfind`.
     - On discovery, save to `gui_config.json`.
     - On failure, capture `std::backtrace::Backtrace::capture()` and log detailed path checklist.

### C. macOS Instance Launching & Switching (`src-tauri/src/modules/instance.rs`)
1. **Launcher Branching**:
   - Check if executable ends with `.app` or is a directory:
     - If `.app`: invoke `Command::new("open")` with `format_macos_open_args`.
     - If executable file/script (e.g. `bin/antigravity-<id>`): invoke `Command::new(&exe_str)` directly.
2. **Configuration Parity**:
   - Create `.gemini/antigravity-ide` and `.gemini/antigravity` in instance home.
   - Synchronize `app_storage.json` with `ide-install-wizard-shown: true` and bound account email.
   - Set environment variables: `HOME`, `SSH_*`, OAuth tokens.
