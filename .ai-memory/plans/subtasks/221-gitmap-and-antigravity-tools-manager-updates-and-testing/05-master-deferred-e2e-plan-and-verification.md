# Subtask 05: Master Deferred E2E Plan & Final Live Verification Execution
Parent: `221-gitmap-and-antigravity-tools-manager-updates-and-testing.md`
Step: 320
Improvement: `Imp-734`
Status: PENDING

## Scope & Objective
In `scripts/test-agm-gitmap-running-backup-e2e.ps1`:
- Add verification gates for reconnect sweep, multi-turn rerun diversity, and GitMap `cmdpull` file line length $\le 300$.
- Execute full verification gates strictly in Phase 3 after all code is authored:
  1. `rustfmt --check --edition 2021`
  2. `npm test`
  3. `npm run build`
  4. GitMap `go test -v ./cmdpull`
  5. Master E2E script execution.

## Invariants
- Zero builds / zero tests during authoring.
- Executed exclusively after Worker 01, Worker 02, and Worker 03 complete.
