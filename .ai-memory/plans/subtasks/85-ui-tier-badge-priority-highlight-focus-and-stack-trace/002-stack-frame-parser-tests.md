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
  ambiguity: ../../../ambiguous-questions/01-new-ambiguity/04-frontend-unit-test-runner.md
target_files:
  - package.json — `devDependencies`, `scripts.test` (only if ambiguity 04 resolves to option A)
  - src/lib/__tests__/stack-frame-parser.test.ts (new)
status: pending
---

# 002 — Add regression tests for the stack frame parser

## 1. Context
`src` already contains `__tests__` folders but `package.json` declares no runner. The parser fix needs a table-driven regression test.

## 2. Target files and symbols
- package.json — `devDependencies`, `scripts.test` (only if ambiguity 04 resolves to option A)
- src/lib/__tests__/stack-frame-parser.test.ts (new)

## 3. Steps
1. Read ambiguity 04. Apply the interim default (vitest as a dev dependency with a `test` script) unless the user resolved it otherwise.
2. Write table-driven cases: URL-only frame, named frame with parentheses, `async` frame, `eval` frame, Windows drive path, `http://localhost:1420/src/x.tsx:10:5`, malformed line returns `null`.
3. Assert function, file, line, column for each case.

## 4. Constraints
- US English spelling; boolean names use `is`/`has`; never write `== true`.
- Function size and file size follow `02-spec/02-coding-guidelines/02-canonical-size-tier.md`; each new component or helper lives in its own file.
- Relative paths only in code, scripts, and docs.
- Reuse existing UI conventions (pill/badge toggles, existing dialogs); no new unrelated controls.
- Dependency change is minimal: one dev dependency, no config beyond what vitest needs; CI keeps compiling without executing tests.

## 5. Out of scope
- Version bump, changelog, release notes, readme version pin (release fires only when the whole plan is complete and the user commands it).
- Any file not listed in section 2 unless a compile error forces a minimal adjacent edit (record it in the plan status).

## 6. Verify
```bash
npm run test -- stack-frame-parser
npm run build
```

## 7. Done When
- [ ] All cases pass locally.
- [ ] The pre-fix regex fails the URL-only case (prove by temporarily restoring it, then revert).

## 8. Ambiguities and interim defaults
- Ambiguity 04 default: vitest. If declined, verify with a Node script instead and record that in the plan status.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
