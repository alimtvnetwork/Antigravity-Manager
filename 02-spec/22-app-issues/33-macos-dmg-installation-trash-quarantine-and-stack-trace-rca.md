# App Issue RCA 33: macOS DMG Installation Failure, Trash Deduplication ("... because it is in the Trash"), Gatekeeper Quarantine, and Diagnostic Stack Trace Gaps

## 1. Symptom & Reproduction Steps

When users attempt to install, launch, or update Antigravity Manager Tools on macOS (Ventura, Sonoma, Sequoia):
1. **Gatekeeper "Damaged" Prompt**: Opening the downloaded `.app` bundle from the DMG displays:
   `"Antigravity Manager Tools" is damaged and can't be opened. You should move it to the Trash.`
2. **Move to Trash & Timestamping**: Clicking "Move to Trash" moves `/Applications/Antigravity Manager Tools.app` into `~/.Trash/`. If a trashed copy already exists, macOS appends a timestamp:
   `~/.Trash/Antigravity Manager Tools 13-20-55-339.app`
3. **Fatal LaunchServices Trash Lockout**:
   Attempting to open the application (via double-clicking the DMG icon, clicking the Dock/Spotlight shortcut, running `open -a "Antigravity Manager Tools"`, or via `install.sh` / auto-updater) fails with:
   `You can’t open the application “Antigravity Manager Tools 13-20-55-339” because it is in the Trash.`
4. **Silent Failure & Missing Diagnostics**:
   Users who encountered DMG installation failure had no stack traces or actionable error logs to determine whether `hdiutil`, `xattr`, `spctl`, `codesign`, or LaunchServices failed.

---

## 2. Root Cause Analysis (4-Part Mechanism)

### 2.1 Missing Apple Notarization & Gatekeeper Quarantine (`com.apple.quarantine`)
- **Mechanism**: The application binaries and DMG are built in GitHub Actions without an Apple Developer ID certificate or notarization ticket. When downloaded via a browser or `curl`, macOS automatically sets the extended attribute `com.apple.quarantine`. Apple Mobile File Integrity (AMFI) flags unsigned quarantined apps as corrupted/damaged and forces the user to move the app to Trash.

### 2.2 macOS Trash Collision Deduplication & Timestamp Naming
- **Mechanism**: When an app is moved to Trash while an item with that name already exists in `~/.Trash/`, macOS Finder / NSFileManager deduplicates the file by appending `HH-MM-SS-mmm` (e.g., `13-20-55-339`). This results in `Antigravity Manager Tools 13-20-55-339.app`.

### 2.3 LaunchServices Database Poisoning
- **Mechanism**: macOS LaunchServices caches bundle identifiers (e.g. `com.lbjlaq.antigravity-tools`). When an app is trashed, LaunchServices points the registered application path to `~/.Trash/Antigravity Manager Tools 13-20-55-339.app`. Any subsequent launch request directed at the application or bundle ID triggers LaunchServices to inspect the target; seeing it located inside `~/.Trash/`, macOS refuses execution and displays:
  `You can’t open the application “Antigravity Manager Tools 13-20-55-339” because it is in the Trash.`
  Even if a new copy is dragged into `/Applications`, LaunchServices remains bound to the trashed instance until the database is rebuilt (`lsregister`) or the trashed bundle is purged.

### 2.4 CI/CD DMG Packaging Disconnect & Swallowed Shell Errors
- **Mechanism**:
  1. `workflows/release.yml` used Tauri's built-in DMG packager which only bundles the `.app` and `/Applications` symlink, omitting `Fix_Damaged.command`. The custom packaging script `scripts/package_dmg.sh` was never executed in CI.
  2. Scripts (`Fix_Damaged.command`, `install.sh`) suppressed command errors with `2>/dev/null || true`, preventing bash `ERR` traps from printing call stack traces.
  3. Rust backend (`delegate_updater.rs`) used `let _ = Command::new("open").spawn()` without capturing stderr or `std::backtrace::Backtrace`.

---

## 3. Engineering Resolution

1. **Trash Purging & LaunchServices Rebuilding in `Fix_Damaged.command`**:
   - Scans `~/.Trash/` for `*antigravity*.app` or `*agm*.app`. If an active installation exists in `/Applications` or `$HOME/Applications`, permanently purges the trashed duplicate. If no active installation exists, automatically restores the app bundle from Trash.
   - Rebuilds and registers with LaunchServices via `lsregister -kill -r -domain local -domain system -domain user` and `lsregister -f "$APP_PATH"`.
   - Clears quarantine attributes recursively (`xattr -cr`, `xattr -d com.apple.quarantine`), registers with Gatekeeper (`spctl --add`), and applies local ad-hoc codesign (`codesign --force --deep --sign -`).
   - Implements POSIX `ERR` trap `report_error_stack` capturing command name, line number, exit code, and function call stack.

2. **Automated Trash Cleanup & Diagnostics in `install.sh`**:
   - Before DMG extraction, scans `~/.Trash/` and removes any stale `*antigravity*.app` or `*agm*.app` bundles to prevent LaunchServices conflicts.
   - Automatically unregisters old paths and registers the freshly installed bundle using `lsregister -f "$target_app"`.
   - Replaces silenced error pipelines with explicit diagnostic logs and backtraces upon command failures.

3. **Multi-Architecture Packaging in `scripts/package_dmg.sh`**:
   - Supports `--arch <aarch64|x64|universal>` and `--target` flags.
   - Dynamically searches target build trees (`src-tauri/target/*/release/bundle/macos/*.app`).
   - Packages `Fix_Damaged.command` (with `chmod +x` and stripped quarantine) into the root of every distributed DMG.
   - Strips quarantine from the final DMG file (`xattr -cr`).

4. **CI Release Workflow Integration (`workflows/release.yml`)**:
   - Executes `scripts/package_dmg.sh --arch "$ARCH"` in the macOS build job immediately following `npm run tauri build`.
   - Uploads the repackaged DMG containing `Fix_Damaged.command` to GitHub release artifacts.

5. **Rust Backend Launch Resilience (`delegate_updater.rs` & `process.rs`)**:
   - Dynamically locates `.app` bundles across `/Applications`, `$HOME/Applications`, and parent directory of `current_exe`.
   - Captures `output()` from `open`, verifies exit status, logs stderr, and captures `std::backtrace::Backtrace::capture()` upon launch failure.

---

## 4. Prevention & Quality Guards

1. **Mandatory Stack Trace Traps**: Every macOS shell script (`install.sh`, `package_dmg.sh`, `Fix_Damaged.command`) must enforce `set -E` and an `ERR` trap handler displaying the exact failing command, line number, and call stack.
2. **LaunchServices Hygiene**: Any installation or repair flow on macOS must proactively purge trashed app bundles and refresh `lsregister` cache.
3. **CI Gatekeeper Parity**: Release workflows must bundle self-service repair utilities (`Fix_Damaged.command`) in all distributed DMGs.
