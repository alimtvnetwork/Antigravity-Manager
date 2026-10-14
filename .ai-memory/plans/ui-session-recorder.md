# Plan: UI session recorder

- **Slug**: `ui-session-recorder`
- **Status**: `PENDING`
- **Spec**: `02-spec/21-app/ui-session-recorder/01-guideline.md`

The guideline is written. The recorder is not implemented. A later change follows the spec. This plan does not add the Start / Stop control, the invoke wrapper, or the fixture test.

## Subtasks

| # | File | Scope | Status |
| :--- | :--- | :--- | :--- |
| `01` | `.ai-memory/plans/subtasks/ui-session-recorder/01-developer-mode-flag.md` | `developerMode` default true, Start / Stop on the existing error or settings surface | `PENDING` |
| `02` | `.ai-memory/plans/subtasks/ui-session-recorder/02-click-and-invoke-sink.md` | Session sink beside `recordClickEvent`, one `invoke` wrapper, steps only while recording | `PENDING` |
| `03` | `.ai-memory/plans/subtasks/ui-session-recorder/03-production-line-strip.md` | Strip `file` and `line` when `developerMode` is false, including frames from `parseStackLine` | `PENDING` |
| `04` | `.ai-memory/plans/subtasks/ui-session-recorder/04-fixture-test.md` | Production fixture has `function` and no `line`. Developer fixture has `function`, `file`, and `line` | `PENDING` |

## Rules

- Read `02-spec/21-app/ui-session-recorder/01-guideline.md` before writing code.
- Search with `gitmap`.
- Relative paths only in tracked files.
- Do not set `RUST_BACKTRACE=1` for a production recording.
- Do not store prompt text, passwords, or API keys in a session.
