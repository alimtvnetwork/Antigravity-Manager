# Subtask 006: macOS DMG Installation, Auto-Install Script & Code Signature Hardening

## Owner: Worker 01 (DevOps/macOS Specialist)
## Target Files: `install.sh`, `scripts/Fix_Damaged.command`, `scripts/package_dmg.sh`, `workflows/release.yml`, `src-tauri/src/modules/delegate_updater.rs`, `src-tauri/src/modules/process.rs`, `src-tauri/src/modules/instance.rs`
## Status: COMPLETED

### Requirements & Resolution
1. **Automated Non-Interactive Installation**:
   - In `install.sh` (`install_macos`):
     - Strip quarantine recursively: `find "$target_app" -exec xattr -d com.apple.quarantine {} + 2>/dev/null || true`.
     - Register with Gatekeeper on macOS 13+: `spctl --add "$target_app" 2>/dev/null || true`.
     - On macOS < 15, apply ad-hoc signature: `codesign --force --deep --sign - "$target_app" 2>/dev/null || true`.
     - Purge stale and conflicting `.Trash/*antigravity*.app` and `.Trash/*agm*.app` bundles with `chflags -R nouchg` and unregister via `lsregister -u` to prevent LaunchServices poisoning.
     - Refresh LaunchServices database with `lsregister -f "$target_app"` and `lsregister -kill -r ...`.
     - Output full call stack trace and system context on failure via `report_error_stack`.
2. **DMG Packaging Hardening & Self-Repair Utility**:
   - In `scripts/package_dmg.sh`:
     - Embed executable `Fix_Damaged.command` in DMG root with quarantine stripped (`xattr -cr`).
     - Embed `Install_Guide.txt` with clear step-by-step instructions.
     - Overwrite any unpatched Tauri-generated DMG in the release bundle directory to ensure downloaded DMGs always contain the repair utility.
   - In `scripts/Fix_Damaged.command`:
     - Automatically clean up lingering trashed duplicates, clear file lock flags (`chflags -R nouchg`), unregister from LaunchServices, rebuild database, and offer automatic launch verification.
     - Add `report_error_stack` trap capturing line numbers, commands, and call stack.
3. **Rust Backend Launch Resilience**:
   - Filter any executable path residing in `.Trash` across `delegate_updater.rs`, `process.rs`, and `instance.rs`.
   - Capture `open` failure exit codes, stderr, and `std::backtrace::Backtrace` on process launch.
