# CI/CD Issue 07 — Cross-spec link checker false-positives in `26-spec-outsides`

**Status:** ✅ Solved
**Rule:** missing-file (custom link checker)
**Date:** 2026-04-23

---

## Symptom

CI fails with:

```
[missing-file] 02-spec/26-spec-outsides/09-pipeline/10-known-issues-and-fixes.md:347
  text:   02-spec/17-consolidated-guidelines/16-cicd.md
  target: ../12-consolidated-guidelines/16-cicd.md
  detail: .../02-spec/26-spec-outsides/12-consolidated-guidelines/16-cicd.md
```

…plus several similar errors for `../01-app/...`, `../02-app-issues/...`.

## Root Cause

The link checker resolves relative paths from the **referencing file's parent directory**, not from the spec root. A link `../12-consolidated-guidelines/16-cicd.md` written inside `02-spec/26-spec-outsides/09-pipeline/...` resolves to `02-spec/26-spec-outsides/12-consolidated-guidelines/16-cicd.md` — which does not exist.

The author intended a repo-root-relative path.

## Solution

Rewrite all such links in `26-spec-outsides/**` to climb out of the `26-spec-outsides/` umbrella before targeting a sibling top-level folder:

```
../../12-consolidated-guidelines/16-cicd.md
../../01-app/09-build-deploy.md
../../02-app-issues/13-release-pipeline-dist-directory.md
```

Two `../` segments — one to escape the `09-pipeline/` subfolder, one to escape `26-spec-outsides/`.

## What NOT to Repeat

- Never assume the link checker is repo-root-relative. It is **file-relative**.
- Never copy spec links from one folder into another without re-verifying the `../` count.
- Always run `linters-cicd/run-all.sh` (or the per-check script) locally before pushing cross-spec edits.

---

*CI/CD issue 07 — v1.0.0 — 2026-04-23*
