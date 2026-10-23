# Subtask 05: AGM & GitMap E2E Master Verification Plan
Sequence: 216
Step: 295
Status: COMPLETED
Module: `02-spec/21-app/216-gitmap-and-antigravity-tools-manager-updates-and-testing/e2e_master_plan.md`

## Description
Author and execute master deferred end-to-end verification plan covering account auto-switching, prompt backup and restore continuity on quota exhaustion, rerun auto-pick, GitMap command responsiveness, and guideline compliance.

## Implementation Details
1. Define test case `E2E-216-01`: Account auto-switch prompt restore on quota exhaustion.
2. Define test case `E2E-216-02`: Incomplete-prompt sweep beyond 120s window.
3. Define test case `E2E-216-03`: Rerun randomized auto-pick by default and persistence.
4. Define test case `E2E-216-04`: GitMap command suite execution (`gitmap status`, `gitmap find`, `gitmap cat`).
5. Define test case `E2E-216-05`: Cross-repository coding guidelines verification.
6. Execute deferred verification upon finalization of all code improvements.

## Done When
- Master E2E plan documented and executed with 100% pass criteria.
