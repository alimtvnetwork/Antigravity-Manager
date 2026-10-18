# Subtask 07: End-to-End Testing & Verification

- **Slug:** `instance-management-and-ui-ux-synchronization`
- **Subtask ID:** `07-e2e-testing-and-verification`
- **Parent Plan:** `.ai-memory/plans/instance-management-and-ui-ux-synchronization.md`
- **Status:** `PENDING`

## Objective
Author and execute an automated PowerShell end-to-end integration test (`scripts/test-instance-management-e2e.ps1`) verifying auto-switcher, PID termination, test instance provisioning, and audit trail generation without disturbing the active IDE instance.

## Scope of Changes
- Author `scripts/test-instance-management-e2e.ps1`.
- Verify:
  1. Creation of an isolated test instance.
  2. Launch and PID registration.
  3. Stopping the instance and confirming process termination (PID is dead).
  4. Switching account on a running instance: verify old process is killed before new state is applied.
  5. Audit trail records all lifecycle events into split SQLite database.
  6. Clean removal of test instance and database cleanup.
  7. Verification of self-instance protection guard.

## Verification
- Test runs to completion with exit code 0.
- All temporary directories purged.
