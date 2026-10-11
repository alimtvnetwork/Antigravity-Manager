# Master Plan: Auto Switch, Rotation, Fast-Forward, Version Downgrade & E7001 Resilience

- **Slug**: `auto-switch-and-rotate-fix`
- **Date**: 2026-10-11
- **Lead Architect**: Md. Alim Ul Karim (@aukgit)
- **Status**: COMPLETED
- **Associated Specs**:
  - `02-spec/21-app/auto-switch-and-rotate-fix/01-architecture-spec.md`
  - `02-spec/21-app/auto-switch-and-rotate-fix/02-component-spec.md`
  - `02-spec/21-app/auto-switch-and-rotate-fix/03-ai-instructions.md`

---

## 1. Executive Summary & Root Cause Architecture

Users reported that Account Switching, Auto-Rotate, and Fast-Forward (restarting instance) ceased working compared to baseline `v4.162.0`. In addition, attempting to downgrade/revert the CLI via `agm update to v4.162` was rejected as `[UP TO DATE]`, and launching cloned instances failed with `E7001: Failed to spawn instance process: Access is denied. (os error 5)`.

### Root Cause Breakthroughs:
1. **Auto-Switcher Candidate Scoring Lockup**:
   - `score_candidate_account` in `src-tauri/src/modules/auto_switcher/scoring.rs` clamped all sub-100% quota accounts to score `0.0`.
   - In `src-tauri/src/modules/auto_switcher/selection.rs`, Step 3 fallback was improperly gated by `cooldown_pool.is_empty()`, and called `score_candidate_account` instead of `score_candidate_account_fallback`, resulting in `Ok(None)` and zero rotations when no 100% account existed.
2. **Cross-Instance PID Contamination**:
   - In `src-tauri/src/modules/instance/pid_scan.rs:302`, `instance_root_pids.contains(&p)` checked global instance roots rather than the target instance's `matched_set`.
   - Child processes of active instances were falsely attached to stopped instances, freezing Fast-Forward and profile resets.
3. **CLI Update Downgrade / Revert Rejection**:
   - In `src-tauri/src/bin/agm/update_cmds.rs`, prepositional tokens (`to`, `revert`, `downgrade`) were stripped, and only `releases/latest` was queried.
   - Older versions were short-circuited as `[UP TO DATE]`, and `install.ps1` rejected lower versions unless `-isPinned` or `-Force` was respected.
4. **Win32 Job Object Breakaway Restriction (E7001)**:
   - In `src-tauri/src/modules/process/launch.rs`, `CREATE_BREAKAWAY_FROM_JOB` (`0x01000000`) caused Win32 `CreateProcessW` to fail with `os error 5` inside restricted Windows Job Objects.
   - In `src-tauri/src/modules/instance/executable.rs`, fallback batch files (`launch-<id>.cmd`) were passed directly to `Command::new`, which cannot execute shell scripts without `cmd.exe /c`.

---

## 2. Work Breakdown & Subtasks Allocation

| Subtask ID | Title | Target Files | Status |
| :--- | :--- | :--- | :---: |
| **Subtask-01** | CI/CD Compiler & Type Fixes (`running.rs`, `selection.rs`, `launch.rs`) | `src-tauri/src/modules/process/running.rs`, `src-tauri/src/modules/auto_switcher/selection.rs`, `src-tauri/src/modules/instance/launch.rs` | `[DONE]` |
| **Subtask-02** | Cross-Instance PID Contamination & Instance Isolation | `src-tauri/src/modules/instance/pid_scan.rs` | `[DONE]` |
| **Subtask-03** | Auto-Switcher Scoring Fallback & Quota Gate Resilience | `src-tauri/src/modules/auto_switcher/selection.rs`, `src-tauri/src/modules/auto_switcher/scoring.rs` | `[DONE]` |
| **Subtask-04** | AGM Update Version Downgrade, Revert Parsing & Installer Support | `src-tauri/src/bin/agm/update_cmds.rs`, `src-tauri/src/modules/delegate_updater/`, `install.ps1` | `[DONE]` |
| **Subtask-05** | Win32 Job Breakaway Fallback, PE Binary Cloning & E7001 Elimination | `src-tauri/src/modules/process/launch.rs`, `src-tauri/src/modules/instance/launch.rs`, `src-tauri/src/modules/instance/executable.rs` | `[DONE]` |

---

## 3. Strict Rules & Quality Invariants

- **Search Tooling**: Exclusively GitMap (`gitmap aum search`, `gitmap find`, `gitmap cat`, `gitmap ps`). Total ban on raw search tools (`rg`, `ripgrep`, `grep`, `Select-String`).
- **Path Hygiene**: Strictly relative Git paths across all specs, plans, release notes, and documentation.
- **Commit Format**: GitMap atomic commit (`gitmap cpb "<module> - <summary>"` with hyphen).
- **CI/CD Quality Gate**: 100% green pass verified across `ubuntu-latest`, `windows-2022`, and `macos-latest` via `gitmap pe -t`.
