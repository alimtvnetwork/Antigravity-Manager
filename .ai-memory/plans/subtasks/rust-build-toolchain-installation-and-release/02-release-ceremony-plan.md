---
plan: rust-build-toolchain-installation-and-release
subtask: "02"
title: Minor Version Bump Release Ceremony (v4.183.0 -> v4.184.0)
domain: release-engineering
depends_on: 01-toolchain-installation-plan.md
citations:
  app_spec: 02-spec/21-app/rust-build-toolchain-installation-and-release/02-component-spec.md
  skill: .agents/skills/agm-release-lifecycle/skill.md
  rules: AGENTS.md
target_files:
  - scripts/bump-version.mjs
  - package.json
  - package-lock.json
  - version.json
  - src-tauri/Cargo.toml
  - src-tauri/Cargo.lock
  - src-tauri/tauri.conf.json
  - Casks/antigravity-tools.rb
  - README.md
  - README_EN.md
  - src/components/layout/MiniView.tsx
  - src/pages/Settings.tsx
  - releases-manifest.json
  - CHANGELOG.md
  - CHANGELOG_EN.md
  - src-tauri/hooks.nsh
  - src/components/layout/SuggestionDeleteThinkingModal.tsx
status: pending
---

# 02 — Minor Version Bump Release Ceremony (v4.183.0 -> v4.184.0)

## 1. Context & Motivation

Antigravity-Manager is currently at version `4.183.0`. Following the completion of the Rust build toolchain installation enhancements, release management engine hardening, and multi-architecture build verification, the repository requires an atomic minor version upgrade to `v4.184.0`.

This subtask provides the complete operational blueprint for executing the release ceremony. Antigravity-Manager requires atomic synchronization across 15 distinct manifest files, strict enforcement of the `@aukgit` attribution invariant in public changelogs, full synchronization of release summaries across both `README.md` and `README_EN.md`, execution of all local pre-flight verification gates, and rigorous rollback readiness.

---

## 2. Target Files & Key Symbols

| Target File | Targeted Symbol / Section | Scope of Operation |
|---|---|---|
| `scripts/bump-version.mjs` | `TARGET_FILES`, `validateVersionUpgrade` | Orchestrates atomic mutation across all 15 manifest targets. |
| `package.json` | `"version": "4.183.0"` | Mutated to `"version": "4.184.0"`. |
| `package-lock.json` | Root `"version"` and `packages[""].version` | Mutated to `"4.184.0"`. |
| `version.json` | `"Version"`, `"version"`, `"releaseDate"` | Canonical version bumped to `"4.184.0"`; release date set. |
| `src-tauri/Cargo.toml` | `[package]` table `version` | Mutated to `version = "4.184.0"`. |
| `src-tauri/Cargo.lock` | `[[package]]` block for `agm-alim` | Mutated to `version = "4.184.0"` without duplicate lines. |
| `src-tauri/tauri.conf.json` | `"version"`, window title | Mutated to `"version": "4.184.0"` and `"Antigravity Manager Tools v4.184.0"`. |
| `Casks/antigravity-tools.rb` | `version "4.183.0"` | Mutated to `version "4.184.0"`. |
| `README.md` | Badges and `## 📝 更新日志` | Version badges updated; release highlight block added under changelog section. |
| `README_EN.md` | Badges and `## 📝 Changelog` | Version badges updated; release highlight block added under changelog section. |
| `src/components/layout/MiniView.tsx` | `setAppVersion(...)` fallback | Updated to `'4.184.0'`. |
| `src/pages/Settings.tsx` | `useState<string>(...)` fallback | Updated to `'4.184.0'`. |
| `releases-manifest.json` | `latest_version`, `latest_tag`, `releases` | Updated to `"4.184.0"`, unshifted top release entry. |
| `CHANGELOG.md` | `* **版本历史记录**:` | Inserted `v4.184.0` heading; populated with details; credited `(Thanks to @aukgit)`. |
| `CHANGELOG_EN.md` | `* **Version History**:` | Inserted `v4.184.0` heading; populated with English details; credited `(Thanks to @aukgit)`. |
| `src-tauri/hooks.nsh` | `StrCpy $0 "4.183.0"` | Mutated to `StrCpy $0 "4.184.0"`. |
| `src/components/layout/SuggestionDeleteThinkingModal.tsx` | `SUGGESTION_DELETE_THINKING_STORE` | Checked and preserved as `false` for routine release. |

---

## 3. Step-by-Step Release Ceremony Procedure

### Phase 1: Pre-Bump Working Tree & Branch Verification

1. **Verify Clean Working Tree**:
   ```bash
   git status
   ```
   Ensure no uncommitted drift or untracked temporary files exist in the repository.
2. **Verify Active Git Branch**:
   ```bash
   git rev-parse --abbrev-ref HEAD
   ```
   Confirm the branch is `main`. Production stable releases (`v4.184.0`) MUST originate from `main`.
3. **Verify Baseline Version**:
   Inspect `package.json` to confirm baseline version is `4.183.0`:
   ```bash
   node -e "console.log(JSON.parse(fs.readFileSync('package.json', 'utf8')).version)"
   ```
4. **Audit Git Commits Since Prior Release**:
   ```bash
   git log v4.183.0..HEAD --oneline
   ```
   Extract commit messages and author metadata to prepare changelog entries.

---

### Phase 2: Atomic Version Bump Execution

1. **Perform Dry-Run Inspection**:
   ```bash
   npm run bump minor -- --dry-run
   ```
   Review the printed diff and verification checks. Confirm:
   - Current version detected: `4.183.0`.
   - Computed target version: `4.184.0`.
   - Mode: Minor version upgrade (`curSem.minor + 1`, `patch: 0`).
   - Guard rails report valid progression.
2. **Execute Live Multi-Manifest Synchronization**:
   ```bash
   npm run bump minor
   ```
   This atomically updates all 15 version manifest files and executes post-bump assertions.
3. **Confirm Zero Post-Bump Mismatches**:
   Verify that `bump-version.mjs` exits with code 0 and logs:
   ```
   Every release manifest matches 4.184.0
   Cargo dependency graph verified
   All 15 version configuration locations synchronized atomically!
   ```

---

### Phase 3: Manifest Verification & Integrity Assertions

Review each manifest file against the expected schema:

1. **`package.json`**: `"version": "4.184.0"`.
2. **`package-lock.json`**: Top-level `"version": "4.184.0"` and `packages[""].version: "4.184.0"`.
3. **`version.json`**: `"Version": "4.184.0"`, `"version": "4.184.0"`, `"releaseDate": "<current date>"`.
4. **`src-tauri/Cargo.toml`**: `version = "4.184.0"` under `[package]`.
5. **`src-tauri/Cargo.lock`**: Package `agm-alim` version matches `"4.184.0"`. Verify no duplicate `version = "..."` lines.
6. **`src-tauri/tauri.conf.json`**: `"version": "4.184.0"` and window title `"Antigravity Manager Tools v4.184.0"`.
7. **`Casks/antigravity-tools.rb`**: `version "4.184.0"`.
8. **`src-tauri/hooks.nsh`**: `StrCpy $0 "4.184.0"`.
9. **`src/components/layout/MiniView.tsx`**: Fallback string `'4.184.0'`.
10. **`src/pages/Settings.tsx`**: Fallback string `'4.184.0'`.
11. **`releases-manifest.json`**: `latest_version: "4.184.0"`, `latest_tag: "v4.184.0"`, top release entry `4.184.0`.
12. **`README.md` & `README_EN.md`**: Title badges updated to `Version-v4.184.0` / `Version-4.184.0`.

---

### Phase 4: Changelog & Documentation Population

1. **Populate `CHANGELOG.md`**:
   Navigate to the newly generated `v4.184.0` block under `* **版本历史记录**:`.
   Fill in comprehensive release descriptions categorized by subsystem:
   ```markdown
       *   **v4.184.0 (2026-10-10)**:
           -   **Rust 构建工具链安装与发布引擎加固**:
               -   **说明**: 实现了 Rust 编译工具链自动化安装能力，完善了发布管理引擎 15 处版本清单原子同步协议，确立了严格的 @aukgit 归属不变量，打通了本地预检质量门禁 (cargo fmt, cargo clippy, npm run build) 并通过了全平台 CI 构建验证。(Thanks to @aukgit)
   ```
2. **Populate `CHANGELOG_EN.md`**:
   Navigate to the newly generated `v4.184.0` block under `* **Version History**:`.
   Fill in English release details:
   ```markdown
       *   **v4.184.0 (2026-10-10)**:
           -   **Rust Build Toolchain Installation & Release Engine Hardening**:
               -   **Description**: Implemented automated Rust build toolchain installation support with pre-flight verification, hardened atomic multi-manifest synchronization across all 15 files, streamlined strict @aukgit attribution invariant, enforced pre-flight quality gates (cargo fmt, cargo clippy, npm run build), and verified clean multi-platform CI release builds. (Thanks to @aukgit)
   ```
3. **Attribution Audit**:
   - Confirm every bullet item ends with `(Thanks to @aukgit)`.
   - **CRITICAL**: Search for any other GitHub username handles (`@...`). Ensure ZERO third-party handles are present.
4. **Synchronize `README.md` ("## 📝 更新日志" / "## 📝 Changelog")**:
   Prepend the new release summary blockquote at the top of the changelog section:
   ```markdown
   > Latest version **v4.184.0**: Rust build toolchain installation and release management engine hardening — implemented automated Rust build toolchain installation support with pre-flight verification, hardened 15-manifest atomic synchronization, streamlined strict @aukgit attribution invariant, and verified clean multi-platform CI gates. (Thanks to @aukgit)
   ```
5. **Synchronize `README_EN.md` ("## 📝 Changelog")**:
   Prepend the identical release summary blockquote at the top of the changelog section in `README_EN.md`.

---

### Phase 5: Thinking Store Cache Invalidation Review

Inspect `src/components/layout/SuggestionDeleteThinkingModal.tsx`:
- Confirm `SUGGESTION_DELETE_THINKING_STORE = false` for this release.
- Rationale: v4.184.0 introduces build toolchain and release management improvements without changing proxy thinking cache protobuf schemas or SQLite layouts. No user cache invalidation is required.

---

### Phase 6: Pre-Flight Quality Gates Execution

Run the three mandatory quality checks:

1. **Rust Code Formatting Gate**:
   ```bash
   cd src-tauri && cargo fmt -- --check
   ```
   *Expected Output*: Exit code 0, zero formatting diffs.
2. **Rust Clippy Compilation Gate**:
   ```bash
   cd src-tauri && cargo clippy --all-targets --all-features
   ```
   *Expected Output*: Exit code 0, zero compilation errors, zero warnings.
3. **Frontend TypeScript & Bundle Gate**:
   ```bash
   npm run build
   ```
   *Expected Output*: Exit code 0, `tsc` passes with zero errors, Vite builds `dist/` cleanly.

---

### Phase 7: Git Staging, Commit, Tagging, and Push

1. **Stage Modified Manifests & Documentation**:
   ```bash
   git add package.json package-lock.json version.json src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/tauri.conf.json Casks/antigravity-tools.rb README.md README_EN.md src/components/layout/MiniView.tsx src/pages/Settings.tsx releases-manifest.json CHANGELOG.md CHANGELOG_EN.md src-tauri/hooks.nsh
   ```
2. **Create Standardized Release Commit**:
   ```bash
   git commit -m "chore(release): bump version to 4.184.0 and update changelog"
   ```
3. **Create Annotated Release Tag**:
   ```bash
   git tag v4.184.0
   ```
4. **Push Upstream to `main` and Tag**:
   ```bash
   git push origin main
   git push origin v4.184.0
   ```

---

### Phase 8: GitHub Actions CI/CD Pipeline Monitoring

1. **Monitor CI Workflow**:
   Observe `.github/workflows/release.yml` for tag `v4.184.0`.
2. **Verify Release Gate Interception**:
   Ensure job `verify-release-target` confirms:
   - Tag name `v4.184.0` is recognized as stable release.
   - Origin branch is `main`.
   - Gate passes and releases build runners (Windows NSIS/MSI, macOS DMG x86_64/arm64, Linux deb/AppImage).
3. **Verify Asset Publication**:
   Confirm GitHub release `v4.184.0` publishes with all platform installers, generated sha256 checksums, and release notes.

---

## 4. Release Ceremony Checklists

### 4.1 Manifest Synchronization Checklist
- [ ] `package.json`: `"version": "4.184.0"`
- [ ] `package-lock.json`: `"version": "4.184.0"` in root and `packages[""]`
- [ ] `version.json`: `"Version": "4.184.0"` and `"version": "4.184.0"`
- [ ] `src-tauri/Cargo.toml`: `version = "4.184.0"`
- [ ] `src-tauri/Cargo.lock`: `agm-alim` `version = "4.184.0"`, zero duplicate lines
- [ ] `src-tauri/tauri.conf.json`: `"version": "4.184.0"`, window title updated
- [ ] `Casks/antigravity-tools.rb`: `version "4.184.0"`
- [ ] `README.md`: Badges updated to `v4.184.0`
- [ ] `README_EN.md`: Badges updated to `4.184.0`
- [ ] `src/components/layout/MiniView.tsx`: Fallback `'4.184.0'`
- [ ] `src/pages/Settings.tsx`: Fallback `'4.184.0'`
- [ ] `releases-manifest.json`: `latest_version: "4.184.0"`, top release unshifted
- [ ] `CHANGELOG.md`: Skeleton header populated with release details
- [ ] `CHANGELOG_EN.md`: Skeleton header populated with English release details
- [ ] `src-tauri/hooks.nsh`: `StrCpy $0 "4.184.0"`

### 4.2 Attribution & Documentation Checklist
- [ ] Every release item in `CHANGELOG.md` credits `(Thanks to @aukgit)`
- [ ] Every release item in `CHANGELOG_EN.md` credits `(Thanks to @aukgit)`
- [ ] Confirmed ZERO third-party `@...` handles in `CHANGELOG.md`
- [ ] Confirmed ZERO third-party `@...` handles in `CHANGELOG_EN.md`
- [ ] `README.md` changelog section contains updated `v4.184.0` summary
- [ ] `README_EN.md` changelog section contains updated `v4.184.0` summary
- [ ] `SuggestionDeleteThinkingModal.tsx`: `SUGGESTION_DELETE_THINKING_STORE = false`

### 4.3 Pre-Flight Gates Checklist
- [ ] `cd src-tauri && cargo fmt -- --check` passes with exit code 0
- [ ] `cd src-tauri && cargo clippy --all-targets --all-features` passes with exit code 0
- [ ] `npm run build` passes with exit code 0

---

## 5. Acceptance Criteria & Invariants

| Invariant / Criterion | Affirmative Boolean | Target Value | Validation Rule |
|---|---|---|---|
| Target version computed | `is_target_version_4_184_0` | `true` | `npm run bump minor` produces `4.184.0`. |
| 15 manifests updated | `has_all_15_manifests_synchronized` | `true` | `bump-version.mjs` verification loop reports 0 mismatches. |
| Attribution strictly aukgit | `is_attribution_strictly_aukgit` | `true` | Dual changelogs credit strictly `@aukgit`; 0 foreign handles. |
| Dual README synchronized | `is_readme_synchronized` | `true` | `README.md` and `README_EN.md` both contain `v4.184.0` summaries. |
| Thinking modal inactive | `is_thinking_modal_inactive` | `true` | `SUGGESTION_DELETE_THINKING_STORE` evaluates to `false`. |
| Rust formatting passing | `is_cargo_fmt_clean` | `true` | `cargo fmt -- --check` returns code 0. |
| Clippy gate passing | `is_cargo_clippy_clean` | `true` | `cargo clippy --all-targets --all-features` returns code 0. |
| Frontend build passing | `is_frontend_build_clean` | `true` | `npm run build` returns code 0. |
| Git tag matches SemVer | `is_tag_format_valid` | `true` | Tag string exactly equals `v4.184.0`. |
| Stable release on main | `is_stable_branch_main` | `true` | Target branch is `main`. |

---

## 6. Rollback Procedures

If an anomaly occurs at any stage of the ceremony, execute the appropriate rollback protocol:

### 6.1 Safe Pre-Bump Rollback Ref
Prior to running the release ceremony, create a local safety ref:
```bash
git branch backup/pre-bump-4.184.0
```

### 6.2 Scenario A: Failure During Pre-Flight Checks (Before Commit)
If `cargo fmt`, `cargo clippy`, or `npm run build` fails after `npm run bump minor`:
1. Revert all file modifications back to clean state:
   ```bash
   git checkout .
   git clean -fd
   ```
2. Verify working tree is restored to version `4.183.0`:
   ```bash
   node -e "console.log(JSON.parse(fs.readFileSync('package.json', 'utf8')).version)"
   ```
3. Fix the underlying compiler/linter error, then restart the ceremony from Phase 1.

### 6.3 Scenario B: Failure After Commit (Before Tag / Push)
If an attribution error, typo, or manifest mismatch is detected after committing:
1. Soft reset the release commit to keep working tree changes for correction:
   ```bash
   git reset --soft HEAD~1
   ```
2. Correct the affected files.
3. Re-verify pre-flight gates and re-commit.

### 6.4 Scenario C: Tag Created Locally (Before Push)
If the tag `v4.184.0` was created locally but not yet pushed:
1. Delete the local tag:
   ```bash
   git tag -d v4.184.0
   ```
2. Reset or amend the commit as needed in Scenario B.

### 6.5 Scenario D: Tag and Commit Pushed Upstream (Post-Push Emergency)
If an unexpected CI or packaging issue is detected after pushing:
1. DO NOT force-push or rewrite shared history on `main`.
2. Address the defect in a follow-up patch release (`v4.184.1`) via:
   ```bash
   npm run bump patch
   ```
3. Follow the patch release ceremony to deploy the fix cleanly to users.
