# Plan 91: CI Workflow Deadlines and Release CI Gate

> **Status:** PENDING
> **Raised:** 2026-10-01
> **Suggestion:** [suggestions/04-cicd-release-and-workflow-hardening.md](../../suggestions/04-cicd-release-and-workflow-hardening.md)
> **Related:** RCA 40, RCA 42, issue 56 (resolved), plan 86
> **Branch:** stage on `beta` first per AGENTS.md (confirm with maintainer)

## Goal
Keep CI green past the announced GitHub runner changes, and make a release impossible on a commit whose CI is not `success`.

## Subtasks
- **01 (deadline 2026-10-19):** pin `ubuntu-24.04` in every workflow that uses `ubuntu-latest` (`ci.yml`, `release.yml`, `deploy-pages.yml`, `purge-actions-artifacts.yml`); add a non-blocking Ubuntu 26 matrix entry to validate WebKitGTK package names. -> [ ]
- **02:** bump `actions/checkout` and `actions/setup-node` to Node 24-native majors; remove `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24` once nothing needs it. -> [ ]
- **03:** in `release.yml` → `verify-release-target`, fail unless the `ci.yml` run for `github.sha` concluded `success` (`cancelled` or missing = fail). -> [ ]
- **04:** set `cancel-in-progress: ${{ github.ref != 'refs/heads/main' }}` in `ci.yml` so runs on `main` always finish. -> [ ]
- **05:** untrack `.ai-memory/temp/recent-file-changes.json` (`git rm --cached` + `.gitignore`). -> [ ]

## Acceptance
- CI green on the commit that lands subtasks 01–04, with no Node 20 deprecation annotation.
- A test tag on a commit with a `cancelled` CI run is refused by `verify-release-target`.
- `git status` stays clean after running `03-ai-scripts/33-test-inventory-generator.py --record`.

## Constraints
- Workflow edits need a token with the `workflow` scope. Check `gh auth status` first; keep workflow edits in their own commit.
- No releases or tags from this plan.

## Progress (2026-10-23, subtask 190-05)

- Subtask 01 implemented: all four workflows now pin `ubuntu-24.04`; `ci.yml` also has a non-blocking experimental Ubuntu 26.04 matrix entry for the Rust job so WebKitGTK package names can be validated without failing the required run.
- Subtask 02 partially implemented: `actions/checkout` and `actions/setup-node` were moved to their Node 24-native v5 majors. `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: true` was retained because the other actions used by these workflows were not verified as Node 24-native during this implementation-only pass; remove it only after that verification.
- Subtask 03 implemented: `release.yml` → `verify-release-target` now requires the latest completed `ci.yml` run for `github.sha` to have concluded `success`; a cancelled or missing run fails the gate.
- Subtask 04 implemented: `ci.yml` uses `cancel-in-progress: ${{ github.ref != 'refs/heads/main' }}`, so runs on `main` are not cancelled by newer runs.
- Subtask 05 partially implemented: `.gitignore` now ignores `.ai-memory/temp/recent-file-changes.json`. The `git rm --cached` untracking step was not executed by worker 05 because this subtask forbids all git commands; the lead must run the untrack when committing.
- Acceptance is not yet verified: no CI run, test tag, build, or test execution was performed in the implementation phase. This plan remains pending.
