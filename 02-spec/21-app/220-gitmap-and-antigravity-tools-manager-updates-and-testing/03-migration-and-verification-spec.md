# Specification 220: Migration & Verification Spec — GitMap and Antigravity Tools Manager Updates and Testing
Sequence: 220
Batch: Steps 311–315
Improvements: `Imp-725` through `Imp-729`

## 1. Migration Protocol
- No breaking database schema alterations required; SQLite migrations are additive or use existing columns in `prompt_backups` and `active_prompts`.
- Any existing interrupted prompts in `prompt_backups` are retroactively discovered by the sweep logic upon the next account rotation.

## 2. Verification Plan & Test Matrix
Verification executes strictly during Phase 3 (deferred verification):

| Test ID | Target Component | Verification Command / Harness | Success Criteria |
| :--- | :--- | :--- | :--- |
| `TEST-220-01` | Restore Honesty Suite | `cargo test test_incomplete_prompt_sweep_selection_and_status_transition` | Interrupted prompts correctly re-attributed and restored |
| `TEST-220-02` | Quota Finish Recovery | `cargo test test_restore_interrupted_prompts_on_quota_exhaustion` | 100% of interrupted turns re-enqueued as `queued` |
| `TEST-220-03` | Rerun Randomizer UX | Frontend test suite (`rerunActions.test.ts`) | Auto-pick variation populated on mount |
| `TEST-220-04` | GitMap Code Quality | `gitmap check` or Go lint suite | Clean static analysis, zero coding guideline regressions |
| `TEST-220-05` | Master Deferred E2E | `scripts/test-agm-gitmap-running-backup-e2e.ps1` | All 25 assertions pass, zero orphan remnants |

## 3. Evidence Collection
All test outputs, telemetry records, and pass/fail receipts will be captured here upon final execution.
