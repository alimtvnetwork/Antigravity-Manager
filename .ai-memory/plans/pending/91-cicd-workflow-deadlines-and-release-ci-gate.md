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
