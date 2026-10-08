---
plan: 146-supabase-multi-machine-instances-and-remote-fleet-sync
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
Execute the standard release ceremony for milestone `146-supabase-multi-machine-instances-and-remote-fleet-sync`: run pre-flight code health checks (`cargo fmt`, `cargo clippy`, `npm run build`), perform a minor version bump (`npm run bump minor`), document release highlights with strict `@aukgit` attribution, synchronize README changelogs in both languages, and finalize with an atomic GitMap commit and push.

---

## 2. Release Standards & Invariants (from `AGENTS.md`)
1. **Pre-flight Gates:**
   - Must run `cd src-tauri && cargo fmt -- --check`.
   - Must run `cd src-tauri && cargo clippy --all-targets --all-features`.
   - Must run `npm run build`.
   - Local pre-flight covers fmt + clippy + frontend build only; do not run full `tauri build`.
2. **Version Bump:**
   - Run `npm run bump minor` to atomically synchronize manifests:
     * `package.json`
     * `src-tauri/tauri.conf.json`
     * `src-tauri/Cargo.toml`
     * `version.json`
     * `releases-manifest.json`
3. **Changelog Attribution Invariant:**
   - Attributions in `changelog.md` and `changelog_en.md` must attribute strictly `@aukgit` (`(Thanks to @aukgit)`).
   - **CRITICAL:** Do NOT include any other GitHub user handles (`@...`) in release changelogs or notes, ensuring the GitHub release page contributors list only contains `aukgit` and none else.
4. **README Changelog Synchronization:**
   - Stable minor releases **MUST** update the release summary in both `README.md` (under `## 📝 更新日志`) and `README_EN.md` (under `## 📝 Changelog`). Never leave README release notes outdated.
5. **Atomic Commit & Push:**
   - Final release commit must use `gitmap cpf "<module> - <summary>"`, e.g.:
     `gitmap cpf "instances - supabase multi-machine remote fleet sync & table view"`

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
Verify that all manifests reflect the bumped version number consistently across `package.json`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.toml`.

### Step 3: Changelog Authoring (`changelog.md` & `changelog_en.md`)
Add new release heading matching `vX.Y.Z` with sections:
- **Features / 新功能:**
  * Multi-machine fleet sync powered by Supabase Root DB.
  * Dedicated `<FleetMachinesTable />` in Instances view beneath local instances (in both Card and Table modes).
  * Real-time monitoring of remote worker alias, IP, running instances, bound accounts, live prompt counts, and heartbeat statuses.
  * Privacy email masking with one-click reveal toggle and monospace IP copying.
- **Attribution:**
  * Must strictly include `(Thanks to @aukgit)` and no other contributor handles.

### Step 4: README Update Synchronization
- In `README.md` (under `## 📝 更新日志`), add the Chinese summary of the new minor release.
- In `README_EN.md` (under `## 📝 Changelog`), add the English summary of the new minor release.

### Step 5: GitMap Atomic Commit & Push
```bash
gitmap cpf "instances - supabase multi-machine remote fleet sync & table view"
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
