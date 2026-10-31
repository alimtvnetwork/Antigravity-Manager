# Subtask 05: Master Deferred E2E Plan & Final Live Testing Execution
Status: COMPLETED
Sequence: 220
Step: 315
Improvement: `Imp-729`
Owner: Lead / Worker 01 & 02

## Objectives
- Conduct master end-to-end integration testing for Antigravity Tools Manager and GitMap following completion of all code improvements.
- Validate:
  1. Account auto-switching and prompt backup / restore on quota finish.
  2. Prompt restore sweep recovering interrupted/pending prompts into active FIFO queue.
  3. Rerun randomizer auto-pick execution.
  4. GitMap CLI commands and health probes.
- Record comprehensive execution telemetry and 100% green pass in `02-spec/21-app/220-gitmap-and-antigravity-tools-manager-updates-and-testing/03-migration-and-verification-spec.md`.

## Disjoint File Boundaries
- `scripts/test-agm-gitmap-running-backup-e2e.ps1`
- `02-spec/21-app/220-gitmap-and-antigravity-tools-manager-updates-and-testing/03-migration-and-verification-spec.md`

## Invariants
- Tests execute ONLY during this final verification phase (after all code changes are complete).
