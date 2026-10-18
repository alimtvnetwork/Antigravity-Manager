# Subtask 07: E2E Testing, Quality Gates & Minor Release Ceremony

**Parent Task:** `ui-and-functionality-enhancement-instructions`  
**Subtask Code:** `Task-07`  
**Status:** `READY`

## 1. Objectives & Scope
- Execute targeted unit tests and linters (`cargo clippy --all-targets --all-features`, `cargo fmt -- --check`, `npm run build`).
- Verify prompt isolation, context menu actions, and updater URL parsing.
- Run minor version bump (`npm run bump minor`).
- Update `changelog.md` and `changelog_en.md` strictly attributing `@aukgit` (`(Thanks to @aukgit)`).
- Synchronize `README.md` and `README_EN.md` with release notes.
- Execute atomic GitMap commit and push: `gitmap cpf "ui - prompt tree enhancements, instance isolation, and updater fixes"`.

## 2. Disjoint File Ownership
- `Owned Files:` `package.json`, `src-tauri/Cargo.toml`, `changelog.md`, `changelog_en.md`, `README.md`, `README_EN.md`

## 3. Verification Evidence Contract
- Git status clean.
- All preflight checks exit 0.
- Pushed to remote tracking branch.
