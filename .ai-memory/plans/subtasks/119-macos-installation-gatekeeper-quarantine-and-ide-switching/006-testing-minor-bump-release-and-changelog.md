---
plan: 119-macos-installation-gatekeeper-quarantine-and-ide-switching
subtask: "006"
title: Verification, pre-flight validation, minor version bump, and changelog synchronization
domain: devops/release
target_files:
  - package.json
  - src-tauri/Cargo.toml
  - CHANGELOG.md
  - CHANGELOG_EN.md
  - README.md
  - README_EN.md
status: pending
---

# 006 — Verification, Pre-flight Validation, Minor Version Bump, and Changelog Synchronization

## Scope
1. Run pre-flight checks:
   - `cd src-tauri && cargo fmt -- --check`
   - `npm run build`
2. Run targeted Rust unit tests for `format_macos_open_args`, `get_macos_candidate_paths`, and `discover_and_persist_initial_ide_info`.
3. Verify `install.sh` syntax and dry run execution.
4. Execute minor version bump (`4.142.0` -> `4.143.0`).
5. Update `CHANGELOG.md` and `CHANGELOG_EN.md` attributing strictly `@aukgit` `(Thanks to @aukgit)`.
6. Synchronize release summary in root `README.md` ("## 📝 更新日志") and `README_EN.md` ("## 📝 Changelog").
7. Ensure zero absolute paths anywhere in the commit or changelogs.
8. Stage and commit atomically using GitMap.
