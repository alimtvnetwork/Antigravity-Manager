# Plan: enhance-telegram-section-and-gitmap-integration

## Status
ACTIVE

## Spec
`02-spec/21-app/enhance-telegram-section-and-gitmap-integration/01-spec.md`

## Objective
Professional Telegram prompt section: IDE exe names, sent-prompt history
with rerun, human-readable bot messages. Then minor bump + CI green.

## Subtasks
Subtask files live in
`.ai-memory/plans/subtasks/enhance-telegram-section-and-gitmap-integration/`:

| Code | Title | Owner | Files |
|------|-------|-------|-------|
| Task-01 | Message audit — catalog current bot strings | Worker 01 (research) | none (report) |
| Task-02 | IDE exe name on prompts | Worker 02 | `src-tauri/src/modules/telegram_inbound/` |
| Task-03 | Sent-prompt history + rerun buttons | Worker 01 | `src-tauri/src/modules/telegram_inbound/` |
| Task-04 | Professional message rewrite | Worker 02 | `src-tauri/src/modules/telegram_inbound/formatting.rs`, `reports_*.rs`, `send_flow.rs` |
| Task-05 | Release — minor bump, CI green | Lead | version files, changelog, tag |

Task-01 runs first (research, read-only): catalog every user-facing bot
string (destination confirmations, Id+text dumps, receipts) with file:line.
Tasks 02–04 run in waves after. Task-05 is the lead's alone.

The lead owns shared indexes (`.ai-memory/plans/readme.md`,
`02-spec/21-app/readme.md`).

## Execution notes
- Spawned via `execute-parent-task-with-n-steps-v6` (A=2, H=2).
- User explicitly authorized the minor bump + CI green (skill R10
  overridden for this task).
- Targeted `git commit <paths>` + `git push` by the lead only. NEVER
  `gitmap cpf`/`cpb`.
- GitMap-only search. Relative paths only. No workflow changes. No deletions.
- No silent errors. Attribution strictly `(Thanks to @aukgit)`.
- Don't regress RCAs 14–16.

## Progress
- [x] Spec written
- [x] Plan enqueued
- [ ] Message audit done
- [ ] IDE exe names done
- [ ] History + rerun done
- [ ] Message rewrite done
- [ ] Released, CI green
