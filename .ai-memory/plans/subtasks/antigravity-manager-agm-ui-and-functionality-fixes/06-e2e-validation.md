# Task-06: E2E validation

## Status
PENDING (needs Tasks 02–05)

## Objective
Validate the fixed flows end-to-end where feasible.

## Requirements
- Read `.ai-memory/e2e-testing/readme.md`, `00-decisions.md`,
  `05-master-checklist.md` first (per what-to-read).
- Validate: Send on inactive project, Enqueue FIFO order, new conversation,
  copy-with-images, tree export/import round-trips.
- This machine is headless — GUI walks can't run here. Validate via:
  CLI equivalents (`agm prompt`, `agm ui ...`), backend unit checks where
  they exist, and code-path tracing. Document exactly what was validated
  vs. what needs the user's machine.
- Record results in `.ai-memory/e2e-testing/` per its conventions.

## Acceptance
- [ ] Each fixed flow has a validation record (validated or needs-machine).
- [ ] No flow marked "done" without evidence.

## Constraints
- GitMap search only. Relative paths only. Docs only — no code changes
  in this task (report gaps back to the lead).
