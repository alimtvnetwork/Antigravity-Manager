# Subtask 04: Test Instance Lifecycle & E2E Verification

Owner: Worker 02 / Lead
Target Files:
- `src-tauri/src/bin/agm/instance_cmds.rs`
- `src-tauri/src/modules/instance/lifecycle.rs`
- `scripts/test-instance-autoswitch-e2e.ps1`

## Requirements
1. Add CLI commands for test instance management:
   - `agm instances create-test [--name <name>] [--json]`:
     Creates transient data-only test instance named `test-<short-id>`.
     Fast creation (< 50ms, no binary duplication).
   - `agm instances rm-test [<id>] [--all] [--json]`:
     Gracefully stops test instance PID if running.
     Purges test data directory.
     Unregisters from `instances.json` and updates SQLite database.
2. Author automated PowerShell E2E test script `scripts/test-instance-autoswitch-e2e.ps1`:
   - Checks `agm instances current` to identify and protect caller instance.
   - Creates a new test instance using `agm instances create-test`.
   - Starts test instance process / mocks low quota.
   - Triggers switch verification, verifying target test PID is stopped and switched.
   - Verifies caller instance was NOT stopped or mutated.
   - Removes test instance using `agm instances rm-test`.
   - Verifies disk space is reclaimed and SQLite state is clean.

## Status: COMPLETED
- Added `agm instances create-test` and `agm instances rm-test` for rapid transient sandbox testing.
- Created and executed `scripts/test-instance-autoswitch-e2e.ps1`, confirming 5/5 verification steps pass.
