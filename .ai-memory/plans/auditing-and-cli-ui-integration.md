# Plan: auditing-and-cli-ui-integration

## Status
ACTIVE

## Spec
`02-spec/21-app/auditing-and-cli-ui-integration/01-spec.md`

## Objective
UI interaction audit recording (method + target + trigger + timestamp per
interaction), CLI commands driving any UI surface (`open`, `goto`, `ls`,
`scroll`, `click`, full help), Telegram follow-up hardening, then a minor
release with CI green.

## Subtasks
Subtask files live in `.ai-memory/plans/subtasks/auditing-and-cli-ui-integration/`:

| Code | Title | Owner | Files |
|------|-------|-------|-------|
| Task-01 | Interaction audit recorder — backend service | Worker 01 | `src-tauri/src/modules/audit/` (new) |
| Task-02 | Interaction audit recorder — frontend hooks + panel | Worker 02 | `src/components/audit/` (new), recorder wiring |
| Task-03 | CLI-UI bridge — `agm ui open/goto/ls` | Worker 01 | `src-tauri/src/bin/agm/ui_cmds.rs` (new) |
| Task-04 | CLI-UI bridge — `agm ui scroll/click` + help text | Worker 02 | `src-tauri/src/bin/agm/ui_cmds.rs` |
| Task-05 | Telegram follow-ups — `/ui` passthrough + hardening | Worker 01 | `src-tauri/src/modules/telegram_inbound/` |
| Task-06 | Release — minor bump, changelog, CI green | Lead | version files, changelog, READMEs, tag |

Ownership is disjoint per worker wave (skill R6). The lead owns shared
indexes (`.ai-memory/plans/readme.md`, `02-spec/21-app/readme.md`) and the
release (Task-06).

## Parallel workstream (outside this plan)
- Clone `alimtvnetwork/macro-ahk-v55` into `~/workspace/repos/` via
  `gitmap cfr <url>` — done by the lead directly (user instruction).
- Create a side chat named `macro-ahk-v55 repo` and read the repo there.

## Execution notes
- Spawned via `execute-parent-task-with-n-steps-v6` (A=2, H=2).
- User preamble outranks the skill: a RELEASE is explicitly requested
  (skill R10 overridden); E2E validation of the recorder/CLI is in scope
  where feasible without a display.
- Targeted `git commit <paths>` + `git push` by the lead only. NEVER
  `gitmap cpf`/`cpb` (parallel streams active).
- No workflow changes, no deletions.
- Pre-flight per AGENTS.md before tagging: `cargo fmt -- --check`,
  `cargo clippy --all-targets --all-features`, `npm run build`.
- Release flow per AGENTS.md: `npm run bump minor` → fill changelog
  skeletons (attribution strictly `(Thanks to @aukgit)`) → sync
  README.md + README_EN.md → tag `vX.Y.Z` → push tag → `gh release create`
  with `-R alimtvnetwork/Antigravity-Manager`.

## Progress
- [x] Spec written
- [x] Plan enqueued
- [ ] Subtasks dispatched
- [ ] Recorder implemented
- [ ] CLI-UI bridge implemented
- [ ] Telegram follow-ups done
- [ ] macro-ahk-v55 cloned + chat created
- [ ] Released, CI green
