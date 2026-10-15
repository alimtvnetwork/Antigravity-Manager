# Plan: antigravity-manager-agm-ui-and-functionality-fixes

## Status
ACTIVE

## Spec
`02-spec/21-app/antigravity-manager-agm-ui-and-functionality-fixes/01-spec.md`

## Objective
Gap-analysis against commit 26e5a622, then fix: Send/Enqueue UI + semantics,
new-conversation option, copy-with-images verification, tree import/export
(SQLite + JSON), UI polish (not the progress bar), E2E, release with CI green.

## Subtasks
Subtask files live in
`.ai-memory/plans/subtasks/antigravity-manager-agm-ui-and-functionality-fixes/`:

| Code | Title | Owner | Files |
|------|-------|-------|-------|
| Task-01 | Gap analysis — diff requirements vs 26e5a622 | Worker 01 (research) | none (report only) |
| Task-02 | Send/Enqueue UI — dropdown + polish | Worker 02 | `src/components/instances/PromptTreeView/DetailHeader.tsx` etc. |
| Task-03 | Send/Enqueue semantics — direct send, FIFO enqueue, any-project | Worker 01 | `usePromptTreePromptActions.ts`, backend dispatch |
| Task-04 | New conversation + copy-with-images verify | Worker 02 | prompt actions, `clipboard.ts` |
| Task-05 | Tree import/export — SQLite + JSON | Worker 01 | tree view, backend import/export |
| Task-06 | E2E validation of fixed flows | Worker 02 | `.ai-memory/e2e-testing/` (docs only) |
| Task-07 | Telegram buttons — send/find/credits E2E + fix | Worker 01 | `src-tauri/src/modules/telegram_inbound/` |
| Task-07 | Release — minor bump, CI green | Lead | version files, changelog, tag |

Task-01 runs first (research, read-only). Tasks 02–06 run in waves after the
gap report lands. Task-07 is the lead's alone.

The lead owns shared indexes (`.ai-memory/plans/readme.md`,
`02-spec/21-app/readme.md`).

## Execution notes
- Spawned via `execute-parent-task-with-n-steps-v6` (A=2, H=2).
- User explicitly authorized releases + minor bumps to reach CI green
  (skill R10 overridden for this task).
- Targeted `git commit <paths>` + `git push` by the lead only. NEVER
  `gitmap cpf`/`cpb`.
- GitMap-only search. Relative paths only. No workflow changes. No deletions.
- No silent errors. Attribution strictly `(Thanks to @aukgit)`.
- Pre-flight per AGENTS.md before tagging.

## Progress
- [x] Spec written
- [x] Plan enqueued
- [ ] Gap analysis done
- [ ] UI fixes done
- [ ] Semantics fixes done
- [ ] Import/export done
- [ ] E2E done
- [ ] Telegram buttons fixed
- [ ] Released, CI green
