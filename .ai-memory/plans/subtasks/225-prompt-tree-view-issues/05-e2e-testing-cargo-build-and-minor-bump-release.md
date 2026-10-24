# Subtask 05: Cargo & Frontend Compilation, Automated E2E Testing Suite, Switching Logic Revalidation & Minor Bump Release

- **Parent Plan:** [.ai-memory/plans/pending/225-prompt-tree-view-issues.md](../../pending/225-prompt-tree-view-issues.md)
- **Parent Spec:** [02-spec/21-app/225-prompt-tree-view-issues/05-e2e-cargo-build-and-release.md](../../../../02-spec/21-app/225-prompt-tree-view-issues/05-e2e-cargo-build-and-release.md)
- **Task ID:** `225`
- **Subtask ID:** `05`
- **Status:** `READY_FOR_EXECUTION`
- **Owner:** Worker 03 (Media, Button & Error Architect)
- **Date:** 2026-10-24

---

## 1. Context & Objectives

The final phase of Task 225 requires comprehensive end-to-end verification, full compiler checks, auto-switcher revalidation, and a formal minor version bump release ceremony on `main`:
1. **Compilation Pre-Flight:** Ensure zero compile errors across both Rust (`src-tauri`) and TypeScript (`src`).
2. **Automated E2E Suite Execution:** Run `.\e2e.ps1 -Suite all` with caller IDE self-preservation and sandboxed test instances.
3. **Switching Logic Revalidation:** Revalidate dynamic candidate scoring, bounce dampener, token exhaustion prediction, and active session continuity.
4. **Minor Version Bump:** Increment minor version via `npm run bump minor`, synchronizing manifests across `package.json` and `Cargo.toml`.
5. **Documentation & Attribution:** Synchronize changelogs (`changelog.md`, `changelog_en.md`) and README files (`README.md`, `README_EN.md`), strictly attributing `@aukgit` and enforcing repository-relative Git paths.

---

## 2. Targeted Files & Areas

- `package.json` & `src-tauri/Cargo.toml` (Atomic version synchronization via bump script)
- `changelog.md` & `changelog_en.md` (Dual-language release changelog with `@aukgit` attribution)
- `README.md` & `README_EN.md` (Release changelog summary synchronization under "## 📝 Changelog")
- `e2e.ps1` (Comprehensive E2E test harness execution)
- `src-tauri/src/modules/auto_switcher/` (Switching logic revalidation targets)
- `.ai-memory/plans/pending/225-prompt-tree-view-issues.md` (Final plan status update to `COMPLETED`)
- `02-spec/21-app/225-prompt-tree-view-issues/00-master-audit-ledger.md` (Master audit ledger final sign-off)

---

## 3. Step-by-Step Execution Plan

### Step 5.1: Pre-Flight Gate 1 — Rust & Frontend Compilation
- Run Rust format check:
  ```powershell
  cd src-tauri && cargo fmt -- --check
  ```
- Run Rust compiler & clippy check:
  ```powershell
  cd src-tauri && cargo clippy --all-targets --all-features
  ```
- Run frontend type check and bundle build:
  ```powershell
  npm run build
  ```
- Verify all 3 gates exit with code `0`.

### Step 5.2: End-to-End Test Suite Execution (`.\e2e.ps1`)
- Run smoke suite first to verify binary health:
  ```powershell
  .\e2e.ps1 -Suite smoke
  ```
- Run targeted prompt and switcher suites:
  ```powershell
  .\e2e.ps1 -Suite prompts
  .\e2e.ps1 -Suite switcher
  ```
- Run full suite:
  ```powershell
  .\e2e.ps1 -Suite all
  ```
- Confirm caller IDE self-preservation is active (`CallerPid` protected) and sandboxes are cleanly purged (`agm instances test-clean`).

### Step 5.3: Switching Logic Revalidation
- Validate dynamic candidate scoring: Verify depleted accounts (0 remaining tokens or 429 status) receive score `0`.
- Validate bounce dampening: Verify recently abandoned accounts are penalized against immediate bounce-backs.
- Validate conversation continuity: Verify `.antigravity_resume_task.json` preserves session integrity during account rotation.

### Step 5.4: Minor Version Bump & Release Ceremony
- Execute atomic bump script:
  ```powershell
  npm run bump minor
  ```
- Audit git changes: `git log <previous-tag>..HEAD --oneline`.
- Update `changelog.md` and `changelog_en.md`:
  - List all prompt tree view improvements, button fixes, and error management enhancements.
  - Attribution trailer: strictly `(Thanks to @aukgit)`. No other user handles.
- Update `README.md` and `README_EN.md` under "## 📝 Changelog".
- Check that all file references use strictly relative Git paths (e.g. `02-spec/...`, `src/...`).
- Run final clippy and frontend build on the release commit.
- Git commit, tag, and push:
  ```powershell
  git add -A
  git commit -m "release: vX.Y.Z - prompt tree view compactness, high-contrast buttons, and CLI error management"
  git tag vX.Y.Z
  git push origin main
  git push origin vX.Y.Z
  ```

---

## 4. Verification Checkpoints

1. Pre-flight checks pass with zero warnings and zero errors.
2. `.\e2e.ps1 -Suite all` finishes 100% green.
3. Version numbers across `package.json`, `Cargo.toml`, changelogs, and READMEs match character-for-character.
4. Git tag matches release heading exactly (`vX.Y.Z`).
5. Zero absolute paths present in commits, changelogs, or release notes.
