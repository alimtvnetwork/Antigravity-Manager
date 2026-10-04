---
plan: 119-macos-installation-gatekeeper-quarantine-and-ide-switching
subtask: "001"
title: Installer quarantine clearing, user permission fallback, ad-hoc codesigning, and error stack trace trap
domain: devops/installer
target_files:
  - install.sh
status: pending
---

# 001 — Installer Quarantine Clearing, User Permission Fallback, Ad-Hoc Codesigning, and Error Stack Trace Trap

## Scope
1. In `install.sh`:
   - Maintain the POSIX `ERR` trap `report_error_stack "$LINENO"` capturing line number, failed command, and call stack trace upon any unexpected command exit.
   - In `install_macos`:
     - Strip quarantine attributes from the downloaded DMG file before mounting (`xattr -cr "$DOWNLOAD_PATH"` and `xattr -r -d com.apple.quarantine "$DOWNLOAD_PATH"`).
     - Dynamically search for `.app` bundle inside the mounted volume (`find "$mount_point" -maxdepth 2 -name "*.app" -type d`).
     - Check if `/Applications` is writable; if not, fall back cleanly to `$HOME/Applications/` without requiring `sudo`.
     - Clean any existing installation at target destination before copying.
     - Unmount DMG cleanly via `hdiutil detach "$mount_point" -force -quiet`.
     - Recursively strip Gatekeeper quarantine attributes from the installed target `.app` bundle.
     - Apply local ad-hoc Mach-O code-signing: `codesign --force --deep --sign - "$target_app"`.
     - Symlink CLI binaries (`agm` and `agm-alim`) into `$HOME/.local/bin/` and configure shell configuration files (`.zshrc`, `.bashrc`, `.bash_profile`).
