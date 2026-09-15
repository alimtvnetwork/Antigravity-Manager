# Slides system overhaul + repo improvement audit

Slug: slides-system-overhaul
Steps: 5
Status: in-progress (steps 3, 4, 5 shipped; steps 1, 2 remain)

## Reconciliation (2026-07-19, v5.56.0)

- Step 3 (release.mjs slides zip + README link): DONE in v5.52.0 - v5.53.0.
- Step 4 (`scripts/sync-guidelines.mjs` + CI drift fail): DONE in v5.50.0.
- Step 5 (smoke + axe-core a11y + Playwright visual regression + SRA validator): DONE across v5.49.0 - v5.55.0.
- Step 1 (publish 3-theme audit): subtask written, publication pending.
- Step 2 (execute 70-task backlog): 13/70 shipped, 57 remain. See SS-02.
Created: 2026-07-19

## Context

User requested (a) a 3-theme repo improvement summary and (b) a 60-100 task plan to evolve `slides-app/` into a slide-per-rule teaching deck for `02-spec/17-consolidated-guidelines/34-compiled-simple-coding-guidelines.md` (v1.4.0), with the built deck attached to every release.

Captured inputs:
- Command: .lovable/02-spec/commands/01-slides-attached-to-release.md
- Command: .lovable/02-spec/commands/02-improvement-audit-cadence.md

Prior pending scan: `.lovable/plans/` did not exist before this turn, so no prior pending tasks were carried over. `.lovable/cicd-issues/` items (01-07) are separate CI-lint issues, unrelated to the slides overhaul, and stay in their existing tracker.

Detailed material lives in subtasks to keep this file skimmable.

## Steps

1. Publish the 3-theme repo-wide improvement audit (guideline SSOT + enforcement parity, slides as canonical teaching surface, release+CI hardening). See ./subtasks/01-slides-system-overhaul/ss-01-repo-improvements-3-themes.md.
2. Execute the 70-task slides backlog (deck infra → content by section → build/release/test). See ./subtasks/01-slides-system-overhaul/ss-02-slides-70-task-backlog.md.
3. Wire `slides-app` build+zip into `scripts/release.mjs` and attach `slides-app/dist.zip` to the GitHub Release, per command 01; add a "Slides deck" link in the root README release section.
4. Add `scripts/sync-guidelines.mjs` that regenerates deck rule cards from file 31 and fails CI on drift (slide count < rule count, or missing rule id).
5. Extend `slides-app/tests/smoke.spec.ts` with a per-rule presence assertion and add axe-core a11y + Playwright visual regression per slide; gate the release script on all three.

## Verification

- Step 1: subtask file exists and is linked from this plan.
- Step 2: subtask file lists 70 numbered tasks grouped A-H; each subsequent implementation PR moves items to a checked state.
- Step 3: `npm run release` dry-run produces `slides-app/dist.zip`; GitHub Release page shows the asset; README release section renders the link.
- Step 4: `scripts/sync-guidelines.mjs --check` exits non-zero when a rule id in file 31 has no matching slide.
- Step 5: `bunx playwright test` runs smoke + a11y + visual specs green; CI job `slides-smoke.yml` blocks merge on failure.

## Appended from prior pending tasks

none
