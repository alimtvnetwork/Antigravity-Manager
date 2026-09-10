# fuzzyMatch.ts — Function Length + Negative Guards

**Status:** ✅ Solved
**Date:** 2026-04-23
**Rules:** CODE-RED-004 (function > 15 lines), CODE-RED-012 (`let` instead of `const`), CODE-RED-023 (negative guard)

## Description

- `scoreCharacterMatch` used `let delta` mutation.
- `appendFuzzyPreviewSnippet` exceeded 15 lines.
- Result-empty check used `=== false` on a negative.

## Solution

- Refactored `scoreCharacterMatch` to compute `delta` via `const` ternary.
- Extracted `buildPreviewWindow` from `appendFuzzyPreviewSnippet`.
- Added `isResultEmpty` positively-named guard.

## File

- `src/components/docs/search/fuzzyMatch.ts`

## What NOT to repeat

Never let a scoring/parsing function grow past 15 lines.
