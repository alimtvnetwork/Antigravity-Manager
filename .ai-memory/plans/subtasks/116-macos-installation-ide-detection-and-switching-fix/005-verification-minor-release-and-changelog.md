---
plan: 116-macos-installation-ide-detection-and-switching-fix
subtask: "005"
title: Verification, pre-flight checks, minor release bump, and changelog synchronization
domain: devops/release
target_files:
  - package.json
  - src-tauri/Cargo.toml
  - CHANGELOG.md
  - CHANGELOG_EN.md
  - README.md
  - README_EN.md
status: completed
---

# 005 — Verification, Pre-flight Checks, Minor Release Bump, and Changelog Synchronization

## Scope
1. Run pre-flight checks:
   - `cd src-tauri && cargo fmt -- --check`
   - `cd src-tauri && cargo clippy --all-targets --all-features`
   - `npm run build`
2. Run targeted unit tests for touched Rust modules.
3. Validate `install.sh` syntax and dry run.
4. Run minor version bump (`4.140.0` -> `4.141.0`).
5. Update `CHANGELOG.md` and `CHANGELOG_EN.md` attributing `@aukgit` `(Thanks to @aukgit)`.
6. Synchronize release notes in root `README.md` and `README_EN.md`.
7. Ensure zero absolute paths anywhere in the commit or changelogs.
8. Stage and commit atomically using GitMap.
