# Issue 39: macOS DMG Installation Failure, Trash Deduplication ("... because it is in the Trash"), Gatekeeper Quarantine, and Diagnostic Stack Trace Gaps

## 1. Symptom & Reproduction Steps
Opening DMG or attempting to launch Antigravity Manager Tools on macOS displays:
`You can’t open the application “Antigravity Manager Tools 13-20-55-339” because it is in the Trash.`
Prior scripts failed to recover because `open` returned `-10810` (no `"Trash"` substring), skipping the recovery branch. Fallback binary launches discarded stderr to `/dev/null`, and shell `rm -rf ~/.Trash` failed due to TCC.

## 2. Root Cause
AMFI Gatekeeper quarantine prompts users to move unsigned apps to Trash, where collisions create timestamped names (`13-20-55-339.app`). LaunchServices caches the bundle ID pointing to the trash path, rejecting launches. Shell `rm -rf` cannot delete from `~/.Trash` due to macOS TCC privacy protection, and previous scripts checked `if [[ "$open_err" == *"Trash"* ]]` which failed because `open` returns `-10810`.

## 3. Resolution
1. **Unconditional Recovery**: Any `open` exit error triggers LaunchServices and trash purge recovery.
2. **TCC Bypass**: AppleScript moves matching items from `~/.Trash` to `/tmp`, where `rm -rf` deletes them cleanly without TCC permission blocks.
3. **LaunchServices Reset**: Unregisters trashed paths (`lsregister -u`), garbage collects (`lsregister -gc`), and force-registers the application (`lsregister -f -r`).
4. **Diagnostic Logging & Crash Dumps**: Direct binary fallback outputs to `$LOG_DIR/binary_launch.log`, checks `~/Library/Logs/DiagnosticReports/` for native thread backtraces, and uses `std::backtrace::Backtrace::force_capture()` in Rust.
5. **Native PKG**: Built `Install_Antigravity_Manager.pkg` via `pkgbuild` with root `postinstall` script and included it directly inside the DMG.

## 4. Prevention & Quality Guards
- Never check for specific text strings in `open` error output; handle all non-zero exit codes.
- Bypass TCC via AppleScript `/tmp` staging for trash removal.
- Never redirect binary fallback stderr to `/dev/null`.
- Use `Backtrace::force_capture()` in Rust so stack traces are preserved in release binaries.
