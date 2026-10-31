# Plan 244 - yt-manage-gitmap-antigravity-package (Antigravity-Manager leg)

**Date**: 01-Nov-26
**Spec**: `02-spec/21-app/244-yt-manage-gitmap-antigravity-package/01-architecture-spec.md`
**Subtasks**: `.ai-memory/plans/subtasks/244-yt-manage-gitmap-antigravity-package/`
**Target branch**: `main`
**Status**: IN_PROGRESS

## Context (01-Nov-26)

The task brief assumed tip `88857d224` (duplicate 4.284.0 bump, CI green).
Pull superseded that state: tip is `073394179` (v4.287.0), six commits past
the released v4.284.0 commit `d5769c5d7`, with functional changes and RED CI.
Decision branch (b) applies with the version adjusted to the next free
version, 4.288.0. Full evidence and reasoning live in the spec.

## Subtasks

- [ ] 01 - Spec, plan, subtasks and ambiguity record (this plan's setup)
  - File: `.ai-memory/plans/subtasks/244-yt-manage-gitmap-antigravity-package/01-spec-and-plan.md`
- [ ] 02 - CI diagnosis from remote runs (no local full build)
  - File: `.ai-memory/plans/subtasks/244-yt-manage-gitmap-antigravity-package/02-ci-diagnosis.md`
- [ ] 03 - One-line E0061 fix in `src-tauri/src/modules/repo_db/queue.rs`
  - File: `.ai-memory/plans/subtasks/244-yt-manage-gitmap-antigravity-package/03-fix.md`
- [ ] 04 - Bump minor to 4.288.0, tag `v4.288.0`, release
  - File: `.ai-memory/plans/subtasks/244-yt-manage-gitmap-antigravity-package/04-bump-tag-release.md`
- [ ] 05 - Final verification: `gitmap pe -t` green, tag and release confirmed
  - File: `.ai-memory/plans/subtasks/244-yt-manage-gitmap-antigravity-package/05-verification.md`

## Commit order

1. Spec + plan + subtasks + ambiguity record (one grouped commit, pushed).
2. Fix commit (pushed), CI green check.
3. Bump commit (pushed), CI green check, then tag + tag push (raw git,
   declared exception), then Release workflow watched to success.
4. Move this plan to `.ai-memory/plans/completed/` only after the release
   is verified on the remote.

## Guardrails

- Antigravity-Manager only; no other repo touched.
- No force-push, no tag moves, no re-bump of 4.284.0 / 4.285.0.
- No workflow-file changes (VM token lacks `workflow` scope).
- Remote CI is the arbiter; no local full build or test suite.
- Relative Git paths only; dates DD-MMM-YY.
