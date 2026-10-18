---
name: instance-management-and-ui-ux-synchronization
description: Specialized skill for managing Antigravity instance lifecycle, PID termination, switch account synchronization, dark-glass UI/UX, instance audit trails, and CLI self-instance preservation.
---

# Instance Management and UI/UX Synchronization Skill

## Key Directives & Architectural Rules
1. **PID Clarification & List View Layout:**
   - Confirm that numeric status values (e.g. `1632`) represent the OS Process ID (`PID`).
   - Format list view items so that the green running indicator with `PID: <pid>`, the ending part of the `.exe` executable, and the ending part of the user data directory reside together on the exact same horizontal line.
2. **Deterministic Process Termination:**
   - On Windows, implement a multi-stage termination ladder: graceful taskkill tree termination -> direct Win32 `TerminateProcess` kernel handle kill.
   - Guard against premature error returns in `close_instance` to allow verification poll loops to complete.
3. **Pre-Switch PID Eviction:**
   - Prior to account credential updates in `state.vscdb`, unconditionally verify that any active instance PID is terminated and confirmed dead.
   - Clear stale Chromium lockfiles (`lockfile`, `code.lock`, `DevToolsActivePort`).
4. **Switch Account Modal Design System Compliance:**
   - Adhere strictly to Royal Violet design tokens (`--ui-surface-1: #171025`, `--ui-border-subtle: #40285e`, `--ui-success: #1af18d`).
   - Support whole-card row selection (`.ui-interactive-row`), `Escape` key dismissal, focus trap, and segmented action capsules.
5. **Full-Stack Instance Audit Logging:**
   - Log all instance lifecycle events (`START`, `STOP`, `RESTART`, `CREATE`, `SWITCH`) into `task_history_db` (`task_index.db` / split databases) tagged with `instance_id`.
   - Ensure the UI modal displays rich historical records in both `Switch Cycles` and `All Operations` tabs.
6. **Caller Process Self-Preservation & E2E Testing:**
   - Detect caller IDE process ID by walking ancestor process trees up to 24 levels.
   - Enforce `refuse_if_caller_instance` to ensure tests and automated switcher daemons never terminate the developer's active workspace session.
   - Efficiently create and remove transient test sandboxes via `agm instances create-test` and `agm instances rm-test`.
