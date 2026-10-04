# Plan 120: macOS Installation Deep Hardening — Ventura / Sequoia

**Status:** COMPLETED  
**Version:** v4.144.0  
**Spec:** `02-spec/21-app/120-macos-installation-deep-hardening-ventura-sequoia/`

---

## Summary

Fixed the macOS "damaged, move to trash" regression introduced by Gatekeeper changes in
macOS 13 Ventura through macOS 15 Sequoia. Hardened the install script with robust DMG
mount point parsing via `awk`, multi-step recursive quarantine clearing, version-gated
signing and `spctl --add` registration, and full stack trace capture on failure. Added
Rust backend parity for IDE discovery (`mdfind`-first with persistent JSON backtrace diagnostics)
and `$HOME/Applications/` candidate paths and launch fallback.

---

## Subtasks

| # | Task | Files | Status | Evidence |
|---|---|---|---|---|
| Task-01 | Installer hardening | `install.sh`, `scripts/Fix_Damaged.command`, `scripts/package_dmg.sh` | COMPLETED | awk mount, recursive xattr -d, spctl --add, sw_vers detection, detect_ide_path |
| Task-02 | Rust backend macOS stack trace and instance path parity | `src-tauri/src/modules/process.rs`, `src-tauri/src/modules/instance.rs` | COMPLETED | user Applications paths, ide-discovery.log JSON diagnostics with backtrace |
| Task-03 | Release ceremony | `CHANGELOG.md`, `CHANGELOG_EN.md`, `README.md`, `README_EN.md`, version bump | COMPLETED | v4.144.0 synchronized and verified |

---

## Acceptance Criteria Verified

- **AC1 (macOS 13 Ventura)**: `spctl --add` and recursive quarantine removal eliminate damaged prompt.
- **AC2 (macOS 14 Sonoma)**: Multi-step `xattr -d com.apple.quarantine` recurses into nested frameworks.
- **AC3 (macOS 15 Sequoia)**: Skips rejected ad-hoc signatures on macOS 15+; registers with Gatekeeper directly.
- **AC4 (CLI Symlink)**: `agm` and `agm-alim` symlinked to `~/.local/bin/`.
- **AC5 (IDE Detection)**: `detect_ide_path` runs and outputs detected Antigravity path during install.
- **AC6 (IDE Switching)**: Candidate discovery and launcher support `$HOME/Applications/Antigravity.app`.
- **AC7 (Stack Trace)**: `report_error_stack` captures commands and call stack; `ide-discovery.log` persists backtraces.
- **AC8 (Fallback)**: Non-writable `/Applications/` gracefully falls back to `$HOME/Applications/`.
