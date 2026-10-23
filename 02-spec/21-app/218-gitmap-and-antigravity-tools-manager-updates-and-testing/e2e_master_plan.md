# Master Deferred E2E Plan: Sequence 218
Sequence: 218
Improvements: `Imp-715` through `Imp-719`
Slug: `218-gitmap-and-antigravity-tools-manager-updates-and-testing`

## Overview
This document catalogs the master deferred end-to-end and integration verification plan for Sequence 218. All testing follows the **Zero-Build & Zero-Test Policy during implementation**, running deferred test commands only after all code improvements are finalized.

---

## Test Specifications

### `E2E-218-01`: GitMap PA Fleet Audit & Coding Guidelines Ingestion (`Imp-715`)
- **Objective**: Verify `gitmap pa` accurately pulls and reconciles all 68 workspace repositories, detecting active diffs without data loss.
- **Verification Criteria**:
  - Confirms pull status of 68 repositories in under 30 seconds.
  - Verifies recent 50–80 git commits in GitMap and Antigravity-Manager follow atomic conventional commit format.
  - Verifies that new coding guidelines rules (installer scripts, boolean naming) are ingested into architectural documentation.

### `E2E-218-02`: Auto-Switch Prompt Restore & Quota Drop Recovery (`Imp-716`)
- **Objective**: Verify that prompts interrupted by quota exhaustion or account switching are completely swept, backed up, re-attributed, and restored without data loss.
- **Verification Criteria**:
  - Interrupted prompts in `running`, `dispatched`, `in_flight`, or `pending` states transition to `backed_up` (`is_restored = 0`).
  - Account switch re-attributes unrestored rows to the new instance profile.
  - Honest `RestoreOutcome` returned: zero fake success reports, all failed dispatches increment `failed_count`.
  - Automatic fallback to unscoped restore when scoped restore yields 0 records.

### `E2E-218-03`: Rerun Randomizer UX & Persistent Execution Engine (`Imp-717`)
- **Objective**: Verify that triggering a rerun automatically selects `'randomized'` by default with count = 4, samples uniformly from 16+ prompt variations, and respects `localStorage` persistence.
- **Verification Criteria**:
  - `RERUN_DEFAULT_MESSAGE_OPTION === 'randomized'` and `RERUN_DEFAULT_COUNT === 4`.
  - Right-click context menus reflect dynamic rerun label with randomized count.
  - Storage persistence survives browser reloads and correctly falls back to defaults.

### `E2E-218-04`: GitMap Code Quality & CG Execute Suite (`Imp-718`)
- **Objective**: Verify GitMap Go CLI codebase adheres strictly to coding guidelines and has zero build or test breakages.
- **Verification Criteria**:
  - Zero explicit `== true` or `== false` checks across Go code.
  - Standardized `apperror.AppError` and `result.Result[T]` container usage.
  - Zero pending plans remaining in `d:/work/gitmap/.ai-memory/plans/pending/`.

### `E2E-218-05`: Sequence 218 Master Regression Gate & Zero Silent Errors (`Imp-719`)
- **Objective**: Final live verification of both repositories.
- **Verification Criteria**:
  - Full frontend test suite pass (`npm test`, 9/9 suites green).
  - Production frontend build pass (`npm run build`, zero TypeScript errors).
  - GitMap CLI test suite pass (`go test ./...`).
