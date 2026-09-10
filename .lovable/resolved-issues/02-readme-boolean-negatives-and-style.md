# Quickstart Example — Boolean Negatives + Style Violations

**Status:** ✅ Solved
**Date:** 2026-04-23
**Rules:** CODE-RED-023 (raw `!` operator), STYLE-001 (blank line before return), STYLE-004 (blank line before if)

## Description

The "Refactored" example in `readme.md` and `02-spec/17-consolidated-guidelines/05-coding-guidelines.md` contained:
- Negative property `user.banned` (P1 violation)
- Raw `!user` and `!isEligible(...)` operators (P2/P8 violation)
- Single-line `if (...) return ...;` blocks (STYLE-001/004)
- Missing blank lines before `return`

## Solution

- Renamed `user.banned` → `user.hasGoodStanding` (positive form).
- Renamed `user.active` → `user.isActive`.
- Replaced `!user` with named guard `isUserMissing(user)`.
- Replaced `user.role !== ROLE_ADMIN` with `isAdmin(user) === false`.
- Replaced `!isEligible(user)` with `isEligible(user) === false`.
- Expanded all `if` to multi-line braced blocks with blank lines before `return`.

## Files

- `readme.md`
- `02-spec/17-consolidated-guidelines/05-coding-guidelines.md`

## Learning

Spec example code is validated by the same linter as production code. Always run `bash linters-cicd/run-all.sh` after editing markdown that contains code samples.

## What NOT to repeat

Never write `!variable` in any example, doc, or production file. Either use `=== false` for booleans or extract a positively-named guard function.
