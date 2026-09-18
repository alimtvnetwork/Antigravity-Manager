# Session — 2026-04-23 — README CODE-RED Walkthrough + Spec References

**Status:** ✅ Done
**Version:** package.json bumped 3.81.0 → 3.92.0 (multiple minor bumps across commits)
**Updated:** 2026-04-23

---

## What Was Done

1. **CODE-RED Validation Walkthrough** added to root `readme.md` for the `riseup-asia-uploader` real-world example. Each rule isolated into its own snippet (Reject vs Require table) — rules covered:
   - Error context (no naked `errors.New`/`fmt.Errorf`)
   - Boolean naming (no `Not`/`No` prefixes; Approved Inverses)
   - Magic strings/numbers (named constants/enums)
   - Single-value returns via `Result[T]`
   - Code mutation / immutability
   - Type aliases for common generic Result shapes
2. **CODE-RED Rules summary section** — 8-row table mapping `CODE-RED-001..023` with the canonical spec link and a quick "where the walkthrough lives" pointer.
3. **Markdown hierarchy fix** — promoted four `####` headings to `###` after the new `h2` to keep heading levels contiguous (no level skips). Verified balanced code fences (31 pairs) and TOC anchors.
4. **Spec References index** — 9-row table at the end of the walkthrough indexing every referenced spec/linter file for quick navigation. Fixed broken link for "AI condensed master guidelines" → `02-spec/02-coding-guidelines/06-ai-optimization/04-condensed-master-guidelines.md`.
5. **Sync** — ran `npm run sync` after each version bump to refresh `version.json`, `public/health-score.json`, `src/data/specTree.json`.

## Files Edited

- `readme.md` (411-line addition over the session)
- `package.json` (3.81.0 → 3.92.0 in increments)
- `version.json`, `public/health-score.json`, `src/data/specTree.json` (sync output)
- `scripts/sync-version.mjs`, `scripts/sync-spec-tree.mjs` (minor)
- `docs/architecture.md`, `docs/author.md`, `docs/principles.md` (auto-touched by sync)

## Decisions

- **Isolate one rule per snippet** in walkthroughs — earlier draft mixed multiple violations and weakened the "fixed" example. New convention: every Reject/Require pair models exactly one rule.
- **Heading hierarchy is non-negotiable** — never skip from `h2` to `h4`. Promote `####` → `###` if they sit directly under an `h2`.
- **Link verification** — every spec path in any new index table must be `ls`-checked before commit.

## Learnings

- The CI "version drift" job (`/tmp/version.head.json` vs `/tmp/version.work.json`) fires whenever `package.json` is bumped without running `npm run sync` locally. Fix is always: `npm run sync` then commit the resulting `version.json` / `health-score.json` / `specTree.json` deltas.
- The CI "missing-file" job under `linters-cicd/` resolves cross-spec links from the *file's directory*, not from repo root. Relative paths starting with `../` are computed against the spec file's parent — this is what triggered the `26-spec-outsides/12-consolidated-guidelines/16-cicd.md` false positive.

## Next Logical Step

- Continue with the active backlog (BOOL-NEG-001 smoke test, Go-aware variant, codegen round-trip tests) — see `29-plan.md` Active Work table.
- Address the `installer-behavior-rollout` Phase 2 (generator template update) — now folded into `29-plan.md`.

---

*Session memory — v1.0.0 — 2026-04-23*
