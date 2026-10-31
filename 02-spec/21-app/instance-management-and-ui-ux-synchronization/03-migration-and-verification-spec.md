# Instance Management & UI/UX Synchronization — Migration & Verification Specification

## 1. Overview
This specification defines the migration steps, integration protocols, testing requirements, and verification procedures for the instance management and UI/UX synchronization changes.

## 2. Verification Protocol & Acceptance Criteria

### 2.1 PID Layout Verification
- **Criteria:**
  - In list view mode, each row renders the PID (when running) or Idle state, the executable path ending, and the data directory ending on the exact same horizontal flex line.
  - Executable ending displays binary filename (e.g. `Antigravity.exe` or `antigravity-<id>.exe`).
  - Data directory ending displays user-data tail (e.g. `…\default-copy-5597\data`).
  - Copy action buttons are grouped in a segmented pill capsule.
- **Verification Method:** Visual and DOM structure inspection; TypeScript compilation passes with zero errors.

### 2.2 Process Termination Verification
- **Criteria:**
  - Invoking "Stop Instance" on a running instance reliably kills the process tree on Windows via the 5-stage ladder.
  - Process liveness check verifies `GetExitCodeProcess != STILL_ACTIVE`.
  - Chromium lockfiles (`lockfile`, `code.lock`, `DevToolsActivePort`) are purged from the instance data directory.
  - SQLite database `instance_processes` updates to `is_active = 0` and `pid = NULL`.
- **Verification Method:** PowerShell E2E test script (`scripts/test-instance-lifecycle-e2e.ps1`).

### 2.3 Account Switching Synchronization Verification
- **Criteria:**
  - Account switch on a running instance forces PID termination before credential modification.
  - Settle delay prevents file sharing violations.
  - `state.vscdb` receives new account tokens without corruption or overwrite.
  - Instance relaunches cleanly with the new account and registers the new PID.
- **Verification Method:** E2E test switches test instance account and verifies database state.

### 2.4 Switch Account Modal UI/UX Verification
- **Criteria:**
  - Modal conforms to Royal Violet design tokens (`var(--ui-surface-1)`, `var(--ui-border-subtle)`, `var(--ui-interactive)`, `var(--ui-success)`).
  - Clicking any card selects the account.
  - Pressing `Escape` closes the modal when no switch is in flight.
  - Segmented quota capsule displays 4H, Weekly, and Total percentages.
- **Verification Method:** Component linting and build validation.

### 2.5 Instance Audit Trail Verification
- **Criteria:**
  - Launch, stop, restart, create, delete, and switch operations log events into `task_history_db` with `instance_id`.
  - Opening the audit trail modal for an instance retrieves and displays all operations.
  - Dual tabs toggle smoothly; auto-tab switch activates if switch cycles are zero but operations exist.
- **Verification Method:** IPC query returns recorded events for test instance.

### 2.6 Caller Preservation & Transient Test Sandboxing
- **Criteria:**
  - `agm instances whoami` identifies caller IDE PID and returns the matching instance ID.
  - `agm instances stop <self-id>` fails with exit code 2 and `REFUSE_CALLER_INSTANCE` unless `--force` is used.
  - `agm instances create-test` creates a transient sandbox in `< 500ms` without duplicating large binaries.
  - `agm instances rm-test` terminates processes, clears lockfiles, purges data folders, and cleans database entries.
- **Verification Method:** Execution of `scripts/test-instance-lifecycle-e2e.ps1` with exit code 0.
