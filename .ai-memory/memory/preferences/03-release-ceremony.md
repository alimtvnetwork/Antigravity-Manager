---
name: Release Ceremony
description: Full release ceremony triggered by "release", "bump version", or "bump version + add changelog + pin to root readme"
type: preference
---

# Release

> **Trigger phrases:** `release` · `bump version` · `bump version + add changelog + pin to root readme`

When the user says **release** (or any trigger phrase above), perform the full release ceremony in a single turn. Do **not** ask for confirmation. Do **not** open plan mode. Just do it.

## Mandatory steps (all of them, in order)

1. **Pick the bump tier**
   - Patch (`x.y.Z`) — lint fix, comment, doc-only, no behavior change.
   - Minor (`x.Y.0`) — new feature, new prompt, UI change, behavior change that is backward compatible.
   - Major (`X.0.0`) — breaking change to storage, prompt schema, public API, or extension contract.
   - If unsure, default to **minor**.

2. **Bump the version everywhere** — these files MUST all match the new version:
   - `version.json`
   - `manifest.json`
   - `src/shared/constants.ts` (`EXTENSION_VERSION`)
   - `readme.md` (root pin / badges)
   - `standalone-scripts/marco-sdk/src/index.ts` and `standalone-scripts/marco-sdk/src/prompts.ts` (`CACHE_SCHEMA_VERSION`)
   - Every `standalone-scripts/*/src/instruction.ts`
   - `standalone-scripts/macro-controller/src/shared-state.ts`
   - `standalone-scripts/payment-banner-hider/src/index.ts`

3. **Update `changelog.md`** — add a new top entry under the `# Changelog` header:
   ```
   ## [vX.Y.Z] — YYYY-MM-DD <short headline>
   ### Fixed / Added / Changed / Removed
   - <one bullet per real change, naming the exact file or behavior>
   ```
   Never invent changes. Only list work actually done in this release.

4. **Pin the new version in the root `readme.md`** wherever the current version appears (badges, install snippets, "current version" lines).

5. **Regenerate bundled prompts** if any prompt source under `standalone-scripts/prompts/` was touched:
   ```
   node scripts/aggregate-prompts.mjs
   ```

6. **Verify no stale references remain**:
   ```
   rg -l "<previous-version>" . | grep -v node_modules | grep -v "\.git/"
   ```
   Anything that surfaces (other than historic changelog entries and intentional code comments) must be updated.

7. **Report** the new version, the bump tier, and the exact files changed. No filler, no apologies.

## Hard rules

- Never bump only part of the files. Version pins must be in lock-step.
- Never skip the changelog entry. A release without a changelog row is invalid.
- Never include a changelog bullet for work that was not actually done in this turn.
- Never auto-publish or call any deploy tool unless the user explicitly says publish/deploy/ship/go live.
- Never ask "should I bump minor or patch?" — pick using the rules above and proceed.

## Why

The release trigger phrase exists so the user can ship without re-explaining the ceremony. If any pin is missed, the extension boots with mismatched versions and prompt caches go stale. Do the full ceremony every time.
