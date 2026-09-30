# Should the changes be staged on `beta` before `main`?

Slug: beta-staging-before-main
Status: open
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
