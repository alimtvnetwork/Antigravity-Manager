# Should Focus scroll to the account bound to the selected instance, or to the globally current account?

Slug: focus-button-target-account
Status: resolved
Raised: 2026-09-30
Blocking: 85 (subtask 009)

## Question
Should Focus scroll to the account bound to the selected instance, or to the globally current account?

## Context
The user said Focus should scroll to the selected one. With several instances the selected instance's account and the global current account can differ.

## Options considered
- A: selected instance's bound account, fallback to the global current account.
- B: always the global current account.

## Impact if guessed wrong
Wrong target makes Focus appear broken again.

## Interim provisional default
Option A.

## Resolution

Answered: 2026-09-30
Answer: User asked to stop asking; default applied: the selected instance's bound account, falling back to the global current account.
Applied solution: Plan 85 step 009.
