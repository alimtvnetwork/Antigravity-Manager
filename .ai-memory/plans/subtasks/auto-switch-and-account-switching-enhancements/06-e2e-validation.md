# Task-06: E2E validation

## Status
PENDING (needs Tasks 02–05)

## Objective
Validate the fixed flows end-to-end where feasible.

## Requirements
- Read `.ai-memory/e2e-testing/readme.md`, `00-decisions.md`,
  `05-master-checklist.md` first.
- Validate: post-switch resume timing, backup/re-send round-trip, rerun
  counts, audit traces present.
- Headless limits: GUI walks can't run here — validate via CLI equivalents,
  backend checks, and code-path tracing. Document validated vs.
  needs-machine per flow.
- Record in `.ai-memory/e2e-testing/` per its conventions.

## Acceptance
- [ ] Each flow has a validation record with evidence.

## Constraints
- Docs only in this task — report code gaps back to the lead.
- GitMap search only. Relative paths only.
