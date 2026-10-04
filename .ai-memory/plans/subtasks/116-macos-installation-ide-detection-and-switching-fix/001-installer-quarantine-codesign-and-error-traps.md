---
plan: 116-macos-installation-ide-detection-and-switching-fix
subtask: "001"
title: Overhaul install.sh on macOS with quarantine clearance, ad-hoc codesigning, dynamic bundle discovery, and error stack trace trap
domain: devops/shell
target_files:
  - install.sh
  - scripts/Fix_Damaged.command
  - scripts/package_dmg.sh
status: pending
---

# 001 — Overhaul install.sh on macOS with Quarantine Clearance, Ad-hoc Codesigning, Dynamic Bundle Discovery, and Error Stack Trace Trap

## Scope
1. Implement POSIX `ERR` trap `report_error_stack` in `install.sh` to capture command, line number, and stack trace on failure.
2. In `install_macos()`:
   - Strip quarantine from downloaded DMG (`xattr -cr "$DOWNLOAD_PATH"`).
   - Mount DMG and dynamically discover `*.app` bundles via `find "$mount_point" -maxdepth 2 -name "*.app" -type d`.
   - Safely remove existing app before copying.
   - Try copying to `/Applications/`; if permission denied, fallback to `$HOME/Applications/`.
   - Unmount DMG with `hdiutil detach "$mount_point" -force`.
   - Clear quarantine attributes on the installed bundle using user-level `xattr -cr` and `xattr -r -d com.apple.quarantine` (no `sudo` requirement).
   - If `codesign` is available, apply ad-hoc re-signing: `codesign --force --deep --sign - "$DEST_APP"`.
   - Symlink `agm` CLI binary to `$HOME/.local/bin/agm` and ensure PATH in `.zshrc` / `.bashrc`.
3. Update `scripts/Fix_Damaged.command` and `scripts/package_dmg.sh` to match current product name and dynamically locate any `.app` bundle.
