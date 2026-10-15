# Subtask 05: Pre-Flight Verification, Minor Version Bump & Release Sync

## 1. Objective
Verify all backend Rust modules and frontend React components for the enhanced Telegram UI, CSV export, and SSH/GitMap fleet integration, perform a minor version bump (`npm run bump minor`), synchronize release changelogs and README summaries with strict `@aukgit` attribution, and run all required pre-flight gates.

## 2. Target Files (Relative Paths)
- `package.json` (Synchronized via `npm run bump minor`)
- `src-tauri/Cargo.toml` (Synchronized via `npm run bump minor`)
- `src-tauri/tauri.conf.json` (Synchronized via `npm run bump minor`)
- `CHANGELOG.md` (Updated with new minor version release notes attributed strictly to `@aukgit`)
- `CHANGELOG_EN.md` (Updated with English release notes attributed strictly to `@aukgit`)
- `README.md` (Synchronized under `## 📝 更新日志`)
- `README_EN.md` (Synchronized under `## 📝 Changelog`)

## 3. Step-by-Step Execution Plan

### 3.1 Pre-Flight Quality Gates
1. **Rust Formatting Gate**:
   - Run `cd src-tauri && cargo fmt -- --check` (or `cargo fmt` to format modified Rust files).
2. **Rust Clippy & Compilation Gate**:
   - Run `cd src-tauri && cargo clippy --all-targets --all-features` and resolve any warnings or errors in touched modules (`ssh_manager`, `telegram_inbound`, `commands/ssh.rs`, `bin/agm`).
3. **Targeted Unit Tests**:
   - Run targeted cargo tests for SSH RSA encryption, CSV formatting, and Telegram command parsing (`cd src-tauri && cargo test ssh` / `cargo test telegram`).
4. **Frontend TypeScript & Vite Production Build**:
   - Run `npm run build` and verify zero TypeScript or bundler errors.

### 3.2 Atomic Minor Version Bump & Documentation Sync
1. **Run Version Bump**:
   - Execute `npm run bump minor` to increment the minor version across `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`.
2. **Populate Changelogs & README Summaries**:
   - Document the new capabilities in `CHANGELOG.md`, `CHANGELOG_EN.md`, `README.md`, and `README_EN.md`:
     - Interactive Telegram `/help`, `/balance`, `/accounts --csv`, `/instances --csv`, `/export csv`, and `/prompt <machine>:<instance> <text>` commands and UI chips.
     - SSH & GitMap Fleet Control UI (`SshGitmapSection.tsx`) with GitMap status detection, one-click PowerShell installer, and GitMap SSH node import.
     - Immediate RSA password encryption (`save_ssh_node_rsa`) and encrypted credential bundle deployment across remote SSH nodes.
     - Multi-machine instance & account balance inspector (`get_cluster_instances_and_balances`) and remote SSH prompt dispatch (`send_remote_ssh_prompt`).
   - Attribute all entries strictly to `@aukgit` (`(Thanks to @aukgit)`), with no other GitHub handles.

## 4. Acceptance Criteria
- [ ] `npm run bump minor` updates all manifest versions cleanly.
- [ ] `CHANGELOG.md`, `CHANGELOG_EN.md`, `README.md`, and `README_EN.md` are synchronized and attribute strictly `@aukgit`.
- [ ] `cd src-tauri && cargo fmt -- --check` passes with zero formatting diffs.
- [ ] `cd src-tauri && cargo clippy --all-targets --all-features` passes cleanly.
- [ ] `npm run build` succeeds without errors.
