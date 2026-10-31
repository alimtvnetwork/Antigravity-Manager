# Application Specification: Instance Management and UI/UX Synchronization

**Document Slug:** `instance-management-and-ui-ux-synchronization`  
**Status:** `Approved / Active Specification`  
**Target Architecture:** Antigravity-Manager Multi-Instance Subsystem, Dark-Glass UI/UX, Process Governance & Audit Trails  
**Relative Path:** `02-spec/21-app/instance-management-and-ui-ux-synchronization/spec.md`

---

## 1. Executive Summary & Problem Classification

This specification defines the architectural enhancements, bug fixes, and design system harmonization required to solve critical multi-instance lifecycle synchronization issues in Antigravity-Manager (AGM). 

### Identified Failure Modes
1. **Ambiguous Process Status Display:**
   In the instance list view, running instances display ambiguous raw numeric values (e.g., `1632`). Users are unsure whether this represents a port, sequence, or process ID. In addition, the executable end-path and the data directory suffix are stacked across multiple lines instead of residing on a unified horizontal line.
2. **Account Switching Desynchronization & Zombie Processes:**
   When switching accounts on an active instance, AGM fails to deterministically terminate the running instance process prior to configuration reload or credential injection. If the old process is alive, it holds an SQLite file lock on `state.vscdb` or flushes stale memory state on exit, wiping switched tokens.
3. **Failing "Stop Instance" Termination:**
   The "Stop Instance" action in the UI fails to reliably terminate target instances on Windows due to premature timeout errors, child process tree detachment, and absence of a kernel-level `TerminateProcess` fallback ladder.
4. **Switch Account Dialog UI/UX Fragmentation:**
   The Switch Account modal diverges visually from AGM's Royal Violet design system, using hardcoded teal palettes, unclickable rows, missing keyboard accessibility (e.g. `Escape` key), and disjointed action buttons.
5. **Empty Instance Audit Trail:**
   When clicking an instance's audit trail, the modal shows zero records because AGM's backend only logged account switches and prompt queue events, completely omitting core instance lifecycle actions (`launch`, `stop`, `restart`, `create`, `clone`, `delete`).
6. **Caller Process Self-Termination Risk in Automated Testing:**
   Automated testing and auto-switcher routines risk terminating the developer's active host IDE instance if caller PID detection is missing or bypassed during instance cleanup cycles.

---

## 2. User Request (Verbatim)

```text
# Instance Management and UI/UX Synchronization: high priority instruction, non-negotiable task

I have given you this screenshot where if we go into the list view, we should see the ending part of the exe, and that should be all together in the same line with the data ending part. And also the running status 1632, what does this mean? If it is running, then I think the green symbol is fine. We can just mention 1632. But what is this 1632? Is it the PID? If PID, please confirm. I think that is what it is. Okay. So, the first thing is the synchronization is very wrong. The switching of the accounts is not working at all somehow. So if the PID is not found, does not match, then first of all, it needs to close the PID in order to reload or change or switch account. Okay? So this is not happening at the moment. And also stop the instance should also kill the instance, which is not happening as well. Stop instance button is not working. Okay? I think you need to be careful on this. And also if I go to the switch account option, which actually shows dialogue, which actually contains the four UI UX, which is not in line with the other UI UX. You need to improve that a lot. Is it clear so far? Can you please help me with this?

Also, one more serious issue. If we click on any one of the items, instance and audit trail, we don't see any audit trail. That looks really, really terrible. I think you need to work on it. And at the end, you do the end-to-end testing for auto switcher, stopping the PID with a new instance, actually. You create your new instance. So in this machine, you can run anything, build anything, test anything. Okay? Do not break your own stuff, which instance you are using, try to find that as well. So you need to have this type of CLI commands that would detect that from the CLI that whatever the IDE, if the IDE is running the CLI command, IDE will automatically figure out what is the instance that it is running the command, and it should not break it. Remember to work like that. And always, it should actually create new instance to test and also remove the existing test instances so that it could reduce some space and make sure removing and creating the instance, it should be very efficient. Okay? And also the database should have the updated information. Remember that. Is it understood? At the end, you should do a minor bump and release with everything

## slug: instance-management-and-ui-ux-synchronization

# Actionable Items Must Follow Non-Negotiable

1. Write spec under 02-spec/21-app/instance-management-and-ui-ux-synchronization/ and enqueue plan task in .ai-memory/plans/instance-management-and-ui-ux-synchronization.md (subtasks in .ai-memory/plans/subtasks/instance-management-and-ui-ux-synchronization/) first
2. Search codebase exclusively via GitMap (gitmap aum search, gitmap find, gitmap cat, gitmap ps, gitmap py, gitmap llm train); TOTAL BAN on rg, ripgrep, grep, git grep, Select-String
3. Strictly use relative Git paths (02-spec/..., .ai-memory/..., cmd/...); only add the relative paths, never add the absolute path during your work, and ensure this is respected on the release page and in release notes as well
4. Use `gitmap` AI agents to enter data
5. Task completion includes committing and pushing to Git
6. Confirm if 1632 is the PID and ensure it is displayed correctly in the list view.
7. Fix synchronization issues with account switching and ensure PID closure when switching accounts.
8. Ensure the stop instance button kills the instance as expected.
9. Improve UI/UX consistency in the switch account dialog.
10. Implement audit trail visibility for instances.
11. Conduct end-to-end testing for auto switcher and PID management.
12. Develop CLI commands to detect and manage instances without breaking existing setups.
13. Ensure efficient creation and removal of test instances.
14. Update the database with current information.
15. Perform a minor version bump and release upon completion.
```

---

## 3. Architecture & Functional Requirements

### 3.1 PID Clarification & List View Inline Layout
- **Process ID Confirmation:** Formally confirm in code and UI that numeric status (e.g. `1632`) is the OS Process ID (`PID`).
- **Inline Single-Line Presentation:**
  - In `src/components/instances/table/InstanceRowCells.tsx` and `src/components/instances/InstanceTable.tsx`:
    - Display the green pulsing status indicator with explicit label `PID: <pid>` (e.g. `PID: 1632`) or `Idle`.
    - Display the ending segment of the `.exe` binary path (e.g. `.../Antigravity.exe`).
    - Display the ending segment of the user data directory (e.g. `.../data-default`).
    - Format all three elements inline on the same horizontal row with subtle dot dividers (`·`), eliminating cramped multi-line stacking.

### 3.2 Robust Process Termination & "Stop Instance" Hardening
- **Direct Win32 Kernel Kill (`TerminateProcess`) Fallback Ladder:**
  - In `src-tauri/src/modules/instance/close.rs`:
    - Attempt graceful tree kill via `taskkill /F /T /PID <pid>`.
    - If PID remains alive after 1000ms or `taskkill` is denied/unavailable, execute direct Win32 `OpenProcess(PROCESS_TERMINATE, 0, pid)` + `TerminateProcess(handle, 1)`.
    - On Unix, fallback from `kill -15` to `kill -9`.
- **Eliminate Premature Failure in `close_instance`:**
  - Remove premature error returns during iterative killing passes, allowing `close_instance_verified` to coordinate verification polling.
- **Frontend & IPC Synchronization:**
  - In `stop_instance`, clear cached PID and set running state to false in registry/status upon confirmed termination.

### 3.3 Account Switch Synchronization & Pre-Switch Process Eviction
- **Strict Pre-Switch Barrier:**
  - Prior to writing new credentials, modifying `state.vscdb`, or updating OS keyring:
    1. Scan for active PIDs matching `--user-data-dir` and registered `instance.pid`.
    2. Forcefully close and terminate all matching PIDs.
    3. Poll until `is_pid_alive_targeted(pid)` returns `false` across all PIDs.
    4. Remove stale Chromium lockfiles (`lockfile`, `code.lock`, `DevToolsActivePort`).
    5. Only once the process is 100% deceased, proceed with account injection and relaunch.
- **Transactional State Management:**
  - If process termination fails, abort the switch immediately *before* mutating credentials or SQLite queues, preserving atomic consistency.

### 3.4 Switch Account Modal UI/UX Modernization
- **Royal Violet Design Alignment:**
  - Refactor `src/pages/instances/SwitchAccountDialog.tsx` to utilize tokens from `ui-tokens.css`:
    - Canvas: `dark:bg-[var(--ui-surface-1)]` (`#171025`)
    - Borders: `border-[var(--ui-border-subtle)]` (`#40285e`)
    - Neon Emerald accents: `#1af18d`
- **Interactive Card Selection:**
  - Entire account cards must be clickable rows (`.ui-interactive-row`).
- **Segmented Dark-Glass Controls:**
  - Replace standalone buttons with contiguous segmented pill capsules for modal actions (`Cancel`, `Switch & Relaunch`).
- **Modal Accessibility:**
  - Add `Escape` key close listener, focus trap, explicit `X` close button, and proper `aria-modal="true"`.

### 3.5 Full-Stack Instance Audit Trail Visibility
- **Expand `task_history_db` Lifecycle Event Logging:**
  - Add audit logging for:
    - `INSTANCE_START` / `INSTANCE_LAUNCH`
    - `INSTANCE_STOP` / `INSTANCE_KILL`
    - `INSTANCE_RESTART`
    - `INSTANCE_CREATE` / `INSTANCE_CLONE`
    - `INSTANCE_DELETE`
    - `INSTANCE_SWITCH_ACCOUNT`
- **Populate Both Tabs in Modal:**
  - Ensure `InstanceAuditTrailModal.tsx` displays rich chronological history in both `Switch Cycles` and `All Operations` tabs.

### 3.6 CLI Caller PID Detection & Test Instance Sandboxing
- **Caller Detection:**
  - Expose/harden `agm instances detect-caller` / `agm instances current` to walk process ancestor trees up to 24 levels and identify the running IDE instance.
  - Enforce `refuse_if_caller_instance` across all lifecycle operations (`stop`, `switch`, `rm`, `rm-test`) to protect the host session.
- **Lightweight Test Instances:**
  - Leverage `agm instances create-test` to spin up isolated transient sandboxes without binary duplication.
  - Provide automated teardown via `agm instances rm-test` with zero orphaned files or SQLite residue.

---

## 4. Disjoint Module Ownership & Execution Plan

| Wave | Module / Subtask | Primary Owner | Owned Disjoint Files |
| :--- | :--- | :--- | :--- |
| **Wave 1** | App Spec & Subtask Plan Generation | Lead Orchestrator | `02-spec/21-app/instance-management-and-ui-ux-synchronization/*`, `.ai-memory/plans/*` |
| **Wave 2** | PID Inline Layout & Switch Account UI | Worker 01 | `src/components/instances/table/InstanceRowCells.tsx`, `src/pages/instances/SwitchAccountDialog.tsx` |
| **Wave 2** | Win32 Process Termination & Pre-Switch Guard | Worker 02 | `src-tauri/src/modules/instance/close.rs`, `src-tauri/src/modules/instance/switch.rs`, `src-tauri/src/modules/integration/system_impl.rs` |
| **Wave 3** | Instance Audit Trail Lifecycle Logging | Worker 03 | `src-tauri/src/modules/task_history_db/*`, `src-tauri/src/modules/instance/create.rs`, `src/components/instances/InstanceAuditTrail/*` |
| **Wave 4** | Caller PID Detection, CLI & E2E Verification | Worker 01 & 02 | `src-tauri/src/bin/agm/*`, `scripts/test-instance-lifecycle-e2e.ps1` |
| **Wave 5** | Minor Version Release Ceremony | Lead Orchestrator | Manifests, `changelog.md`, `changelog_en.md`, `README.md`, `README_EN.md` |

---

## 5. Verification Evidence Contract
1. **PID & Inline Layout:** Inspect rendered DOM; confirm PID badge and single-line path endings.
2. **Stop Instance & Kill:** Verified with active process; confirm PID is removed from OS process table.
3. **Account Switch:** Switched profile loads with new credentials and clean process restart.
4. **Audit Trail:** Modal renders populated history records for instance actions.
5. **Caller Protection & E2E:** `scripts/test-instance-lifecycle-e2e.ps1` runs to completion and exits with code 0 without affecting active host IDE.
