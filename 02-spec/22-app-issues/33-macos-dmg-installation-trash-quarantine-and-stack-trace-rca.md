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
4. **Script Crash & Silent Failure**:
   When users attempted to run `Install_Or_Repair.command` or `Fix_Damaged.command`, `open` returned exit error code `-10810` (which does not contain the word `"Trash"` in stderr), causing scripts to skip the recovery branch. Furthermore, fallback binary launches discarded stderr to `/dev/null`, preventing diagnostics.
5. **Missing Diagnostics & Stack Traces**:
   Users who encountered DMG installation failure had no persistent logs or stack traces to diagnose whether `hdiutil`, `xattr`, `spctl`, `codesign`, or LaunchServices failed.

---

## 2. Root Cause Analysis (8-Part Root Mechanism)

### 2.1 Missing Apple Notarization & Gatekeeper Quarantine (`com.apple.quarantine`)
- **Mechanism**: The application binaries and DMG were built without an Apple Developer ID certificate or Apple notarization ticket. When downloaded via a browser or `curl`, macOS automatically sets the extended attribute `com.apple.quarantine`. Apple Mobile File Integrity (AMFI) flags unsigned quarantined apps as corrupted/damaged and prompts the user to move the app to Trash.

### 2.2 macOS Trash Collision Deduplication & Timestamp Naming
- **Mechanism**: When an app is moved to Trash while an item with that name already exists in `~/.Trash/`, macOS Finder / NSFileManager deduplicates the file by appending `HH-MM-SS-mmm` (e.g., `13-20-55-339`). This produces `Antigravity Manager Tools 13-20-55-339.app`.

### 2.3 LaunchServices Database Poisoning
- **Mechanism**: macOS LaunchServices caches bundle identifiers (e.g. `com.lbjlaq.antigravity-tools`). When an app is trashed, LaunchServices points the registered application path to `~/.Trash/Antigravity Manager Tools 13-20-55-339.app`. Any subsequent launch request directed at the application or bundle ID triggers LaunchServices to inspect the target; seeing it located inside `~/.Trash/`, macOS refuses execution and displays:
  `You can’t open the application “Antigravity Manager Tools 13-20-55-339” because it is in the Trash.`
  Even if a new copy is dragged into `/Applications`, LaunchServices remains bound to the trashed instance until the database unregisters the trashed path (`lsregister -u`), garbage collects dead entries (`lsregister -gc`), and force-registers the new path (`lsregister -f -r`).

### 2.4 macOS TCC Privacy Protection on `~/.Trash`
- **Mechanism**: On macOS 10.15+, `$HOME/.Trash` is protected by Transparency, Consent, and Control (TCC). Shell commands (`rm -rf ~/.Trash/*`) run from non-privileged Terminal sessions fail with `Operation not permitted`. Previous attempts to invoke `osascript -e 'tell application "Finder" to delete (every item of trash...)'` failed because Finder's `delete` command only moves items *into* trash, throwing an error when called on items already in the trash.
- **Resolution**: Use AppleScript to tell Finder to `move anItem to (POSIX file "/tmp") with replacing`. Once moved to `/tmp`, TCC restrictions do not apply, and `rm -rf /tmp/Antigravity*` cleanly wipes the files.

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
- **Resolution**: Upgrade to `std::backtrace::Backtrace::force_capture()` across `logger.rs` and `delegate_updater.rs`, and persist panic logs to `~/Library/Logs/AntigravityManager/panic.log`.

### 2.8 Native PKG Absence in DMG
- **Mechanism**: Drag-and-drop `.app` installation inherits DMG quarantine. A native `.pkg` installer runs with installer root privileges, executing `scripts/pkg-scripts/postinstall` to cleanly install, purge trash, strip quarantine, and refresh LaunchServices.
- **Resolution**: Build a native `.pkg` installer using `pkgbuild` with `postinstall` script and place it inside the DMG root as `Install_Antigravity_Manager.pkg` and publish as release asset.

---

## 3. Engineering Resolution

1. **Unconditional LaunchServices Recovery (`Install_Or_Repair.command` & `Fix_Damaged.command`)**:
   - Replaced conditional `open_err` check with unconditional recovery on any `open` error.
   - Bypassed TCC via AppleScript Finder move-to-`/tmp` then `rm -rf`.
   - Stripped immutable flags with `chflags -R nouchg,noschg`.
   - Executed `lsregister -gc`, `lsregister -u`, and `lsregister -f -r`.
   - Replaced silent `nohup >/dev/null` with persistent logging to `$LOG_DIR/binary_launch.log`.

2. **System Crash Dumps & Stack Traces**:
   - In `dump_diagnostic_state`, inspect `~/Library/Logs/DiagnosticReports/` for recent `.ips` and `.crash` files of `agm` / `agm-alim` and output thread backtraces.
   - Captured unified macOS system logs via `log show --predicate 'process == "agm-alim" || process == "open"' --last 2m`.
   - Added `otool -L` dynamic library inspection and `file` architecture verification.

3. **Rust Backend Hardening (`src-tauri/src/modules/logger.rs`, `delegate_updater.rs`, `lib.rs`)**:
   - Upgraded to `std::backtrace::Backtrace::force_capture()` in panic hook and updater.
   - Persisted panic logs to `$HOME/Library/Logs/AntigravityManager/panic.log`.
   - In updater, executed unconditional LaunchServices and trash purge on `open` failure, logging direct binary spawns to `updater_binary.log`.
   - In `lib.rs` startup thread, swept trash via AppleScript `/tmp` bypass and `lsregister -gc`.

4. **Native PKG & DMG UX Architecture (`scripts/package_dmg.sh`, `scripts/pkg-scripts/postinstall`, `release.yml`)**:
   - Created `scripts/pkg-scripts/postinstall` to strip quarantine, clean all user trash, and refresh LaunchServices at root level.
   - Built `Install_Antigravity_Manager.pkg` using `pkgbuild` and placed it directly inside the DMG.
   - Included clear numbered options in DMG:
     - `Install_Antigravity_Manager.pkg` (Option 1: Recommended macOS PKG installer)
     - `1-Click_Install_Or_Repair.command` (Option 2: 1-Click script installer)
     - `2-Click_Fix_Trash_Error.command` (Option 3: 1-Click repair for drag-and-drop trash errors)
     - `Antigravity Manager Tools.app` (Manual bundle)
     - `Applications` (Symlink)
     - `Install_Guide.txt` (Comprehensive installation guide)

---

## 4. Prevention & Quality Guards

1. **Unconditional LaunchServices Recovery**: Never gate LaunchServices / trash repair on specific error strings from `open` because OSStatus error codes (such as -10810) do not contain human-readable keywords in stderr.
2. **TCC Bypass Pattern**: Never rely on shell `rm -rf ~/.Trash/*` on modern macOS; always move items out of trash to `/tmp` using Finder AppleScript before deleting.
3. **Mandatory Persistent Logging**: Never discard process output to `>/dev/null` in installation or repair scripts. Always pipe to `$LOG_DIR/*.log`.
4. **Crash Report Extraction**: Always inspect `~/Library/Logs/DiagnosticReports/` when diagnosing macOS process launch failures to obtain native system thread backtraces.
5. **Forced Backtraces**: Always use `Backtrace::force_capture()` in Rust panics and recovery blocks rather than `Backtrace::capture()` so backtraces are not dropped in release builds.
