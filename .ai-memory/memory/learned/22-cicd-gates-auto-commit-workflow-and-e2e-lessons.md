# Learned 22: CI/CD Gates, Auto-Commit Workflow, and E2E Lessons (2026-10-01)

## What changed
| Commit | Change |
|--------|--------|
| `056ad955` | `scripts/test-instance-e2e.ps1` creates `build-demo/` itself; prompt backup is scoped with `agm backup-running-prompts --instance <id>`. E2E passed twice on Windows. |
| `fb86a734` | rustfmt fix in `src-tauri/src/bin/agm.rs` (`cmd_instances` arg filter) that had kept CI red since `bf491cfd`. |
| `09338155` | Tracked `.githooks/pre-commit` blocks staged unformatted `src-tauri/*.rs`; installed by `npm install` (`prepare` → `scripts/install-git-hooks.mjs`) or `npm run hooks:install`. |
| `8c8c096f` | RCA 40, RCA 41, CI index audit; `03-ai-scripts/33-test-inventory-generator.py` accepts list or dict manifests and epoch `updated_at`. |
| `6b18ddc4` | `release.yml` → `verify-release-target` runs `cargo fmt -- --check` before any build job. |
| `bc2a303f` | Issue 56 closed. First fully green CI run including the release gate. |

## Root cause to remember
The rustfmt rule was documentation only. A bump commit carried hand-edited Rust, and each new push cancelled the in-flight CI run, so the red went unseen while `v4.109.1`–`v4.109.4` published. Full write-up: `.ai-memory/cicd-issues/40-recurring-rustfmt-drift-and-releases-shipping-on-red-ci-rca.md`.

## Operational facts
- `gh` defaults to `upstream` (`lbjlaq/Antigravity-Manager`). Always pass `-R alimtvnetwork/Antigravity-Manager`.
- `gh run list --commit` needs the full SHA; a short SHA returns nothing.
- Pushing `.github/workflows/*` needs a token with the `workflow` scope (`gh auth refresh -h github.com -s workflow; gh auth setup-git`). Granted on the maintainer host on 2026-10-01; keep workflow edits in their own commit anyway.
- Local `cargo clippy --all-targets --all-features` runs out of memory on the 8 GB Windows host. Trust CI for Clippy.
- `.ai-memory/temp/recent-file-changes.json` is a per-run cache that is tracked; it shows modified in every tree. Do not stage it with feature work (suggestion 04 item 8 proposes untracking it).
- The E2E script can look hung for minutes during `agm stop`/`ff`/`switch` because output is buffered. Wait for it rather than killing it.

## Maintainer workflow preference (auto commit and push)
The maintainer wants every change committed and pushed without being asked. See `.ai-memory/user-preferences/01-auto-commit-and-push.md`. This is configured on the maintainer host, outside the repo:
- Personal Cursor rule "Always commit and push".
- User hook `~/.cursor/hooks.json` → `~/.cursor/hooks/auto-commit-push.ps1`: `afterFileEdit` records edited files per conversation; `stop` stages only those files, commits `chore(agent): update <files>`, pushes, and on rejection rebases onto upstream and retries. Log: `~/.cursor/hooks/logs/auto-commit-push.log`.
- Cursor settings: `git.postCommitCommand: "push"`, `git.confirmSync: false`, smart commit, autofetch.

Consequence for agents: still stage by explicit path and write a real commit message yourself; the hook is a safety net and its generic message is a last resort.

## Next steps
Plan 91 (`.ai-memory/plans/pending/91-cicd-workflow-deadlines-and-release-ci-gate.md`): pin Ubuntu before 2026-10-19, Node 24 actions, gate releases on full CI success.
