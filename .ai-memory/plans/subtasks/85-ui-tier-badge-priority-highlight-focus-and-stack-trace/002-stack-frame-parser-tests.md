---
plan: 85-ui-tier-badge-priority-highlight-focus-and-stack-trace
subtask: "002"
title: Add regression tests for the stack frame parser
domain: frontend
depends_on: 001-stack-frame-parser-regex.md
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#acceptance-criteria
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../issues/48-stack-frame-parser-swallows-url-rca.md
  ambiguity: ../../../ambiguous-questions/02-ambiguity-resolved/04-frontend-unit-test-runner.md
target_files:
  - src/lib/__tests__/stack-frame-parser.test.ts (new, standalone assertion script like the existing tests)
  - scripts/run-frontend-tests.mjs (existing runner, `npm run test`)
status: completed
---

# 002 — Add regression tests for the stack frame parser

## 1. Context
The frontend test runner is fixed (resolved ambiguity 04): `npm run test` runs every `src/**/__tests__/*.test.ts` through `npx tsx`, no new dependency. The parser fix needs a table-driven regression test in the same style as `src/services/__tests__/instanceService.test.ts`.

## 2. Target files and symbols
- src/lib/__tests__/stack-frame-parser.test.ts (new, standalone assertion script like the existing tests)
- scripts/run-frontend-tests.mjs (existing runner, `npm run test`)

## 3. Steps
1. Write table-driven cases: URL-only frame, named frame with parentheses, `async` frame, `eval` frame, Windows drive path, `http://localhost:1420/src/x.tsx:10:5`, malformed line returns `null`.
2. Assert function, file, line, column for each case; throw at the end when any case failed (same pattern as the existing tests).

## 4. Constraints
- US English spelling; boolean names use `is`/`has`; never write `== true`.
- Function size and file size follow `02-spec/02-coding-guidelines/02-canonical-size-tier.md`; each new component or helper lives in its own file.
- Relative paths only in code, scripts, and docs.
- Reuse existing UI conventions (pill/badge toggles, existing dialogs); no new unrelated controls.
- No new dependency; use only the existing runner.

## 5. Out of scope
- Version bump, changelog, release notes, readme version pin (release fires only when the whole plan is complete and the user commands it).
- Any file not listed in section 2 unless a compile error forces a minimal adjacent edit (record it in the plan status).

## 6. Verify
```bash
npm run test -- stack-frame-parser
npm run test
npm run build
```

## 7. Done When
- [ ] All cases pass locally.
- [ ] The pre-fix regex fails the URL-only case (prove by temporarily restoring it, then revert).

## 8. Ambiguities and interim defaults
- None open for this step.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
