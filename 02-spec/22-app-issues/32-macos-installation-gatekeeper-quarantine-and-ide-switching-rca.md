# App Issue RCA 32: macOS Gatekeeper "Move to Trash" Quarantine, Installation Failures, and IDE Switching Root Cause Analysis

## 1. Executive Summary & Problem Classification

When users attempt to install, launch, or switch Antigravity Manager Tools on macOS (macOS 12 Monterey, macOS 13 Ventura, macOS 14 Sonoma, and macOS 15 Sequoia), they encounter severe blockers:
1. **"App is Damaged and Can't Be Opened. You Should Move it to the Trash"**:
   Double-clicking the downloaded application bundle or DMG install causes Gatekeeper and Apple Mobile File Integrity (AMFI) to refuse execution, showing a fatal prompt instructing the user to move the app to Trash.
2. **Piped Shell Installer (`curl ... | bash`) Failures**:
   Piped shell installations consume standard input, causing `sudo` prompts to fail silently. When `/Applications` permissions are restricted or an older version is present with root ownership, the installation fails or results in half-copied bundles.
3. **IDE Switching & Launch Argument Collisions**:
   Launching Antigravity via `/usr/bin/open` with custom arguments (`--new-window`, `--user-data-dir`) causes `open` to error out (`unrecognized option`) because arguments were placed before `--args`. Cloned instances running via wrapper shell scripts fail because `open -a` only accepts `.app` bundles.
4. **Missing First-Time IDE Discovery & Diagnostic Traces**:
   When the IDE is located in custom user paths or Spotlight directories, the app fails silently without persisting the discovered path or capturing Rust backtraces.

---

## 2. Windows vs. macOS Comparative Architectural Audit

| Dimension | Windows (`install.ps1`, `src-tauri`) | macOS (`install.sh`, `src-tauri`) | Root Defect & Divergence |
| :--- | :--- | :--- | :--- |
| **Download Quarantine** | `Zone.Identifier` ADS is attached, but NSIS installers and PowerShell bypass it cleanly. | `com.apple.quarantine` extended attribute is attached to DMG and extracted `.app`. | Unnotarized apps with `com.apple.quarantine` trigger AMFI fatal "Move to Trash" dialog. |
| **Execution Security** | SmartScreen warns on unrecognized publishers, but offers "Run Anyway". | Gatekeeper / AMFI blocks execution completely on macOS 14+ & 15+ without notarization ticket or local ad-hoc signature. | Without `codesign --force --deep --sign -`, macOS classifies unsigned Mach-O binaries as damaged. |
| **Installation Destination** | Installs to user-writable `%LOCALAPPDATA%\Programs`, avoiding admin/UAC elevation. | Hardcoded `/Applications/`, which may be owned by `root:admin` or read-only in restricted environments. | Failed write to `/Applications` crashed the script instead of falling back to `$HOME/Applications/`. |
| **App Bundle Discovery** | Probes known executable names (`agm-alim.exe`, `Antigravity Tools.exe`). | Hardcoded `Antigravity Manager Tools.app`, failing if DMG contains differently cased or named bundles. | Needs dynamic discovery via `find "$mount_point" -maxdepth 2 -name "*.app" -type d`. |
| **Process Launching** | Directly executes `.exe` with `Command::new` passing arguments as a vector. | Uses `/usr/bin/open -n -a "<app>" [args]`. | `/usr/bin/open` requires application arguments to follow `--args`. Passing `--new-window` before `--args` causes `open` syntax error. |
| **Cloned Instance Launch** | Executes target `.exe` directly. | Invoked `open -n -a "<script>"` on wrapper scripts (`bin/antigravity-<id>`). | `open -a` strictly rejects shell scripts; scripts must be launched directly via `Command::new`. |
| **Error Telemetry** | Rich error logs in `logs/` and panic hooks. | Shell errors were silenced with `2>/dev/null \|\| true`. | Failures during install or IDE detection lacked stack traces and diagnostic context. |

---

## 3. The 5 Root Causes & Engineering Remedies

### Root Cause 1: Gatekeeper Quarantine & AMFI Enforcement (`com.apple.quarantine`)
* **Mechanism**: macOS attaches `com.apple.quarantine` to all files downloaded from the web. When an application bundle has this attribute and lacks an Apple Developer ID signature + Notarization ticket from Apple's servers, Gatekeeper and AMFI flag the bundle as damaged and refuse to launch it.
* **Remedy**:
  1. Strip quarantine before mounting: `xattr -cr "$DOWNLOAD_PATH"` and `xattr -r -d com.apple.quarantine "$DOWNLOAD_PATH"`.
  2. Strip quarantine from destination bundle: `xattr -cr "$target_app"` and `xattr -r -d com.apple.quarantine "$target_app"`.
  3. Apply local ad-hoc signature: `codesign --force --deep --sign - "$target_app"`. This marks the Mach-O binary as valid to local AMFI, completely bypassing the "Move to Trash" prompt.

### Root Cause 2: Piped Installer Sudo Stall & Destination Permissions
* **Mechanism**: Running `curl ... | bash` consumes `stdin`. Any command invoking `sudo` fails because it cannot read user password input. If `/Applications` is not writable, `cp -R` fails with `Permission denied`.
* **Remedy**:
  1. Test write permissions on `/Applications`.
  2. If unwritable, gracefully fall back to user-scoped `$HOME/Applications/` without requiring `sudo`.
  3. Strip quarantine at user level without sudo privileges.

### Root Cause 3: `/usr/bin/open` Argument Sequence Collision
* **Mechanism**: `/usr/bin/open` syntax is `open -n -a "<app>" --args [app_arguments...]`. In `process.rs`, `--new-window` was passed directly to `open`, causing `open: unrecognized option '--new-window'`.
* **Remedy**: Centralize argument formatting via `format_macos_open_args`, guaranteeing all application parameters (`--new-window`, `--user-data-dir`) strictly follow `--args`.

### Root Cause 4: Shell Script vs. App Bundle Execution Mismatch
* **Mechanism**: Cloned instances generate a launcher script in `bin/antigravity-<id>`. Calling `open -n -a` on this script fails because `open -a` expects an `.app` bundle directory.
* **Remedy**: In `src-tauri/src/modules/instance.rs`, inspect the executable target: if it is an `.app` bundle, use `open -n -a`; if it is a binary or shell script, execute it directly with `Command::new()`.

### Root Cause 5: Silent Error Swallowing & Missing Stack Traces
* **Mechanism**: Installation and detection errors were swallowed with `2>/dev/null || true`, hiding the root cause when installation failed.
* **Remedy**:
  1. In `install.sh`: Implement POSIX `ERR` trap `report_error_stack "$LINENO"` displaying line number, failing command, and call stack.
  2. In `process.rs` and `lib.rs`: Proactively detect IDE on first startup, persist to `gui_config.json`, and capture `std::backtrace::Backtrace` on failures.
