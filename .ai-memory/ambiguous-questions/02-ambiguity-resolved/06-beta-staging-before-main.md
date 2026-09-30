# Should the changes be staged on `beta` before `main`?

Slug: beta-staging-before-main
Status: resolved
Raised: 2026-09-30
Blocking: none (release step only)

## Question
Should the changes be staged on `beta` before `main`?

## Context
Repo rule: confirm with the maintainer whether new changes are implemented and tested on `beta` first. The update-gating work (plan 86) and the switch pipeline (plan 87) are non-trivial.

## Options considered
- A: stage all four plans on `beta`, then merge.
- B: stage plans 86 and 87 on `beta`, ship 85 and 88 directly.
- C: all on `main`.

## Impact if guessed wrong
Skipping staging on risky paths can push a bad update gate to stable users.

## Interim provisional default
Option B. No branch is created or pushed until the user commands implementation.

## Resolution

Answered: 2026-09-30
Answer: Work directly on `main`; the user has been committing plan changes there and asked not to wait for confirmation.
Applied solution: No branch created. Plan 86 keeps a post-publish verification job as its safety net.
