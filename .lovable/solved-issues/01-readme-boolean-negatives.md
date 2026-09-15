# Solved — readme.md Boolean Negatives + Style Violations

**Date:** 2026-04-23
**Mirror of:** `.lovable/cicd-issues/01-readme-boolean-negatives-and-style.md`

## Solution

Renamed `user.banned` → `user.hasGoodStanding` and `user.active` → `user.isActive`. Replaced raw `!user` with `isUserMissing` guard, replaced `!isEligible(...)` with `isEligible(...) === false`. Expanded all `if` statements to multi-line braced blocks with blank lines before `return`.

## Iteration count

1 attempt — clean fix on first pass.

## Learning

The Boolean Principles spec (`02-spec/02-coding-guidelines/01-cross-language/02-boolean-principles`) applies to **every** code sample in markdown, not just production source. Always re-run `bash linters-cicd/run-all.sh`.

## What NOT to repeat

- Never write `!variable` or `!fn()` in any sample.
- Never use a negative property name (`banned`, `disabled`, `failed`) when a positive form (`hasGoodStanding`, `isEnabled`, `isSuccessful`) exists.
