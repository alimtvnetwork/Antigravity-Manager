# Subtask 06: Caller PID Detection & Automated E2E Testing Suite

**Parent Plan:** `.ai-memory/plans/instance-management-and-ui-ux-synchronization.md`  
**Status:** `IN PROGRESS`  
**Owner:** Worker 03 / Worker 01  
**Exclusive Files:**  
- `src-tauri/src/bin/agm/instance_cmds.rs`  
- `scripts/test-instance-lifecycle-e2e.ps1`  

## Objectives & Deliverables
1. Verify `agm instances current` and ensure CLI commands refuse to kill or modify caller IDE session.
2. Author PowerShell E2E test script `scripts/test-instance-lifecycle-e2e.ps1`:
   - Identify caller instance and ensure it is protected.
   - Create isolated transient test sandbox (`agm instances create-test`).
   - Launch / test PID stop and verify process terminates cleanly.
   - Verify account switch pre-switch PID eviction.
   - Verify audit trail entries exist for the test instance.
   - Cleanly remove test instance (`agm instances rm-test`), ensuring zero disk leftovers and clean database.
