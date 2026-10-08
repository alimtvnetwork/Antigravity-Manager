---
plan: 147-smart-instance-process-cache-and-prompt-dispatch
subtask: "04"
title: Pre-flight Verification, Minor Version Bump, Changelog Sync, and Atomic Release
domain: release/ceremony
target_files:
  - package.json
  - src-tauri/tauri.conf.json
  - src-tauri/Cargo.toml
  - changelog.md
  - changelog_en.md
  - README.md
  - README_EN.md
status: pending
---

# 04 — Pre-flight Verification, Minor Version Bump, Changelog Sync, and Atomic Release

## 1. Objective

Execute the release ceremony for milestone `147-smart-instance-process-cache-and-prompt-dispatch`: run pre-flight code health checks (`cargo fmt`, `cargo clippy`, `npm run build`), perform an atomic minor version bump (`npm run bump minor` from `4.164.0` to `4.165.0`), document release highlights with strict `@aukgit` attribution, synchronize README changelogs in both English and Chinese, and finalize with an atomic GitMap commit and push.

---

## 2. Release Standards & Invariants (from `AGENTS.md`)

1. **Pre-flight Gates:**
   - Run `cd src-tauri && cargo fmt -- --check`.
   - Run `cd src-tauri && cargo clippy --all-targets --all-features` (comprehensive Rust gate, already includes compilation).
   - Run `npm run build` (TypeScript check and Vite bundle).
   - Local pre-flight covers fmt + clippy + frontend build only; do not run full `tauri build`.
2. **Atomic Version Bump:**
   - Run `npm run bump minor` to atomically increment from `4.164.0` to `4.165.0` across all project manifests:
     * `package.json`
     * `src-tauri/tauri.conf.json`
     * `src-tauri/Cargo.toml`
     * `version.json`
     * `releases-manifest.json`
3. **Changelog Attribution Invariant:**
   - Attributions in `changelog.md` and `changelog_en.md` must attribute strictly `@aukgit` (`(Thanks to @aukgit)`).
   - **CRITICAL:** Do NOT include any other GitHub user handles (`@...`) in release changelogs or notes, ensuring the GitHub release page contributors list only contains `aukgit` and none else.
4. **README Changelog Synchronization (同步首页更新日志):**
   - Stable minor releases **MUST** update the release summary in both `README.md` (under `## 📝 更新日志`) and `README_EN.md` (under `## 📝 Changelog`). Never leave README release notes outdated.
5. **Atomic Commit & Push:**
   - Final release commit must use `gitmap cpf "<module> - <summary>"`, e.g.:
     `gitmap cpf "instances - smart process cache, prompt dispatch & tag compaction v4.165.0"`

---

## 3. Step-by-Step Execution Sequence

### Step 1: Pre-flight Quality Verification
```bash
# 1. Rust formatting check
cd src-tauri && cargo fmt -- --check

# 2. Rust clippy lint gate
cargo clippy --all-targets --all-features

# 3. Frontend compilation & TypeScript checks
cd .. && npm run build
```

### Step 2: Atomic Minor Version Bump
```bash
npm run bump minor
```
Verify that all manifests reflect `4.165.0` consistently across `package.json`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.toml`.

### Step 3: Changelog Authoring (`changelog.md` & `changelog_en.md`)
Add new release heading matching `v4.165.0` with sections:
- **Features & Enhancements / 新功能与优化:**
  * **Smart Instance Process Cache & Relaunch Prevention:** Prevent unwanted IDE restarts when sending or enqueuing prompts. Cached PIDs are verified against the OS process table, ensuring prompts dispatch directly to active running instances without window disruption.
  * **Process Death Double-Check:** Graceful relaunch triggers only when a cached PID is confirmed dead in the system process table.
  * **Prompt Tree View Tag Compaction:** Compacted dual sequence badges (`P001 · #1` and `C001 · <cid>`), eliminating outer brackets and redundant text prefixes.
  * **Cleaned Tree Item Rows:** Removed redundant uppercase role tags (`USER`, `SUBAGENT`, `SYSTEM`, `TOOL`) while preserving distinct 4-tier origin visual iconography.
  * **Segmented Dark-Glass Capsules:** Standardized action toolbars and header controls into unified segmented pill capsules per `AGENTS.md`.
  * **False-Positive Running Elimination:** Deep transcript inspection and PID liveness verification to filter out false-positive running indicators on completed or idle prompts.
  * **CLI Command Parity:** Enhanced `agm prompt send`, `agm prompt queue`, `agm prompt running`, and `agm instance status` verbs.
- **Attribution:**
  * Must strictly include `(Thanks to @aukgit)` and no other contributor handles.

### Step 4: README Update Synchronization
- In `README.md` (under `## 📝 更新日志`), add the Chinese summary of the `4.165.0` release.
- In `README_EN.md` (under `## 📝 Changelog`), add the English summary of the `4.165.0` release.

### Step 5: GitMap Atomic Commit & Push
```bash
gitmap cpf "instances - smart process cache, prompt dispatch & tag compaction v4.165.0"
```

---

## 4. Deliverables Checklist
- [ ] `cargo fmt -- --check` passes cleanly with zero formatting discrepancies.
- [ ] `cargo clippy --all-targets --all-features` passes cleanly with zero warnings or errors.
- [ ] `npm run build` passes with zero TypeScript or bundling errors.
- [ ] Manifest versions bumped consistently (`package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`).
- [ ] `changelog.md` and `changelog_en.md` updated with `@aukgit` attribution only.
- [ ] `README.md` and `README_EN.md` updated with release notes summary.
- [ ] Clean working tree committed and pushed via `gitmap cpf`.
