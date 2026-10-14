# Plan: root-cause-analysis-and-telegram-staff-improvement

## Status
ACTIVE

## Spec
`02-spec/21-app/root-cause-analysis-and-telegram-staff-improvement/01-spec.md`

## Objective
Durable RCA memory, an executable E2E testing playbook, Telegram command
improvements (`/open`, `/screenshot`, `/send`, colorful help with copy-paste),
a Supabase failure investigation, and a voice-command feasibility note.

## Subtasks
Subtask files live in `.ai-memory/plans/subtasks/root-cause-analysis-and-telegram-staff-improvement/`:

| Code | Title | Owner | Files |
|------|-------|-------|-------|
| Task-01 | RCA batch 1 — AGM incidents (fmt, E0061, pa branch conflict) | Worker 01 | `.ai-memory/rca/` (new) |
| Task-02 | RCA batch 2 — gitmap file-split incidents (imports, marker, coverage test, Windows skip) | Worker 02 | `.ai-memory/rca/` (new) |
| Task-03 | RCA batch 3 — CI workflow bugs (poisoned cache, E2E short-circuit) + rca index | Worker 01 | `.ai-memory/rca/` |
| Task-04 | E2E playbook — auto-switch + instance open flows | Worker 02 | `.ai-memory/e2e-playbook/` (new) |
| Task-05 | E2E playbook — prompt preview + prompt send + master checklist | Worker 01 | `.ai-memory/e2e-playbook/` |
| Task-06 | Telegram — `/open` and `/screenshot` commands | Worker 02 | `src-tauri/src/modules/telegram_inbound/` |
| Task-07 | Telegram — `/send` two-step flow + colorful `/help` with copy-paste | Worker 01 | `src-tauri/src/modules/telegram_inbound/`, `notification_hub/` |
| Task-08 | Supabase investigation + RCA (+ fix if in-repo) | Worker 02 | `src-tauri/src/modules/supabase_sync/`, `.ai-memory/rca/` |
| Task-09 | Voice command feasibility note (analysis only) | Worker 01 | `.ai-memory/e2e-playbook/` or spec dir |

Ownership is disjoint per worker wave (see skill R6). The lead owns shared
indexes (`.ai-memory/rca/readme.md`, `.ai-memory/e2e-playbook/readme.md`,
`.ai-memory/plans/readme.md`, `02-spec/21-app/readme.md`).

## Execution notes
- Spawned via `execute-parent-task-with-n-steps-v6` (A=2, H=2).
- User preamble outranks the skill: E2E testing IS required here (skill R1
  "zero tests" is overridden by explicit user instruction); targeted
  `git commit <paths>` + `git push` instead of `gitmap cpf/cpb` (parallel
  streams active — cpf/cpb run `git add -A` internally); no releases (R10
  holds — user did not ask for a release in this task).
- Codebase search via GitMap only. Relative paths only. No silent errors.

## Progress
- [x] Spec written
- [x] Plan enqueued
- [ ] Subtasks dispatched (wave 1)
- [ ] RCAs complete
- [ ] E2E playbook complete
- [ ] Telegram commands implemented
- [ ] Supabase RCA complete
- [ ] Voice note complete
- [ ] Committed + pushed
