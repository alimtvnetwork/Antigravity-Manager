# Master Execution Plan: Instance Management and UI/UX Synchronization

**Slug:** `instance-management-and-ui-ux-synchronization`  
**Parent Task ID:** `task-20261018041848-instance-management-and-ui-ux-synchronization`  
**Status:** `ACTIVE`  
**Phase:** `2 - Execution Step (Worker Waves)`  

---

## 1. Executive Summary & Root Cause Resolution

This master plan addresses critical defects and UI/UX inconsistencies in Antigravity-Manager's instance management subsystem:
1. **PID & Path Display in List View (Task-01 & Task-02)**:
   - Operating System process identifier `1632` is confirmed as the native OS Process ID (`PID`).
   - Format status explicitly as `Running (PID: <pid>)` with green pulsating indicator beacon.
   - Place the terminal segment of the executable and data directory on the same horizontal line (`{exeEnding} · {dataEnding}`) in `IdeExeAndDataCell`, with zero wrapping (`whitespace-nowrap flex-nowrap`).
2. **Deterministic Process Termination (Task-03)**:
   - Fix dead-code bug in `stop_instance` (`create.rs`): check `saved_pid` before wiping registry/DB state so termination liveness check functions correctly.
   - Fix empty `proc.cmd()` handling on Windows in `close.rs` so non-default PIDs aren't discarded when command line is empty.
   - Implement hardened multi-stage process tree kill ladder with Toolhelp32 child traversal and direct Win32 `TerminateProcess` kernel handles.
3. **Pre-Switch PID Eviction & Synchronization (Task-04)**:
   - In `switch.rs`, replace raw OS-wide PID check (`is_pid_alive_targeted`) with verified identity check (`saved_pid_matches`) to prevent false-positive aborts on recycled PIDs.
   - Implement deterministic 1500ms settle drain loop for OS kernel and NTFS file handle release before writing credentials to `state.vscdb`.
   - Purge stale lockfiles (`lockfile`, `code.lock`, `DevToolsActivePort`, `singleton*`).
4. **Switch Account Dialog UI/UX Modernization (Task-05)**:
   - Remove ad-hoc 4-part segmented quota capsule (`4h | Weekly | Total | [custom bar]`).
   - Integrate standardized `QuotaProgressBar` component from `src/components/accounts/QuotaProgressBar.tsx`.
   - Apply Royal Violet design tokens (`--ui-surface-1: #171025`, `--ui-border-subtle: #40285e`, `--ui-success: #1af18d`).
   - Support whole-card row selection (`.ui-interactive-row`), keyboard navigation (`Enter`/`Space`), and `Escape` key dismissal.
5. **Instance Lifecycle Audit Trail Logging & Visibility (Task-06)**:
   - Wire all lifecycle operations (`LaunchInstance`, `StopInstance`, `RestartInstance`, `CreateInstance`, `SwitchAccount`, `DeleteInstance`) into `task_history_db` split databases.
   - In `launch.rs`, call `audit.succeed()` on process spawn to prevent tasks dropping as aborted/failed.
   - In `InstanceAuditTrailModal.tsx`, support dual tabs (`Switch Cycles` and `All Operations`), and automatically route to `operations` when switches are 0.
6. **Active IDE Instance Protection & CLI Detection (Task-07)**:
   - Walk ancestor process tree up to 24 levels and inspect `VSCODE_PID`, `ANTIGRAVITY_INSTANCE_ID`, and `--user-data-dir` arguments.
   - Enforce `guard_caller_instance` and `refuse_if_caller_instance` with exit code 2 across all lifecycle CLI commands.
   - Ensure transient test sandbox commands `agm instances create-test` and `agm instances rm-test` operate efficiently with sub-50ms latency.
7. **End-to-End Lifecycle Verification & Release Ceremony (Task-08)**:
   - Execute `scripts/test-instance-lifecycle-e2e.ps1` to verify caller immunity, test instance creation, PID reporting, process termination, account switching, and clean disposal.
   - Execute minor bump (`npm run bump minor`), update changelogs and readmes with `@aukgit` attribution, and commit/push atomically via GitMap.

---

## 2. Multi-Agent Concurrency Topology & Exclusive File Boxes

Capacity = 6 (A = 3 agents × H = 2 operations).
Strictly disjoint file ownership across worker waves:

| Worker | Code | Title | Exclusive Owned Files | Verification Contract |
|---|---|---|---|---|
| **Worker 01** | `Task-02`, `Task-05` | Frontend List View, Switch Dialog & Audit Modal | `src/components/instances/table/InstanceRowCells.tsx`, `src/pages/instances/SwitchAccountDialog.tsx`, `src/components/instances/InstanceAuditTrail/InstanceAuditTrailModal.tsx` | Status displays `PID: <pid>`; exe & data tails on same line; QuotaProgressBar integrated; dual-tab audit trail |
| **Worker 02** | `Task-03`, `Task-04`, `Task-06` | Backend Process Termination, Switch Sync & Audit Logging | `src-tauri/src/modules/instance/create.rs`, `src-tauri/src/modules/instance/close.rs`, `src-tauri/src/modules/instance/switch.rs`, `src-tauri/src/modules/instance/launch.rs`, `src-tauri/src/modules/instance/lifecycle.rs` | Stop kills PID reliably; switch closes PID first; launch/restart logged to audit trail |
| **Worker 03** | `Task-07`, `Task-08` | CLI Caller Safety, E2E Test Suite & Pre-flight | `src-tauri/src/bin/agm/instance_cmds.rs`, `src-tauri/src/bin/agm/instance_routes_b.rs`, `scripts/test-instance-lifecycle-e2e.ps1` | `whoami` returns caller; self-stop refused; E2E PowerShell script exits 0 |
| **Lead Agent** | `Task-08` | Version Bump, Attribution & Atomic GitMap Commit | `package.json`, `src-tauri/Cargo.toml`, `CHANGELOG.md`, `changelog_en.md`, `README.md`, `README_EN.md` | `gitmap cpf "instances - instance management and ui ux synchronization"` |
