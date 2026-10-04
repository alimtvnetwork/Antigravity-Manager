---
plan: 119-macos-installation-gatekeeper-quarantine-and-ide-switching
subtask: "002"
title: DMG packaging and standalone bilingual Gatekeeper repair utility
domain: devops/packaging
target_files:
  - scripts/Fix_Damaged.command
  - scripts/package_dmg.sh
status: pending
---

# 002 — DMG Packaging and Standalone Bilingual Gatekeeper Repair Utility

## Scope
1. In `scripts/Fix_Damaged.command`:
   - Provide a bilingual (Chinese & English) terminal wizard that guides users through repairing damaged application errors.
   - Dynamically discover the application bundle locally, in `/Applications`, in `$HOME/Applications`, or via fallback pattern search.
   - Clear Gatekeeper extended attributes without `sudo` first; fall back to `sudo` if permissions require it.
   - Apply local ad-hoc codesigning (`codesign --force --deep --sign -`).
   - Create CLI symlinks for `agm` and `agm-alim` into `$HOME/.local/bin/`.
2. In `scripts/package_dmg.sh`:
   - Automatically bundle `Fix_Damaged.command` directly inside the distributable DMG.
   - Ensure the repair utility has executable permissions (`chmod +x`) and stripped quarantine attributes.
