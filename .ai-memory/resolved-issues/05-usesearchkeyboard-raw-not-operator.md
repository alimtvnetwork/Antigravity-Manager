# useSearchKeyboard.ts — Raw `!` Operator

**Status:** ✅ Solved
**Date:** 2026-04-23
**Rule:** CODE-RED-023 (negative guard)

## Description

`if (!isSearchShortcut(e)) return;` used the raw `!` operator on a function call.

## Solution

Inlined the positive condition with named const `isMissingModifier` and explicit `=== false` / inverted predicates.

## File

- `src/components/docs/search/useSearchKeyboard.ts`

## What NOT to repeat

Never write `if (!someFunctionCall())` — assign to a positively-named const first.
