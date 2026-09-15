# CI/CD Issue 06 — Version drift after package.json bump

**Status:** ✅ Solved
**Rule:** version-drift (custom CI gate, `scripts/sync-version.mjs`)
**Date:** 2026-04-23

---

## Symptom

CI job fails with:

```
Error: Drift detected in version.json (git/updated fields ignored).
Error: Fix: run 'npm run sync' locally and commit the resulting changes.
Error: Process completed with exit code 1.
```

Diff shows `version.json` `stats.totalFiles`, `totalLines`, and per-folder `fileCount`/`lineCount` mismatching `HEAD` vs working tree.

## Root Cause

`package.json` was bumped (3.81.0 → 3.92.0 across the session) without running `npm run sync` in the same commit. The sync script regenerates:

- `version.json` (totals + per-folder stats)
- `public/health-score.json`
- `src/data/specTree.json`

If those files lag behind `package.json`, the CI gate fails.

## Solution

After every `package.json` version bump:

```bash
npm run sync
git add version.json public/health-score.json src/data/specTree.json
```

Then commit.

## What NOT to Repeat

- Never bump `package.json.version` in isolation. The bump and the sync output **must** ship in the same commit.
- Never edit `version.json` by hand — it is fully derived.

## Suggested follow-up

Add a `pre-commit` hook (see `.lovable/suggestions.md`) that runs `npm run sync` automatically when `package.json` is staged.

---

*CI/CD issue 06 — v1.0.0 — 2026-04-23*
