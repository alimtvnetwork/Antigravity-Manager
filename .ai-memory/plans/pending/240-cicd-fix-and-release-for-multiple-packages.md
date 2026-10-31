# Plan 240 - CI/CD Fix and Release for Multiple Packages

Date: 31-Oct-26
Branch: main
Base: 3659500cf (origin/main at pull time)
Current version: 4.283.0 -> target 4.284.0 (minor bump, recomputed
after other agents' v4.282.0/v4.283.0 pushes moved main; original
wave-1 target 4.282.0 is superseded - see CI state update below)

## User request (verbatim summary)

Fix CI/CD until fully green including every linter, minor version bump
with the repo's own bump script, create the release tag and release,
verify the tag on the remote and the release workflow green, and finish
with a clean tree and local equal to origin on the release branch.

## CI state before

- CI run 41195657081 on 3659500cf: failure.
  https://github.com/alimtvnetwork/Antigravity-Manager/actions/runs/41195657081
- Failed jobs: Check Rust Code (ubuntu-24.04, windows-2022, and the
  experimental ubuntu-26.04 entry) and Build Tauri App (ubuntu-24.04,
  windows-2022). Build Frontend passed.
- Root causes: E0609 in
  src-tauri/src/modules/auto_switcher/manual.rs:188 (no field
  `rolling_4h_quota` on ProfileCandidate) and E0594 x2 in
  src-tauri/src/modules/repo_db/agy.rs:1138-1139 (immutable
  `active_prompt` binding declared at line 1046), plus a fourth error
  masked by the first: E0382 in manual.rs (candidate moved into the
  rotation call, then read for tier classification) - fixed by
  capturing the f64 quota before the move.
- Release run 41195653954 failed only at its CI gate (CI concluded
  failure, release refused). No workflow change is needed.

## Subtasks

- 01 - spec and plan (this plan plus
  02-spec/21-app/240-cicd-fix-and-release-for-multiple-packages/)
- 02 - compile-error fix (scoring.rs field, selection.rs seven sites,
  agy.rs `mut`) plus local fmt and clippy pre-flight
- 03 - bump, tag, release, and final verification
- 04 - wave-2 E0500 fix (tree_node_conv.rs index-first borrow) on tip
  608abea60, after other agents' v4.282.0/v4.283.0 pushes moved main

## CI state update (wave 2)

- Wave-1 fix commit 45edad629: CI run 41201814856 fully green (all 6
  jobs success).
- New tip 608abea60 (other agents' work on top): CI run 41202605011
  failed, single error E0500 in tree_node_conv.rs:856. Fixed in this
  run; the release base is therefore the post-wave-2 tip, and the
  minor bump is recomputed from the then-current version (4.283.0 ->
  4.284.0), not the stale 4.282.0 target from wave 1.

Subtask files live in
.ai-memory/plans/subtasks/240-cicd-fix-and-release-for-multiple-packages/.

## Layout note

The spec sequence in 02-spec/21-app/ runs to 239, so this task is 240.
The pending-plan folder was empty except .gitkeep; this plan file is
its first pending entry. No layout conflict.

## Rules applied

- Search exclusively via gitmap (aum search / find / cat). No rg,
  ripgrep, grep, or git grep for code search.
- Strictly relative git paths in all docs, commits, and release notes.
- Dates DD-MMM-YY. Code in UTC+0. Branch names date only.
- Commit with the gitmap commit suite, hyphen format, no colons in the
  message argument. Configured identity: gitmap automation bot - muse -
  runs autonomously <info@rgitmap.com> (repo-local, verified before
  committing). Never force-push, never rewrite history under tags.
- No .github/workflows/ change in this run, so the known
  workflow-scope token blocker does not apply; if one becomes needed
  it is committed locally and reported as blocked, never bypassed.
- Headless VM: no Tauri GUI launch; `agm` CLI headless only if needed.
  Verification is via local clippy/fmt plus remote CI, once at the end
  of the tunnel.

## Open questions

None blocking. Pre-existing clippy warnings (~1234) are reported, not
fixed here. The beta v5.x line and v5.0.12 tag are out of scope.
