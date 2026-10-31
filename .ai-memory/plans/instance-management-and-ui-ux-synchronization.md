# Master Execution Plan: instance-management-and-ui-ux-synchronization

## Strategic Intent & Systems Reasoning
Eliminate process lifecycle gaps, UI path misalignment, switch account failures, and missing audit trails:
1. **PID & Path Tail Display (List View)**:
   - Confirm that `1632` is indeed the running operating system Process ID (PID).
   - Display `PID: 1632` prominently with a green running badge.
   - Format the executable path tail (`.../Antigravity.exe`) and data path tail (`.../data`) together on the exact same line in the instance list view.
2. **Account Switching & Process Termination Ladder**:
   - Before switching accounts, reloading, or changing instances, if a running or mismatched PID exists, terminate/kill the process tree first (`taskkill /F /T /PID`).
   - Fix the "Stop Instance" button to reliably kill the process and mark the instance stopped/idle.
3. **Switch Account Modal UI/UX Modernization**:
   - Redesign the switch account modal with the dark-glass design language, clean card segmentation, high-contrast badges, and accessible controls.
4. **Instance Audit Trail Visibility**:
   - Repair and display historical telemetry, switch events, and lifecycle operations in `InstanceAuditModal.tsx` from `task_history_db` / audit databases.
5. **Caller IDE Self-Preservation & CLI Parity**:
   - Implement `detect_caller_instance_id()` so CLI/backend commands protect the active developer session (`default-copy-5597` / PID 7480/8824).
   - Add `agm instances test-create` and `agm instances test-clean` for efficient sandbox creation, testing, and teardown without orphan files.
6. **Minor Version Bump & Release**:
   - Run pre-flight checks, bump minor version, sync changelogs with strict `@aukgit` attribution, sync READMEs, and push via GitMap.

---

## Subtask Decomposition

### Subtask 01: List View Single-Line Path Tails & Explicit PID Badge
- **Files Owned**: `src/components/instances/InstanceRow.tsx`, `src/components/instances/InstanceCard.tsx`
- **Deliverables**: Single-line path tail formatting and explicit `PID: <pid>` badge.

### Subtask 02: Account Switching Process Termination & Stop Instance Repair
- **Files Owned**: `src-tauri/src/modules/instance/lifecycle.rs`, `src-tauri/src/modules/instance/process.rs`, `src-tauri/src/commands/instances.rs`
- **Deliverables**: Reliable tree termination prior to account switch; functional Stop Instance button.

### Subtask 03: Switch Account Modal UI/UX Redesign
- **Files Owned**: `src/components/instances/SwitchAccountModal.tsx`
- **Deliverables**: Cohesive dark-glass styling, unified card layout, and accessible interaction states.

### Subtask 04: Instance Audit Trail Telemetry & Display
- **Files Owned**: `src-tauri/src/modules/task_history_db.rs`, `src-tauri/src/commands/audit.rs`, `src/components/instances/InstanceAuditModal.tsx`
- **Deliverables**: Complete audit trail data pipeline and rendering.

### Subtask 05: Caller IDE Auto-Detection & Transient Test Sandbox CLI
- **Files Owned**: `src-tauri/src/bin/agm/instance_cmds.rs`, `src-tauri/src/bin/agm/instance_handlers.rs`, `src-tauri/src/modules/instance/caller.rs`
- **Deliverables**: `detect_caller_instance_id()`, `test-create`, and `test-clean` CLI commands.

---

## Pre-flight & Release Gate
- `cargo fmt -- --check` [PASSED]
- `cargo clippy --all-targets --all-features` [PASSED]
- `npm run build` [PASSED]
- `npm run bump minor` -> `v4.233.0` [PASSED]
- `gitmap cpf "Release - v4.233.0 GitMap Node and Prompt System Issues with Instance Management Sync"` [READY]

## Status: COMPLETED (Verified 2026-10-19)

