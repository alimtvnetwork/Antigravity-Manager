# Plan 120: macOS Installation Deep Hardening — Ventura / Sequoia

**Status:** ACTIVE  
**Spec:** `02-spec/21-app/120-macos-installation-deep-hardening-ventura-sequoia/`

---

## Summary

Fix the macOS "damaged, move to trash" regression introduced by Gatekeeper changes in
macOS 13 Ventura through macOS 15 Sequoia. Harden the install script with robust DMG
mount point parsing, multi-step quarantine clearing, version-gated signing/registration,
and full stack trace capture on failure. Add Rust backend parity for IDE discovery
(`mdfind`-first with JSON diagnostics) and `$HOME/Applications/` launch path fallback.

---

## Subtasks

| # | Task | Files | Status |
|---|---|---|---|
| Task-01 | Installer hardening | `scripts/install.sh`, `scripts/Fix_Damaged.command`, `scripts/package_dmg.sh` | PENDING |
| Task-02 | Rust backend macOS stack trace and instance path parity | `src-tauri/src/process.rs`, `src-tauri/src/instance.rs` | PENDING |
| Task-03 | Release ceremony | `CHANGELOG.md`, `changelog_en.md`, `README.md`, `README_EN.md`, version bump | PENDING |

---

## Task-01: Installer Hardening

**Scope:** `scripts/install.sh`, `scripts/Fix_Damaged.command`, `scripts/package_dmg.sh`

- Replace `grep -o '/Volumes/.*'` with `awk '/\/Volumes\//{print $NF}'` in all three files
- Add early-exit guard when mount point is empty
- Implement `quarantine_clear()` with two-phase xattr removal (bulk `xattr -cr` + recursive `find … xattr -d`)
- Detect macOS major version via `sw_vers -productVersion | awk -F'.' '{print $1}'`
- Gate `codesign --sign -` to macOS < 13 only; skip on 13+
- Run `spctl --add "$target_app"` after successful install on macOS 13+
- Replace minimal `trap ERR` with full stack trace using `caller` loop + `FUNCNAME` array

---

## Task-02: Rust Backend macOS Stack Trace and Instance Path Parity

**Scope:** `src-tauri/src/process.rs`, `src-tauri/src/instance.rs`

- `process.rs` — `discover_and_persist_initial_ide_info`:
  - Use `mdfind 'kMDItemCFBundleIdentifier == "com.lbjlaq.antigravity-tools"'` as primary IDE discovery
  - Fall back to `/Applications/`, `$HOME/Applications/`, name-with-space variants
  - Write structured JSON diagnostic log to `~/.local/share/antigravity/ide-discovery.log`
- `instance.rs` — macOS launch path check:
  - Add `$HOME/Applications/<app>` as a fallback candidate after `/Applications/<app>`
  - Iterate candidates in order; use first existing path

---

## Task-03: Release Ceremony

**Scope:** `CHANGELOG.md`, `changelog_en.md`, `README.md`, `README_EN.md`, version manifests

- Document all changes from Tasks 01–02 in both changelogs
- Synchronize release summary into README.md (under "## 📝 更新日志") and README_EN.md (under "## 📝 Changelog")
- Run `npm run bump patch` (or `minor` if scope warrants) to sync all version manifests
- Run pre-flight: `cargo fmt -- --check`, `cargo clippy --all-targets --all-features`, `npm run build`
- Tag and push to appropriate branch per release channel discipline

---

## Acceptance Criteria Reference

See `02-spec/21-app/120-macos-installation-deep-hardening-ventura-sequoia/01-architecture-spec.md` §5
for the full AC table (AC1–AC8).
