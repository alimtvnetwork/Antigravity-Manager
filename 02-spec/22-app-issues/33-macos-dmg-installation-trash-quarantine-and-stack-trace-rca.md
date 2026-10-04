# App Issue RCA 33: macOS DMG Installation Failure, Trash Deduplication ("... because it is in the Trash"), Gatekeeper Quarantine, and Diagnostic Stack Trace Gaps

## 1. Symptom & Reproduction Steps

When users attempt to install, launch, or update Antigravity Manager Tools on macOS (Ventura, Sonoma, Sequoia):
1. **Gatekeeper "Damaged" Prompt**: Opening the downloaded `.app` bundle from the DMG displays:
   `"Antigravity Manager Tools" is damaged and can't be opened. You should move it to the Trash.`
2. **Move to Trash & Timestamping**: Clicking "Move to Trash" moves `/Applications/Antigravity Manager Tools.app` into `~/.Trash/`. If a trashed copy already exists, macOS appends a timestamp:
   `~/.Trash/Antigravity Manager Tools 13-20-55-339.app`
3. **Fatal LaunchServices Trash Lockout**:
   Attempting to open the application (via double-clicking the DMG icon, dragging to `/Applications`, clicking the Dock/Spotlight shortcut, running `open -a "Antigravity Manager Tools"`, or via `install.sh` / auto-updater) fails with:
   `You can’t open the application “Antigravity Manager Tools 13-20-55-339” because it is in the Trash.`
4. **AppleScript Container Specifier Failure**:
   Previous attempts to bypass TCC via AppleScript used `move tItem to (POSIX file "/tmp") with replacing`. In Finder AppleScript, `POSIX file` without `as alias` is a raw file URL record (`furl`), not a Finder container. Finder threw error `-10010` (`Handler can't handle objects of this class`), silently failing to move items out of Trash.
5. **Script Syntax & Local Variable Error**:
   In `scripts/fix_app.sh`, `local bin_log` and `local bin_pid` were invoked outside of any shell function at the root script level, causing bash to abort on error.
6. **Missing Diagnostics & Stack Traces**:
   When DMG installation failed, users had insufficient diagnostic traces to pinpoint whether architecture mismatch (x86_64 vs arm64), dynamic library linkage, LaunchServices, codesign, or Gatekeeper caused the failure.

---

## 2. Root Cause Analysis (8-Part Root Mechanism)

### 2.1 Missing Apple Notarization & Gatekeeper Quarantine (`com.apple.quarantine`)
- **Mechanism**: The application binaries and DMG were built without an Apple Developer ID certificate or Apple notarization ticket. When downloaded via a browser or `curl`, macOS automatically sets the extended attribute `com.apple.quarantine`. Apple Mobile File Integrity (AMFI) flags unsigned quarantined apps as corrupted/damaged and prompts the user to move the app to Trash.

### 2.2 macOS Trash Collision Deduplication & Timestamp Naming
- **Mechanism**: When an app is moved to Trash while an item with that name already exists in `~/.Trash/`, macOS Finder / NSFileManager deduplicates the file by appending `HH-MM-SS-mmm` (e.g., `13-20-55-339`). This produces `Antigravity Manager Tools 13-20-55-339.app`.

### 2.3 LaunchServices Database Poisoning
- **Mechanism**: macOS LaunchServices caches bundle identifiers (e.g. `com.lbjlaq.antigravity-tools`). When an app is trashed, LaunchServices points the registered application path to `~/.Trash/Antigravity Manager Tools 13-20-55-339.app`. Any subsequent launch request directed at the application or bundle ID triggers LaunchServices to inspect the target; seeing it located inside `~/.Trash/`, macOS refuses execution and displays:
  `You can’t open the application “Antigravity Manager Tools 13-20-55-339” because it is in the Trash.`
  Even if a new copy is dragged into `/Applications`, LaunchServices remains bound to the trashed instance until the database unregisters the trashed path (`lsregister -u`), garbage collects dead entries across domains (`lsregister -gc -R -v -apps u,s,l`), and force-registers the new path (`lsregister -f -r`).

### 2.4 macOS TCC Privacy Protection on `~/.Trash` & AppleScript Alias Requirement
- **Mechanism**: On macOS 10.15+, `$HOME/.Trash` is protected by Transparency, Consent, and Control (TCC). Shell commands (`rm -rf ~/.Trash/*`) run from non-privileged Terminal sessions fail with `Operation not permitted`.
- **Flaw**: Using `move tItem to (POSIX file "/tmp")` failed because Finder's `move` command requires a folder alias object.
- **Resolution**: Use `set destFolder to (POSIX file "/private/tmp") as alias` with `move anItem to destFolder with replacing`. Clean `/private/tmp` before and after.

### 2.5 Silent `open` Error Handling (`open_err` Never Contains "Trash")
- **Mechanism**: When LaunchServices blocks a trashed app, the GUI alert displays the message, but the CLI command `open` returns:
  `LSOpenURLsWithRole() failed with error -10810 for the file ...`
  Because `open` stderr did not contain the substring `"Trash"`, conditional checks like `if [[ "$open_err" == *"Trash"* ]]` evaluated to false and skipped the recovery routine.
- **Resolution**: Unconditionally trigger LaunchServices and trash purge recovery whenever `open` returns non-zero.

### 2.6 Fallback Direct Binary stderr Discarded to `/dev/null`
- **Mechanism**: When `open` failed, scripts launched the core binary via `nohup "$internal_bin" >/dev/null 2>&1 &`. If the binary failed to launch (due to quarantine, dynamic linkage, or permissions), all error output and stack traces were discarded into `/dev/null`.
- **Resolution**: Redirect stdout/stderr to `$LOG_DIR/binary_launch.log`, verify process liveness with `kill -0`, and dump the log tail and system crash reports if the binary terminates.

### 2.7 Missing Stack Traces in Rust Panic & Updater Handlers
- **Mechanism**: `std::backtrace::Backtrace::capture()` was disabled in release mode without `RUST_BACKTRACE=1`.
- **Resolution**: Upgrade to `std::backtrace::Backtrace::force_capture()` across `logger.rs` and `delegate_updater.rs`, and persist panic logs and crash stack traces to `~/Library/Logs/AntigravityManager/crash_stacktrace.log`.

### 2.8 Native PKG Architecture & Preinstall Cleanup
- **Mechanism**: Drag-and-drop `.app` installation cannot execute code prior to launch. A native `.pkg` installer runs with root privileges, executing `preinstall` to clean trash and stale registrations before payload deployment, followed by `postinstall` to strip quarantine, add Gatekeeper trust, and re-register LaunchServices.
- **Resolution**: Provide `scripts/pkg-scripts/preinstall` and `scripts/pkg-scripts/postinstall`, build `Install_Antigravity_Manager.pkg`, and bundle inside DMG and release assets.

---

## 3. Engineering Resolution

1. **Working AppleScript TCC Eviction Pattern (`Install_Or_Repair.command`, `Fix_Damaged.command`, `fix_app.sh`, `install.sh`)**:
   - Switched to `set destFolder to (POSIX file "/private/tmp") as alias`.
   - Iterated over `every item of trash` matching `Antigravity` or `agm`.
   - Purged destination `/private/tmp` before and after move.
   - Cleared file flags via `chflags -R nouchg,noschg`.

2. **Full-Domain LaunchServices Regeneration**:
   - Replaced plain `lsregister -gc` with `lsregister -gc -R -v -apps u,s,l`.
   - Forced re-registration of target app via `lsregister -f -r "$TARGET_APP"`.
   - Reloaded Dock and Finder to synchronize UI state.

3. **PKG Preinstall & Postinstall Architecture**:
   - Created `scripts/pkg-scripts/preinstall`: terminates running processes, runs console user Finder AppleScript eviction, unregisters existing bundle, and purges stale `/Applications` directory.
   - Enhanced `scripts/pkg-scripts/postinstall`: strips quarantine recursively, enforces executable permissions, registers with Gatekeeper (`spctl --add`), refreshes LaunchServices across all domains, and creates CLI symlinks.

4. **Deep System Diagnostics & Crash Stack Traces**:
   - Enriched `dump_diagnostic_state`:
     - Hardware & Rosetta translation status (`machdep.cpu.brand_string`, `sysctl.proc_translated`).
     - Gatekeeper deep audit (`spctl -a -vvvv`).
     - Extended attributes with values (`xattr -lv`).
     - Codesign verification and entitlements dump (`codesign -d --entitlements :-`).
     - Crash reports and IPS thread backtraces from `~/Library/Logs/DiagnosticReports/` in the last 120 minutes.
     - Unified macOS system logs for LaunchServices and process lifecycles.
     - Direct binary launch stderr log tail (`binary_launch.log`).

5. **Rust Backend Hardening**:
   - Upgraded to `std::backtrace::Backtrace::force_capture()` across `logger.rs` and `delegate_updater.rs`.
   - In `logger.rs`, enriched panic hook with thread name, process ID, timestamp, and appended to `$HOME/Library/Logs/AntigravityManager/crash_stacktrace.log`.
   - In `delegate_updater.rs` and `lib.rs`, repaired AppleScript trash eviction and full-domain LaunchServices garbage collection.

---

## 4. Prevention & Quality Guards

1. **AppleScript Container Rule**: Always cast POSIX paths to `as alias` (e.g., `(POSIX file "/private/tmp") as alias`) when supplying target folders to Finder's `move` command.
2. **Unconditional LaunchServices Recovery**: Never gate LaunchServices / trash repair on specific error strings from `open` because OSStatus error codes (such as -10810) do not contain human-readable keywords in stderr.
3. **Mandatory Persistent Logging**: Never discard process output to `>/dev/null` in installation or repair scripts. Always pipe to `$LOG_DIR/*.log`.
4. **Crash Report Extraction**: Always inspect `~/Library/Logs/DiagnosticReports/` when diagnosing macOS process launch failures to obtain native system thread backtraces.
5. **Forced Backtraces**: Always use `Backtrace::force_capture()` in Rust panics and recovery blocks rather than `Backtrace::capture()` so backtraces are not dropped in release builds.
