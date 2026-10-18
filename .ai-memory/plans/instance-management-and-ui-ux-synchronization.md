# Master Execution Plan: Instance Management and UI/UX Synchronization

**Slug:** `instance-management-and-ui-ux-synchronization`  
**Parent Task ID:** `task-20261018030546-instance-management-and-ui-ux-synchronization`  
**Status:** `COMPLETED`  
**Phase:** `3 - Completed and Verified`  

---

## 1. Executive Summary & Problem Diagnosis

This master plan addresses critical defects and UI/UX inconsistencies in Antigravity-Manager's instance management subsystem:
1. **PID & Path Display in List View (Task-01)**:
   - Operating System process identifier `1632` was rendered without context (e.g. `(1632)` in card view, `PID 1632` in list view without colon).
   - In `src/components/instances/table/InstanceRowCells.tsx` (`IdeExeAndDataCell`), the executable binary name and user data directory were stacked vertically in two separate lines.
   - *Fix*: Format status explicitly as `PID: <pid>`. Place the terminal segment of the executable and data directory on the same horizontal line (`{exeEnding} · {dataEnding}`) in `IdeExeAndDataCell`.
2. **Account Switching & Stop Instance Process Lifecycle (Task-02)**:
   - `stop_instance` failed because `close_instance` had a 3-second hard-abort returning an `Err`, aborting before lockfile clearance, database `is_active = 0`, and process cache invalidation could occur.
   - On Windows, `taskkill` lacked fallback escalation to Win32 `TerminateProcess` or PowerShell `Stop-Process -Force`.
   - Account switching was not verifying process death before overwriting credentials, causing SQLite file locks on `state.vscdb`, credential rollbacks upon IDE exit, and failing to emit `instances://refreshed` events to the Tauri UI.
   - *Fix*: Remove 3s hard abort, implement multi-stage kill ladder (taskkill -> TerminateProcess / Stop-Process), unconditional lockfile + DB cleanup, and implement 6-stage deterministic switch sequence with Tauri UI event emission.
3. **Switch Account Dialog UI/UX Modernization (Task-03)**:
   - `SwitchAccountDialog.tsx` used legacy purple palette, an unstyled inline progress bar (`w-20 h-1.5`), and raw button styling.
   - *Fix*: Modernize with dark glass styling (`dark:bg-[#171025]`, `bg-[var(--ui-surface-1)]`, subtle borders), segmented pill capsules, reusable `QuotaProgressBar` integration, and Escape key dismissal.
4. **Audit Trail Visibility & Lifecycle Event Recording (Task-04)**:
   - `task_history_db` never recorded instance start/stop/restart/reload events (only scheduled prompts and switches were recorded). Standard instances had 0 records in the database.
   - *Fix*: Expand `AuditAction` enum with lifecycle action codes (`LaunchInstance`, `StopInstance`, `RestartInstance`, etc.), record events on instance operations, and ensure frontend displays both switch cycles and lifecycle operations.
5. **Active IDE Instance Protection & CLI Detection (Task-05)**:
   - CLI lacked explicit defense-in-depth against self-termination when running inside active IDE terminals (VS Code, Cursor, Antigravity).
   - *Fix*: Enhance `detect_caller_instance_id()` with `VSCODE_PID` inspection and ancestor tree scanning, enforce `guard_caller_instance` across all lifecycle CLI commands, and provide `agm instances current` output.
6. **End-to-End Lifecycle Verification & Clean Teardown (Task-06)**:
   - Author `scripts/test-instance-lifecycle-e2e.ps1` to verify test instance creation, start, stop, account switch, PID termination, and clean removal without touching active IDE instances.
7. **Minor Version Bump & Release Ceremony (Task-07)**:
   - Execute minor bump (`npm run bump minor`), update changelogs and readmes with `@aukgit` attribution, and commit/push atomically via GitMap.

---

## 2. Subtask Breakdown & Multi-Agent Concurrency Topology

Capacity = 6 (A = 3 agents × H = 2 operations).
Zero file ownership overlap across worker waves.

| Task ID | Title | Assigned Agent | Exclusive Owned Files | Verification Contract |
|---|---|---|---|---|
| **Task-01** | List View Layout & PID Clarity | Worker 01 | `src/components/instances/table/InstanceRowCells.tsx`, `src/pages/instances/InstanceCard.tsx` | Status displays `PID: <pid>`; exe & data tails on same line |
| **Task-02** | Instance Process Lifecycle & PID Control | Worker 02 | `src-tauri/src/modules/instance/close.rs`, `src-tauri/src/modules/instance/create.rs`, `src-tauri/src/modules/instance/switch.rs`, `src-tauri/src/commands/instance.rs` | Stop kills PID reliably; switch closes PID first |
| **Task-03** | Switch Account Dialog UI/UX Modernization | Worker 01 | `src/pages/instances/SwitchAccountDialog.tsx` | Modern dark glass styling; QuotaProgressBar integrated |
| **Task-04** | Audit Trail Visibility & Lifecycle Logging | Worker 03 | `src-tauri/src/modules/audit_action.rs`, `src-tauri/src/modules/task_history_db/`, `src/components/instances/InstanceAuditTrail/InstanceAuditTrailModal.tsx` | Instance start/stop/switch recorded and visible in modal |
| **Task-05** | Active IDE Instance Protection & CLI Detection | Worker 03 | `src-tauri/src/bin/agm/instance_cmds.rs`, `src-tauri/src/modules/instance/close.rs` | `agm instances current` returns active instance; self-stop refused |
| **Task-06** | E2E Testing & Lifecycle Verification | Worker 02 | `scripts/test-instance-lifecycle-e2e.ps1` | PowerShell test completes with Exit Code 0 |
| **Task-07** | Minor Version Bump & Release Ceremony | Lead | `package.json`, `src-tauri/Cargo.toml`, `CHANGELOG.md`, `changelog_en.md`, `README.md`, `README_EN.md` | `gitmap cpf "instance - synchronization and ui ux enhancements"` |

---

## 3. Disjoint Execution Wave Plan

### Wave 1 (Core Implementation - Parallel Execution across A = 3 Workers):
- **Worker 01**:
  - Implements **Task-01**: Single-line path display in `InstanceRowCells.tsx` and explicit `PID: <pid>` label in `InstanceRowCells.tsx` and `InstanceCard.tsx`.
  - Implements **Task-03**: Modernizes `SwitchAccountDialog.tsx` with dark glass styling, segmented pills, and `QuotaProgressBar`.
- **Worker 02**:
  - Implements **Task-02**: Eliminates 3s hard-abort in `close.rs`, implements Windows tree kill fallback ladder, unconditional cleanup, 6-stage switch sequence in `switch.rs`, and UI event emission in `commands/instance.rs`.
- **Worker 03**:
  - Implements **Task-04**: Expands `AuditAction` enum, instruments lifecycle logging in instance operations, and updates `InstanceAuditTrailModal.tsx`.
  - Implements **Task-05**: Enhances `detect_caller_instance_id` with `VSCODE_PID`, shields caller IDE, and updates CLI commands.

### Wave 2 (Verification, E2E Testing & Release):
- **Worker 02**:
  - Implements **Task-06**: Creates `scripts/test-instance-lifecycle-e2e.ps1` and runs the verification suite.
- **Lead Orchestrator**:
  - Runs targeted linter and pre-flight checks (`cargo fmt --check`, `cargo clippy`, `npm run build`).
  - Executes **Task-07**: Minor version bump, changelog / README synchronization, atomic GitMap commit and push.
