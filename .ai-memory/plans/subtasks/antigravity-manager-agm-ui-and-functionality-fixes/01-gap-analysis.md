# Task-01: Gap analysis (research, read-only)

## Status
PENDING

## Objective
Diff the 7 requirement groups in the spec against the current implementation
(especially commit 26e5a622) and produce a build-only-the-gaps report.

## Method
- Read `02-spec/21-app/antigravity-manager-agm-ui-and-functionality-fixes/01-spec.md`
  requirements 3.1–3.6.
- Inspect via GitMap search (no edits):
  - `src/components/instances/PromptTreeView/DetailHeader.tsx` — Send
    dropdown? Enqueue entry? New-conversation option? UI polish state?
  - `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`
    — Send path: direct dispatch or enqueue? Works on inactive projects?
  - Queue backend — FIFO ordering? Enqueue working?
  - `src/components/instances/PromptTreeView/clipboard.ts` — copy format,
    image handling.
  - Tree view — copy-all? import/export (SQLite/JSON)?
- For each requirement: DONE (with file:line evidence) / PARTIAL (what's
  missing) / MISSING.

## Output
Report via send_message to the lead with the per-requirement verdict table.
Do not write files. Do not run git.

## Constraints
- Read-only. GitMap search only. Relative paths only.
