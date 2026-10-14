# Task-04: E2E playbook — auto-switch + instance open

## Status
PENDING

## Objective
Write `.ai-memory/e2e-playbook/01-auto-switch.md` and
`.ai-memory/e2e-playbook/02-instance-open.md` (the lead owns `readme.md`).

## 01-auto-switch.md must contain
- Purpose: verify AGM automatically switches to the correct instance/account
  when quota or focus conditions trigger.
- Preconditions: AGM version under test, ≥2 configured instances on the
  test machine, known quota states.
- Numbered steps with exact UI targets (tray/window/panel names as they
  appear), expected result per step, and what to capture as evidence.
- Failure branches: for each step, what the AI should check/log when the
  expected result does not occur (no guessing — exact commands/paths).
- Final verification: how to confirm the switch happened (UI state + logs).

## 02-instance-open.md must contain
- Purpose: verify opening an instance from AGM launches/focuses the correct
  IDE window.
- Preconditions: instance configured, IDE installed.
- Numbered steps: select instance → open action → expected OS window/process
  state → how to confirm the right project/workspace opened.
- Failure branches and evidence capture.

## Style rules
- Written for an AI with no prior context ("dummy AI" proof): every step
  names the exact button/label/command; no "you know where this is".
- Numbered checklists with `[ ]` boxes the tester ticks off.
- Each flow ends with a PASS/FAIL verdict rule.

## Constraints
- GitMap search only for codebase references. Relative paths only.
- No live testing required in this subtask — document the procedure.
