# Task-03: RCA batch 3 — CI workflow bugs + RCA index

## Status
DONE

## Objective
Document the two gitmap CI workflow bugs as RCAs, and write the
`.ai-memory/rca/readme.md` index covering all 10 RCAs.

## Incidents to document

### 08-gitmap-workflow-poisoned-cache.md
- Symptom: re-running CI on a failed SHA short-circuits everything via the
  `ci-passed-<sha>` cache, even though the SHA never passed.
- Root cause: the `test-summary` job runs `if: always()` and its
  `Mark SHA as built` step uses `if: success()` — which checks the JOB's
  steps, not the pipeline. So a failed pipeline still writes the "passed"
  cache entry (`.github/actions/mark-sha-built/action.yml`).
- How found: `gh run view --job <sha-check> --log` showed
  "Cache hit for: ci-passed-<sha>"; traced the writer via repo search for
  `ci-passed`.
- Workaround used: `gh cache delete "ci-passed-<sha>"` to force a fresh run.
- Recommended fix (NOT implemented — standing no-workflow-changes rule):
  gate the mark step on all needed jobs succeeding, not just the job's own
  steps.
- Prevention: never trust a green short-circuit; verify `already-built`
  semantics before re-running.

### 09-gitmap-workflow-e2e-shortcircuit.md
- Symptom: E2E jobs fail on re-run at step "Already validated":
  "An error occurred trying to start process '/bin/bash' with working
  directory '.../cli'. No such file or directory".
- Root cause: the E2E job sets `defaults.run.working-directory: cli`, but
  when `already-built == 'true'` the checkout step is skipped, so `cli/`
  does not exist and bash cannot start.
- How found: CI log; read the E2E job definition in `.github/workflows/ci.yml`.
- Recommended fix (NOT implemented — standing no-workflow-changes rule):
  override `working-directory` on the "Already validated" step or drop the
  job-level default in favor of per-step directories.
- Prevention: any short-circuit/skip path must be tested, not just the happy
  path.

### 10-rca-index (lead-owned file, but draft the content here)
Draft the index table for `.ai-memory/rca/readme.md` listing all 10 RCAs
(01–09 above + 10 = this workflow-bug pair summary) with one-line causes
and links. The lead will write the final index file.

## Format per RCA
Symptom → Root cause → How found → Workaround/fix → Recommended fix (if
blocked by constraints) → Prevention checklist item.

## Constraints
- GitMap search only. Relative paths only. Do NOT modify
  `.github/workflows/` (standing rule) — document only.
