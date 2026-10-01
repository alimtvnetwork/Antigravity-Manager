# User Preference 01: Always Commit and Push, Never Ask

Recorded: 2026-10-01
Source: maintainer instruction ("always commit and push full access, never ask")

## Rule
After finishing any change, commit and push to the current branch without asking for permission.

## How
- Stage only the files you changed, by explicit path. Never `git add -A` or `git add .`.
- Write a concise conventional commit message describing the final state.
- Push. If the remote is ahead, `git pull --rebase` and push again.
- Exclude unrelated files, per-run caches (`.ai-memory/temp/recent-file-changes.json`), generated artifacts, and secrets.

## Never
- Force-push, `--no-verify`, `reset --hard`, or `clean` to make a push go through.
- Create release tags because of this rule. Releases still follow AGENTS.md and `docs/release_guide.md`.

## If blocked
Report the exact error and the one command the maintainer must run (for example, a missing `workflow` token scope: `gh auth refresh -h github.com -s workflow; gh auth setup-git`).

## Enforcement on the maintainer host
A Cursor user hook auto-commits agent-edited files when the agent stops. Details: `.ai-memory/memory/learned/22-cicd-gates-auto-commit-workflow-and-e2e-lessons.md`.
