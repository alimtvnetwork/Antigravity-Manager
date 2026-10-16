# Plan: auto-switch-and-account-switching-enhancements

## Status
ACTIVE

## Spec
`02-spec/21-app/auto-switch-and-account-switching-enhancements/01-spec.md`

## Objective
Post-switch prompt resume (10s), durable prompt backup with CLI/UI re-send,
rerun section in the prompts tree, audit stack traces, then minor release
with CI green.

## Subtasks
Subtask files live in
`.ai-memory/plans/subtasks/auto-switch-and-account-switching-enhancements/`:

| Code | Title | Owner | Files |
|------|-------|-------|-------|
| Task-01 | RCA review — auto-switch/account-switch/fast-forward | Worker 01 (research) | none (report) |
| Task-02 | Post-switch resume (10s timer) | Worker 02 | `src-tauri/src/modules/auto_switcher.rs`, `instance.rs` |
| Task-03 | Prompt backup + CLI/UI re-send | Worker 01 | `src-tauri/src/modules/repo_db/`, `backup_prompts_db.rs`, CLI cmds |
| Task-04 | Rerun section in prompts tree | Worker 02 | `src/components/instances/PromptTreeView/` |
| Task-05 | Audit stack traces (func + file) | Worker 01 | `src-tauri/src/modules/audit/`, `src/components/audit/` |
| Task-06 | E2E validation | Worker 02 | `.ai-memory/e2e-testing/` (docs) |
| Task-07 | Release — minor bump, CI green | Lead | version files, changelog, tag |

Task-01 runs first (research, read-only). Tasks 02–06 in waves after.
Task-07 is the lead's alone. The lead owns shared indexes.

## Execution notes
- Spawned via `execute-parent-task-with-n-steps-v6` (A=2, H=2).
- User explicitly authorized the minor bump + CI green (skill R10
  overridden).
- Targeted `git commit <paths>` + `git push` by the lead only. NEVER
  `gitmap cpf`/`cpb`.
- GitMap-only search. Relative paths only. No workflow changes. No deletions.
- No silent errors. Attribution strictly `(Thanks to @aukgit)`.
- Don't regress RCA-15 remediation, RCAs 14–16, FIFO invariant.

## Progress
- [x] Spec written
- [x] Plan enqueued
- [ ] RCA review done
- [ ] Resume timer done
- [ ] Backup + re-send done
- [ ] Rerun section done
- [ ] Audit stack traces done
- [ ] E2E done
- [ ] Released, CI green
