# Master Plan: Multi-Machine Fleet Synchronization & UI/UX Polish

## Overview
- **Slug:** `fleet-gitmap-multi-machine-sync`
- **Objective:** Fix multi-machine SSH connection, resolve empty IP address and un-interpolated `${adminUser}` in the Multi-Machine Matrix, resolve W2 machine idle status via robust 1500ms timeout with retry and GitMap fallback, provide rich cluster balance check diagnostics with error toasts, overhaul settings header and install box UI into modern segmented pill capsules, enhance CLI and AI log methods, upgrade multi-agent concurrency to `A = 3, H = 2`, and release via minor bump to `v4.221.0`.
- **Status:** In Progress
- **Concurrency Model:** `A = 3` concurrent subagents, each with `H = 2` operational hands (total 6 concurrent operations).

---

## Subtask Breakdown

### Phase 1: Planning, Concurrency & Specifications (Lead)
- [x] **Subtask-01:** Upgrade concurrency configuration to `A = 3, H = 2` across skills (`execute-parent-task-with-n-steps-v6`, `gitmap`, `execute-coding-guideline-fix`, `execute-ai-instruction-writer`, `muse-master-prompt`, AI checklist).
- [x] **Subtask-02:** Author Architecture (`01-architecture-spec.md`) and UI/UX (`02-ui-ux-spec.md`) specifications.

### Phase 2: Parallel Subagent Execution (3 Workers)
- [x] **Subtask-03 (Worker 01):** Backend SSH Manager, W2 Connectivity & CLI Diagnostics
  - Resolve `${adminUser}` and `${linuxUser}` in `connections.rs` and `config.rs`.
  - Fix `ClusterInstanceBalanceItem` field serialization to populate both `machine_ip` and `ip_address`.
  - Extend TCP probe timeout to 1500ms with retry ladder and GitMap fallback in `exec.rs`.
  - Add CLI diagnostic commands: `agm ssh test <alias>`, `agm ssh logs <alias>`, and `agm ssh check-balances`.
- [x] **Subtask-04 (Worker 02):** Backend Commands & Frontend Service Integration
  - Enhance `check_cluster_balances` in `src-tauri/src/commands/cmd_ssh.rs` to return rich diagnostic report (`ClusterBalanceCheckReport`).
  - Update `src/services/sshService.ts` to type and consume the rich diagnostic payload.
  - Expose structured error details and latency metrics.
- [x] **Subtask-05 (Worker 03):** Settings SSH & GitMap Section UI Polish
  - Redesign top-right toolbar buttons into a contiguous segmented pill capsule (`rounded-full`, dark-glass backdrop, subtle dividers).
  - Modernize `Install GitMap` section with segmented pill buttons for `Copy Install Command` and `Open PowerShell & Install GitMap`.
  - Clean up raw path text in GitMap status banner into compact metadata badges.
  - Update Multi-Machine Matrix to render resolved IPs, resolved usernames, and connection status badges with error tooltips.

### Phase 3: Pre-Flight Quality Gates & Minor Bump Release
- [ ] Pre-flight checks: `cargo fmt -- --check`, `cargo clippy --lib --all-features`, `npm run build`.
- [ ] Minor bump to `v4.221.0` and update `changelog.md`, `changelog_en.md`, `readme.md`, `readme_en.md` with strict `@aukgit` attribution.
- [ ] Atomic commit and push git tag `v4.221.0`.
