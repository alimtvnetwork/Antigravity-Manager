# Plan: End-to-End Testing, Prompt Tree Crash Fix, and Update Asset Gate

- **Slug**: `end-to-end-testing-and-ui-update-instructions`
- **Status**: `COMPLETED`
- **Spec**: `02-spec/21-app/end-to-end-testing-and-ui-update-instructions/01-architecture-spec.md`

## Subtasks

| # | File | Scope | Status |
| :--- | :--- | :--- | :--- |
| `01` | `.ai-memory/plans/subtasks/end-to-end-testing-and-ui-update-instructions/01-prompt-tree-oom-and-update-asset-gate.md` | Fix Prompt Tree `useEffect` render loop (`0xe0000008` OOM), serialize tree scans, verify platform assets before announcing updates, restyle `UpdateNotification`, and expose `prompts resend` / `prompts copy` in CLI | `COMPLETED` |
| `02` | `.ai-memory/plans/subtasks/end-to-end-testing-and-ui-update-instructions/02-sequential-e2e-and-minor-release.md` | Sequential E2E verification on Windows (Prompt Tree speed & Reset button, running prompts JSON/terminal/UI, resend, copy, restore, auto-switch, UI screenshots), then minor version bump and CI/CD check | `COMPLETED` |

## Rules
- Run all builds and E2E checks **sequentially** (single agent, `-j 1` Cargo builds) so system commit memory stays well below the 8.7 GB ceiling.
- Search exclusively via `gitmap`.
- Use only relative repository paths in all tracked files, changelogs, and release notes.
