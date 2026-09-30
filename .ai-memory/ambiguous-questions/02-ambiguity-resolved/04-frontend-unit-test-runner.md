# Which runner should run frontend unit tests, given `package.json` has no test script?

Slug: frontend-unit-test-runner
Status: resolved
Raised: 2026-09-30
Blocking: 85 (subtask 002)

## Question
Which runner should run frontend unit tests, given `package.json` has no test script?

## Context
Test files exist under `src/**/__tests__` but no runner or `test` script is declared. Adding a dev dependency is a repo-level decision (cross-platform, CI compiles tests without executing).

## Options considered
- A: `vitest` (Vite-native) as a dev dependency and a `test` script.
- B: no runner; keep the parser a pure module verified by `tsc` and a Rust-free script.
- C: Node built-in `node --test` with TypeScript stripping.

## Impact if guessed wrong
Option A adds a dependency; B leaves the regression unguarded.

## Interim provisional default
Option A, added in subtask 002 only, with CI continuing to compile without running. Falls back to B if the maintainer declines.

## Resolution

Answered: 2026-09-30
Answer: Fix the test runner with whatever works. Chosen: no new dependency. `npm run test` runs every `src/**/__tests__/*.test.ts` standalone script through `npx tsx` (`scripts/run-frontend-tests.mjs`). Adding vitest would have changed the lockfile (esbuild bump), so it was rejected.
Applied solution: Runner added and green (2/2 files). Two stale test expectations fixed (score normalization `/ 1000`, exact model ids from fix #3344). Plan 85 step 002 uses it.
